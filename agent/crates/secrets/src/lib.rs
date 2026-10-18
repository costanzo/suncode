//! Platform credential-store access for user-owned SunCode secrets.
//!
//! This crate deliberately stores the secret value directly in the operating
//! system credential store. It does not implement an application-wide key,
//! envelope encryption, or a file fallback.

use std::{
    collections::BTreeMap,
    fmt,
    sync::{Arc, Mutex},
};
use zeroize::Zeroizing;

pub const PROVIDER_API_KEY_SERVICE: &str = "ai.suncode.provider-api-key.v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecretKind {
    ProviderApiKey,
}

impl SecretKind {
    fn service(self) -> &'static str {
        match self {
            Self::ProviderApiKey => PROVIDER_API_KEY_SERVICE,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecretId {
    pub kind: SecretKind,
    pub reference: String,
}

impl SecretId {
    pub fn provider_api_key(provider_id: &str) -> Result<Self, SecretStoreError> {
        let reference = provider_id.trim();
        if reference.is_empty() || reference.len() > 255 {
            return Err(SecretStoreError::InvalidReference);
        }
        Ok(Self {
            kind: SecretKind::ProviderApiKey,
            reference: reference.to_string(),
        })
    }
}

#[derive(PartialEq, Eq)]
pub struct SecretValue(Zeroizing<String>);

impl SecretValue {
    pub fn new(value: String) -> Self {
        Self(Zeroizing::new(value))
    }

    pub fn expose(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_string(self) -> String {
        self.0.to_string()
    }
}

impl fmt::Debug for SecretValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretValue([redacted])")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretStoreError {
    InvalidReference,
    Missing,
    AccessDenied,
    Locked,
    Unavailable,
    Backend(String),
}

impl fmt::Display for SecretStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidReference => "secret reference is invalid",
            Self::Missing => "secret entry is missing",
            Self::AccessDenied => "access to the system credential store was denied",
            Self::Locked => "the system credential store is locked",
            Self::Unavailable => "the system credential store is unavailable",
            Self::Backend(_) => "the system credential store failed",
        })
    }
}

impl std::error::Error for SecretStoreError {}

pub trait SecretStore: Send + Sync {
    fn get(&self, id: &SecretId) -> Result<Option<SecretValue>, SecretStoreError>;
    fn set(&self, id: &SecretId, value: &str) -> Result<(), SecretStoreError>;
    fn delete(&self, id: &SecretId) -> Result<(), SecretStoreError>;
}

#[derive(Clone, Debug, Default)]
pub struct SystemSecretStore;

impl SystemSecretStore {
    fn entry(id: &SecretId) -> Result<keyring::Entry, SecretStoreError> {
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
        {
            keyring::Entry::new(id.kind.service(), &id.reference).map_err(map_keyring_error)
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            let _ = id;
            Err(SecretStoreError::Unavailable)
        }
    }
}

impl SecretStore for SystemSecretStore {
    fn get(&self, id: &SecretId) -> Result<Option<SecretValue>, SecretStoreError> {
        match Self::entry(id)?.get_password() {
            Ok(value) => Ok(Some(SecretValue::new(value))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(map_keyring_error(error)),
        }
    }

    fn set(&self, id: &SecretId, value: &str) -> Result<(), SecretStoreError> {
        if value.trim().is_empty() {
            return Err(SecretStoreError::Backend("secret value is empty".into()));
        }
        Self::entry(id)?
            .set_password(value)
            .map_err(map_keyring_error)
    }

    fn delete(&self, id: &SecretId) -> Result<(), SecretStoreError> {
        match Self::entry(id)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(map_keyring_error(error)),
        }
    }
}

fn map_keyring_error(error: keyring::Error) -> SecretStoreError {
    match error {
        keyring::Error::NoEntry => SecretStoreError::Missing,
        keyring::Error::NoStorageAccess(error) => {
            classify_platform_error(error.as_ref(), SecretStoreError::AccessDenied)
        }
        keyring::Error::PlatformFailure(error) => {
            classify_platform_error(error.as_ref(), SecretStoreError::Unavailable)
        }
        keyring::Error::BadEncoding(_) => SecretStoreError::Backend("secret is not UTF-8".into()),
        keyring::Error::TooLong(_, _) | keyring::Error::Invalid(_, _) => {
            SecretStoreError::InvalidReference
        }
        keyring::Error::Ambiguous(_) => {
            SecretStoreError::Backend("secret entry is ambiguous".into())
        }
        _ => SecretStoreError::Unavailable,
    }
}

fn classify_platform_error(
    error: &(dyn std::error::Error + Send + Sync + 'static),
    fallback: SecretStoreError,
) -> SecretStoreError {
    // Secret Service wraps its backend's locked/prompt errors. Keep the
    // fallback conservative when it does not provide a recognizable reason.
    let normalized = error.to_string().to_ascii_lowercase();
    if normalized.contains("locked")
        || (normalized.contains("interaction") && normalized.contains("not allowed"))
    {
        return SecretStoreError::Locked;
    }
    fallback
}

#[derive(Clone, Default)]
pub struct MemorySecretStore {
    values: Arc<Mutex<BTreeMap<(SecretKind, String), SecretValue>>>,
}

impl SecretStore for MemorySecretStore {
    fn get(&self, id: &SecretId) -> Result<Option<SecretValue>, SecretStoreError> {
        let values = self
            .values
            .lock()
            .map_err(|_| SecretStoreError::Unavailable)?;
        Ok(values
            .get(&(id.kind, id.reference.clone()))
            .map(|value| SecretValue::new(value.expose().to_string())))
    }

    fn set(&self, id: &SecretId, value: &str) -> Result<(), SecretStoreError> {
        if value.trim().is_empty() {
            return Err(SecretStoreError::Backend("secret value is empty".into()));
        }
        self.values
            .lock()
            .map_err(|_| SecretStoreError::Unavailable)?
            .insert(
                (id.kind, id.reference.clone()),
                SecretValue::new(value.to_string()),
            );
        Ok(())
    }

    fn delete(&self, id: &SecretId) -> Result<(), SecretStoreError> {
        self.values
            .lock()
            .map_err(|_| SecretStoreError::Unavailable)?
            .remove(&(id.kind, id.reference.clone()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_store_round_trips_without_debugging_the_secret() {
        let store = MemorySecretStore::default();
        let id = SecretId::provider_api_key("openai").unwrap();
        store.set(&id, "test-secret").unwrap();
        let value = store.get(&id).unwrap().unwrap();
        assert_eq!(value.expose(), "test-secret");
        assert_eq!(format!("{value:?}"), "SecretValue([redacted])");
        store.delete(&id).unwrap();
        assert!(store.get(&id).unwrap().is_none());
    }

    #[test]
    fn provider_reference_is_validated() {
        assert!(SecretId::provider_api_key(" ").is_err());
        assert!(SecretId::provider_api_key("openai").is_ok());
    }
}
