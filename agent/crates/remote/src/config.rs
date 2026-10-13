use serde::{Deserialize, Serialize};
use suncode_common::BusinessError;
use suncode_data::Store;
use url::Url;

use crate::RemoteResult;

pub(crate) const REMOTE_SERVER_URL: &str = "remote_server_url";
pub(crate) const REMOTE_PAIRING_CODE: &str = "remote_pairing_code";
pub(crate) const REMOTE_HOST_ID: &str = "remote_host_id";
pub(crate) const REMOTE_DESKTOP_TOKEN: &str = "remote_desktop_token";
pub(crate) const REMOTE_REFRESH_TOKEN: &str = "remote_refresh_token";
pub(crate) const REMOTE_ACCESS_TOKEN_EXPIRES_AT: &str = "remote_access_token_expires_at";
pub(crate) const REMOTE_MOBILE_PAIRING_CODE: &str = "remote_mobile_pairing_code";
pub(crate) const REMOTE_AES_KEY: &str = "remote_aes_key";
pub(crate) const REMOTE_E2E_ENABLED: &str = "remote_e2e_enabled";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RemoteServerConfiguration {
    pub server_url: String,
    pub pairing_code: String,
    #[serde(default = "default_e2e_enabled")]
    pub e2e_enabled: bool,
}

pub(crate) fn default_e2e_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RemoteServerStatus {
    pub configured: bool,
    pub connected: bool,
    pub connecting: bool,
    pub host_id: Option<String>,
    pub mobile_pairing_payload: Option<String>,
    pub access_token_expires_at: Option<String>,
    pub mobile_pairing_code: Option<String>,
    pub mobile_pairing_url: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PairRequest {
    pub(crate) pairing_code: String,
    pub(crate) display_name: String,
    pub(crate) agent_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PairResponse {
    pub(crate) host_id: String,
    pub(crate) access_token: String,
    pub(crate) refresh_token: String,
    pub(crate) access_token_expires_at: String,
    pub(crate) mobile_pairing_code: String,
    #[serde(skip)]
    pub(crate) aes_key: String,
}

impl PairResponse {
    pub(crate) fn mobile_pairing_url(&self, endpoint: &str, e2e_enabled: bool) -> String {
        let separator = if endpoint.contains('?') { '&' } else { '?' };
        format!(
            "{}{}code={}&hostId={}&k={}&e2e={}",
            endpoint.trim_end_matches('/'),
            separator,
            urlencoding(&self.mobile_pairing_code),
            urlencoding(&self.host_id),
            urlencoding(&self.aes_key),
            if e2e_enabled { "1" } else { "0" }
        )
    }
}

pub(crate) fn urlencoding(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

#[derive(Clone)]
pub(crate) struct PersistedRemote {
    pub(crate) config: RemoteServerConfiguration,
    pub(crate) pairing: PairResponse,
}

impl RemoteServerStatus {
    /// Disconnected status carrying the pairing details of `persisted`, if any.
    pub(crate) fn from_persisted(
        persisted: Option<&PersistedRemote>,
        configured: bool,
        connecting: bool,
    ) -> Self {
        let mobile_pairing_url = persisted.map(|value| {
            value
                .pairing
                .mobile_pairing_url(&value.config.server_url, value.config.e2e_enabled)
        });
        Self {
            configured,
            connected: false,
            connecting,
            host_id: persisted.map(|value| value.pairing.host_id.clone()),
            mobile_pairing_payload: mobile_pairing_url.clone(),
            access_token_expires_at: persisted
                .map(|value| value.pairing.access_token_expires_at.clone()),
            mobile_pairing_code: persisted.map(|value| value.pairing.mobile_pairing_code.clone()),
            mobile_pairing_url,
            error: None,
        }
    }
}

pub(crate) fn validate_remote_configuration(
    mut config: RemoteServerConfiguration,
) -> RemoteResult<RemoteServerConfiguration> {
    config.server_url = normalize_server_url(&config.server_url)?.to_string();
    config.pairing_code = config.pairing_code.trim().to_string();
    if config.pairing_code.is_empty() || config.pairing_code.len() > 256 {
        return Err(BusinessError::invalid("pairing code is required"));
    }
    Ok(config)
}

pub(crate) fn normalize_server_url(value: &str) -> RemoteResult<Url> {
    let url = Url::parse(value.trim())
        .map_err(|_| BusinessError::invalid("Remote Server URL is invalid"))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(BusinessError::invalid(
            "Remote Server URL must be an HTTP(S) origin without embedded credentials",
        ));
    }
    Ok(url)
}

pub(crate) fn hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "SunCode Desktop".into())
}

pub(crate) fn load_persisted(store: &Store) -> RemoteResult<Option<PersistedRemote>> {
    let values = store.settings(None, None)?;
    let get = |key: &str| {
        values
            .iter()
            .find(|entry| entry.key == key)
            .and_then(|entry| entry.value.as_str())
            .map(ToOwned::to_owned)
    };
    let (Some(server_url), Some(pairing_code)) = (get(REMOTE_SERVER_URL), get(REMOTE_PAIRING_CODE))
    else {
        return Ok(None);
    };
    let Some(host_id) = get(REMOTE_HOST_ID) else {
        return Ok(None);
    };
    let Some(desktop_token) = get(REMOTE_DESKTOP_TOKEN) else {
        return Ok(None);
    };
    let Some(refresh_token) = get(REMOTE_REFRESH_TOKEN) else {
        return Ok(None);
    };
    let Some(access_token_expires_at) = get(REMOTE_ACCESS_TOKEN_EXPIRES_AT) else {
        return Ok(None);
    };
    let Some(mobile_pairing_code) = get(REMOTE_MOBILE_PAIRING_CODE) else {
        return Ok(None);
    };
    let Some(aes_key) = get(REMOTE_AES_KEY) else {
        return Ok(None);
    };
    Ok(Some(PersistedRemote {
        config: RemoteServerConfiguration {
            server_url,
            pairing_code,
            e2e_enabled: values
                .iter()
                .find(|entry| entry.key == REMOTE_E2E_ENABLED)
                .and_then(|entry| entry.value.as_bool())
                .unwrap_or(true),
        },
        pairing: PairResponse {
            host_id,
            access_token: desktop_token,
            refresh_token,
            access_token_expires_at,
            mobile_pairing_code,
            aes_key,
        },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_url_rejects_embedded_credentials_and_non_http_schemes() {
        assert!(normalize_server_url("https://example.test").is_ok());
        assert!(normalize_server_url("https://user:password@example.test").is_err());
        assert!(normalize_server_url("file:///tmp").is_err());
    }

    #[test]
    fn remote_status_uses_camel_case_for_managed_clients() {
        let status = RemoteServerStatus {
            configured: true,
            connected: true,
            connecting: false,
            host_id: Some("host-1".into()),
            mobile_pairing_payload: Some("https://relay.example/pair".into()),
            access_token_expires_at: Some("2026-10-04T00:00:00Z".into()),
            mobile_pairing_code: Some("once".into()),
            mobile_pairing_url: Some("https://relay.example/pair".into()),
            error: None,
        };
        let value = serde_json::to_value(status).unwrap();
        assert_eq!(value["hostId"], "host-1");
        assert_eq!(value["mobilePairingUrl"], "https://relay.example/pair");
        assert!(value.get("host_id").is_none());
        assert!(value.get("mobile_pairing_url").is_none());
    }
}
