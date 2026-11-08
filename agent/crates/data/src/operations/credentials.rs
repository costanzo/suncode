use crate::{
    domain::CredentialStorageMode,
    store::{lock, now, Store},
};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Nullable, Text};
use suncode_common::BusinessError;
use suncode_secrets::{SecretId, SecretStoreError};

#[derive(Clone, PartialEq, Eq)]
pub struct ProviderCredentialRecord {
    pub provider_id: String,
    pub mode: CredentialStorageMode,
    pub secret_ref: Option<String>,
    pub mask: Option<String>,
}

#[derive(diesel::QueryableByName)]
struct CredentialRow {
    #[diesel(sql_type = Text)]
    provider_id: String,
    #[diesel(sql_type = Text)]
    credential_mode: String,
    #[diesel(sql_type = Nullable<Text>)]
    secret_ref: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    api_key_mask: Option<String>,
}

impl Store {
    pub fn llm_provider_api_key(&self, provider_id: &str) -> Result<Option<String>, BusinessError> {
        self.resolve_provider_credential(provider_id)
    }

    pub fn set_llm_provider_api_key(
        &self,
        provider_id: &str,
        value: &str,
    ) -> Result<(), BusinessError> {
        self.save_provider_credential(provider_id, value)
    }

    pub fn delete_llm_provider_api_key(&self, provider_id: &str) -> Result<(), BusinessError> {
        self.remove_provider_credential(provider_id)
    }

    pub fn provider_credential(
        &self,
        provider_id: &str,
    ) -> Result<Option<ProviderCredentialRecord>, BusinessError> {
        if provider_id.trim().is_empty() {
            return Err(BusinessError::invalid("model provider id is required"));
        }
        let mut connection = lock(&self.connection)?;
        sql_query("SELECT provider_id,credential_mode,secret_ref,api_key_mask FROM llm_model_provider WHERE provider_id=?")
            .bind::<Text, _>(provider_id.trim())
            .get_result::<CredentialRow>(&mut *connection)
            .optional()
            .map_err(crate::database_error)?
            .map(|row| {
                Ok(ProviderCredentialRecord {
                    provider_id: row.provider_id,
                    mode: CredentialStorageMode::parse(&row.credential_mode)?,
                    secret_ref: row.secret_ref,
                    mask: row.api_key_mask,
                })
            })
            .transpose()
    }

    fn set_provider_credential_metadata(
        &self,
        provider_id: &str,
        secret_ref: Option<&str>,
        mask: Option<&str>,
    ) -> Result<(), BusinessError> {
        let mut connection = lock(&self.connection)?;
        // Keep the database layout compatible with existing keychain records.
        // The former plaintext column is reserved and always cleared.
        let changed = sql_query("UPDATE llm_model_provider SET api_key=NULL,credential_mode='keychain',secret_ref=?,api_key_mask=?,updated_at=? WHERE provider_id=?")
            .bind::<Nullable<Text>, _>(secret_ref)
            .bind::<Nullable<Text>, _>(mask)
            .bind::<Text, _>(&now())
            .bind::<Text, _>(provider_id.trim())
            .execute(&mut *connection)
            .map_err(crate::database_error)?;
        if changed == 0 {
            Err(BusinessError::invalid("model provider does not exist"))
        } else {
            Ok(())
        }
    }

    pub fn save_provider_credential(
        &self,
        provider_id: &str,
        value: &str,
    ) -> Result<(), BusinessError> {
        let value = suncode_secrets::SecretValue::new(value.trim().to_string());
        if value.expose().is_empty() {
            return Err(BusinessError::invalid("provider API key is required"));
        }
        let _guard = self.credential_guard()?;
        let current = self
            .provider_credential(provider_id)?
            .ok_or_else(|| BusinessError::invalid("model provider does not exist"))?;
        let reference = current
            .secret_ref
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let id = Self::credential_secret_id(&reference)?;
        let previous = if current.secret_ref.is_some() {
            self.secrets.get(&id).map_err(credential_store_error)?
        } else {
            None
        };
        self.secrets
            .set(&id, value.expose())
            .map_err(credential_store_error)?;
        let mask = mask_secret(value.expose());
        let update =
            self.set_provider_credential_metadata(provider_id, Some(&reference), Some(&mask));
        if update.is_err() {
            let rollback = match previous {
                Some(previous) => self.secrets.set(&id, previous.expose()),
                None => self.secrets.delete(&id),
            };
            if rollback.is_err() {
                return Err(BusinessError::new("credential_recovery_required", "Credential storage update and rollback failed. Check system keychain access and enter the API key again."));
            }
        }
        update
    }

    pub fn remove_provider_credential(&self, provider_id: &str) -> Result<(), BusinessError> {
        let _guard = self.credential_guard()?;
        let current = self
            .provider_credential(provider_id)?
            .ok_or_else(|| BusinessError::invalid("model provider does not exist"))?;
        let previous = if let Some(reference) = current.secret_ref.as_deref() {
            let id = Self::credential_secret_id(reference)?;
            let previous = self.secrets.get(&id).map_err(credential_store_error)?;
            self.secrets.delete(&id).map_err(credential_store_error)?;
            previous.map(|value| (id, value))
        } else {
            None
        };
        let update = self.set_provider_credential_metadata(provider_id, None, None);
        if update.is_err() {
            if let Some((id, previous)) = previous {
                self.secrets.set(&id, previous.expose()).map_err(|_| {
                    BusinessError::new(
                        "credential_recovery_required",
                        "Credential removal rollback failed. Enter the API key again.",
                    )
                })?;
            }
        }
        update
    }

    pub fn resolve_provider_credential(
        &self,
        provider_id: &str,
    ) -> Result<Option<String>, BusinessError> {
        let _guard = self.credential_guard()?;
        let Some(record) = self.provider_credential(provider_id)? else {
            return Ok(None);
        };
        let Some(reference) = record.secret_ref.as_deref() else {
            return Ok(None);
        };
        let id = Self::credential_secret_id(reference)?;
        self.secrets
            .get(&id)
            .map_err(credential_store_error)?
            .map(|value| value.into_string())
            .ok_or_else(|| {
                BusinessError::new(
                    "credential_missing",
                    "The saved system keychain entry is missing. Enter a replacement API key.",
                )
            })
            .map(Some)
    }

    pub fn provider_credential_status(
        &self,
        provider_id: &str,
    ) -> Result<&'static str, BusinessError> {
        let Some(record) = self.provider_credential(provider_id)? else {
            return Ok("missing");
        };
        // Status is a metadata projection. Reading the secret here could
        // trigger a macOS Keychain prompt on every startup.
        Ok(if record.secret_ref.is_some() {
            "configured"
        } else {
            "missing"
        })
    }

    pub fn credential_secret_id(provider_id: &str) -> Result<SecretId, BusinessError> {
        SecretId::provider_api_key(provider_id)
            .map_err(|_| BusinessError::invalid("model provider id is invalid"))
    }

    fn credential_guard(&self) -> Result<std::sync::MutexGuard<'_, ()>, BusinessError> {
        self.credential_lock
            .lock()
            .map_err(|_| BusinessError::unavailable("credential state is unavailable"))
    }
}

pub fn credential_store_error(error: SecretStoreError) -> BusinessError {
    let code = match error {
        SecretStoreError::Missing => "credential_missing",
        SecretStoreError::AccessDenied => "credential_access_denied",
        SecretStoreError::Locked => "credential_locked",
        SecretStoreError::Unavailable => "credential_store_unavailable",
        SecretStoreError::InvalidReference => "credential_invalid_reference",
        SecretStoreError::Backend(_) => "credential_unavailable",
    };
    BusinessError::new(code, error.to_string())
}

pub fn mask_secret(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= 12 {
        return "••••••••".into();
    }
    format!(
        "{}••••••••{}",
        chars[..4].iter().collect::<String>(),
        chars[chars.len() - 4..].iter().collect::<String>()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::connection::SimpleConnection;
    use std::sync::{Arc, Mutex};
    use suncode_secrets::{MemorySecretStore, SecretStore, SecretValue};

    #[derive(Default)]
    struct FailingStore {
        memory: MemorySecretStore,
        get_error: Mutex<Option<SecretStoreError>>,
        set_error: Mutex<Option<SecretStoreError>>,
        delete_error: Mutex<Option<SecretStoreError>>,
    }

    impl SecretStore for FailingStore {
        fn get(&self, id: &SecretId) -> Result<Option<SecretValue>, SecretStoreError> {
            if let Some(error) = self.get_error.lock().unwrap().clone() {
                return Err(error);
            }
            self.memory.get(id)
        }
        fn set(&self, id: &SecretId, value: &str) -> Result<(), SecretStoreError> {
            if let Some(error) = self.set_error.lock().unwrap().clone() {
                return Err(error);
            }
            self.memory.set(id, value)
        }
        fn delete(&self, id: &SecretId) -> Result<(), SecretStoreError> {
            if let Some(error) = self.delete_error.lock().unwrap().clone() {
                return Err(error);
            }
            self.memory.delete(id)
        }
    }

    fn fail_database_updates(store: &Store) {
        store.connection.lock().unwrap().batch_execute(
            "CREATE TRIGGER reject_credential_update BEFORE UPDATE ON llm_model_provider BEGIN SELECT RAISE(FAIL, 'injected write failure'); END;"
        ).unwrap();
    }

    #[test]
    fn keychain_write_failure_preserves_the_existing_key() {
        let backend = Arc::new(FailingStore::default());
        let store = Store::open_memory()
            .unwrap()
            .with_secret_store(backend.clone());
        store
            .save_provider_credential("openai", "fake-original-key")
            .unwrap();
        let before = store.provider_credential("openai").unwrap().unwrap();
        *backend.set_error.lock().unwrap() = Some(SecretStoreError::AccessDenied);
        assert_eq!(
            store
                .save_provider_credential("openai", "fake-replacement-key")
                .unwrap_err()
                .code,
            "credential_access_denied"
        );
        assert!(store.provider_credential("openai").unwrap().unwrap() == before);
        assert_eq!(
            store
                .resolve_provider_credential("openai")
                .unwrap()
                .as_deref(),
            Some("fake-original-key")
        );
    }

    #[test]
    fn unavailable_store_never_creates_a_database_credential() {
        let backend = Arc::new(FailingStore::default());
        let store = Store::open_memory()
            .unwrap()
            .with_secret_store(backend.clone());
        *backend.set_error.lock().unwrap() = Some(SecretStoreError::Unavailable);
        assert_eq!(
            store
                .save_provider_credential("openai", "fake-secret-value")
                .unwrap_err()
                .code,
            "credential_store_unavailable"
        );
        let record = store.provider_credential("openai").unwrap().unwrap();
        assert!(record.secret_ref.is_none());
        assert!(record.mask.is_none());
        assert_eq!(
            store.provider_credential_status("openai").unwrap(),
            "missing"
        );
    }

    #[test]
    fn failed_sqlite_update_restores_replaced_keychain_value() {
        let store = Store::open_memory().unwrap();
        store
            .save_provider_credential("openai", "fake-original-key")
            .unwrap();
        fail_database_updates(&store);
        assert!(store
            .save_provider_credential("openai", "fake-replacement-key")
            .is_err());
        assert_eq!(
            store
                .resolve_provider_credential("openai")
                .unwrap()
                .as_deref(),
            Some("fake-original-key")
        );
        assert_eq!(
            store
                .provider_credential("openai")
                .unwrap()
                .unwrap()
                .mask
                .as_deref(),
            Some("fake••••••••-key")
        );
    }

    #[test]
    fn failed_keychain_delete_preserves_storage_metadata() {
        let backend = Arc::new(FailingStore::default());
        let store = Store::open_memory()
            .unwrap()
            .with_secret_store(backend.clone());
        store
            .save_provider_credential("openai", "fake-original-key")
            .unwrap();
        let before = store.provider_credential("openai").unwrap().unwrap();
        *backend.delete_error.lock().unwrap() = Some(SecretStoreError::AccessDenied);
        assert_eq!(
            store.remove_provider_credential("openai").unwrap_err().code,
            "credential_access_denied"
        );
        assert!(store.provider_credential("openai").unwrap().unwrap() == before);
        assert_eq!(
            store
                .resolve_provider_credential("openai")
                .unwrap()
                .as_deref(),
            Some("fake-original-key")
        );
    }

    #[test]
    fn failed_metadata_clear_restores_removed_keychain_entry() {
        let store = Store::open_memory().unwrap();
        store
            .save_provider_credential("openai", "fake-original-key")
            .unwrap();
        fail_database_updates(&store);
        assert!(store.remove_provider_credential("openai").is_err());
        assert_eq!(
            store
                .resolve_provider_credential("openai")
                .unwrap()
                .as_deref(),
            Some("fake-original-key")
        );
    }

    #[test]
    fn access_failure_states_preserve_configured_status_mask_and_error_codes() {
        let backend = Arc::new(FailingStore::default());
        let store = Store::open_memory()
            .unwrap()
            .with_secret_store(backend.clone());
        store
            .save_provider_credential("openai", "fake-original-key")
            .unwrap();
        for (error, code) in [
            (SecretStoreError::Locked, "credential_locked"),
            (SecretStoreError::AccessDenied, "credential_access_denied"),
            (
                SecretStoreError::Unavailable,
                "credential_store_unavailable",
            ),
        ] {
            *backend.get_error.lock().unwrap() = Some(error);
            // Status reflects persisted configuration and must not probe the
            // keychain, which could trigger an OS credential prompt.
            assert_eq!(
                store.provider_credential_status("openai").unwrap(),
                "configured"
            );
            assert_eq!(
                store
                    .resolve_provider_credential("openai")
                    .unwrap_err()
                    .code,
                code
            );
            let record = store.provider_credential("openai").unwrap().unwrap();
            assert!(record.mask.is_some());
        }
    }

    #[test]
    fn disabled_provider_credentials_can_be_managed_and_short_keys_are_fully_masked() {
        let store = Store::open_memory().unwrap();
        store
            .connection
            .lock()
            .unwrap()
            .batch_execute("UPDATE llm_model_provider SET enabled=0 WHERE provider_id='openai'")
            .unwrap();
        store.save_provider_credential("openai", "tiny").unwrap();
        assert_eq!(
            store
                .provider_credential("openai")
                .unwrap()
                .unwrap()
                .mask
                .as_deref(),
            Some("••••••••")
        );
        store.remove_provider_credential("openai").unwrap();
        assert_eq!(
            store.provider_credential_status("openai").unwrap(),
            "missing"
        );
    }
}
