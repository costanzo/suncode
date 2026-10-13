use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::Serialize;
use serde_json::{json, Value};
use suncode_common::BusinessError;

use crate::{config::PersistedRemote, RemoteResult};

pub(crate) fn generate_aes_key() -> String {
    let mut key = [0u8; 32];
    key[..16].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
    key[16..].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
    URL_SAFE_NO_PAD.encode(key)
}

/// Desktop failure body for `POST /v1/desktop/responses`. Successful results are
/// posted as the bare command payload (or `encPayload`), never wrapped.
#[derive(Debug, Serialize)]
pub(crate) struct RemoteErrorBody {
    pub(crate) code: i32,
    pub(crate) message: String,
}

pub(crate) const REMOTE_DESKTOP_ERROR_CODE: i32 = 50400;

pub(crate) fn decrypt_remote_payload(payload: &str, key_text: &str) -> RemoteResult<Value> {
    let encoded = payload
        .strip_prefix("e2e-v1:")
        .ok_or_else(|| BusinessError::invalid("unsupported encrypted payload version"))?;
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| BusinessError::invalid("encrypted payload is not valid base64url"))?;
    if bytes.len() < 12 + 16 {
        return Err(BusinessError::invalid("encrypted payload is truncated"));
    }
    let key = URL_SAFE_NO_PAD
        .decode(key_text)
        .map_err(|_| BusinessError::invalid("remote encryption key is not valid base64url"))?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| BusinessError::invalid("remote encryption key is invalid"))?;
    let nonce = Nonce::from_slice(&bytes[..12]);
    let plaintext = cipher
        .decrypt(nonce, &bytes[12..])
        .map_err(|_| BusinessError::invalid("encrypted payload authentication failed"))?;
    serde_json::from_slice(&plaintext)
        .map_err(|_| BusinessError::invalid("encrypted payload JSON is invalid"))
}

pub(crate) fn encrypt_remote_payload(value: &Value, key_text: &str) -> RemoteResult<String> {
    let key = URL_SAFE_NO_PAD
        .decode(key_text)
        .map_err(|_| BusinessError::invalid("remote encryption key is not valid base64url"))?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| BusinessError::invalid("remote encryption key is invalid"))?;
    let nonce_source = uuid::Uuid::new_v4();
    let nonce_bytes = &nonce_source.as_bytes()[..12];
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = serde_json::to_vec(value)
        .map_err(|_| BusinessError::invalid("remote payload JSON is invalid"))?;
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|_| BusinessError::unavailable("remote payload encryption failed"))?;
    let mut bytes = nonce_bytes.to_vec();
    bytes.extend_from_slice(&ciphertext);
    Ok(format!("e2e-v1:{}", URL_SAFE_NO_PAD.encode(bytes)))
}

pub(crate) fn remote_request_body(
    value: &Value,
    persisted: &PersistedRemote,
) -> RemoteResult<Value> {
    if !persisted.config.e2e_enabled {
        return Ok(value.clone());
    }
    Ok(json!({ "encPayload": encrypt_remote_payload(value, &persisted.pairing.aes_key)? }))
}

/// Builds the `/v1/desktop/responses` body: the plain payload (or its
/// `encPayload` wrapper) on success, and an unencrypted `{code, message}` on
/// failure so the Server can map it to an HTTP error for Mobile.
pub(crate) fn remote_response_body(
    outcome: RemoteResult<Value>,
    persisted: &PersistedRemote,
) -> RemoteResult<Value> {
    match outcome {
        Ok(data) => remote_request_body(&data, persisted),
        Err(error) => serde_json::to_value(RemoteErrorBody {
            code: REMOTE_DESKTOP_ERROR_CODE,
            message: error.message,
        })
        .map_err(|e| BusinessError::unavailable(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{PairResponse, RemoteServerConfiguration};

    fn persisted_fixture(e2e_enabled: bool) -> PersistedRemote {
        PersistedRemote {
            config: RemoteServerConfiguration {
                server_url: "https://example.test".into(),
                pairing_code: "pair".into(),
                e2e_enabled,
            },
            pairing: PairResponse {
                host_id: "host-1".into(),
                access_token: "access".into(),
                refresh_token: "refresh".into(),
                access_token_expires_at: "2026-10-04T00:00:00Z".into(),
                mobile_pairing_code: "once".into(),
                aes_key: generate_aes_key(),
            },
        }
    }

    #[test]
    fn response_body_posts_plain_success_payload_without_code_envelope() {
        let body = remote_response_body(
            Ok(json!({"projects": [{"projectId": "p-1"}]})),
            &persisted_fixture(false),
        )
        .unwrap();
        assert_eq!(body, json!({"projects": [{"projectId": "p-1"}]}));
        assert!(body.get("code").is_none());
        assert!(body.get("payload").is_none());
    }

    #[test]
    fn response_body_encrypts_only_success_payloads() {
        let persisted = persisted_fixture(true);
        let body = remote_response_body(Ok(json!({"ok": true})), &persisted).unwrap();
        assert_eq!(body.as_object().unwrap().len(), 1);
        assert!(body["encPayload"].is_string());

        let error = remote_response_body(
            Err(BusinessError::invalid("projectId is required")),
            &persisted,
        )
        .unwrap();
        assert_eq!(
            error,
            json!({"code": REMOTE_DESKTOP_ERROR_CODE, "message": "projectId is required"})
        );
    }
}
