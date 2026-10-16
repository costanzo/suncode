use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::Serialize;
use serde_json::{json, Value};
use suncode_common::BusinessError;

use crate::{config::PersistedRemote, RemoteResult};

/// AES-256 key length.
const AES_KEY_BYTES: usize = 32;
/// AES-GCM standard nonce length. Each encryption uses a fresh random nonce.
const NONCE_BYTES: usize = 12;

pub(crate) fn generate_aes_key() -> String {
    URL_SAFE_NO_PAD.encode(suncode_common::random::bytes::<AES_KEY_BYTES>())
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
    if bytes.len() < NONCE_BYTES + 16 {
        return Err(BusinessError::invalid("encrypted payload is truncated"));
    }
    let key = URL_SAFE_NO_PAD
        .decode(key_text)
        .map_err(|_| BusinessError::invalid("remote encryption key is not valid base64url"))?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| BusinessError::invalid("remote encryption key is invalid"))?;
    let nonce = Nonce::from_slice(&bytes[..NONCE_BYTES]);
    let plaintext = cipher
        .decrypt(nonce, &bytes[NONCE_BYTES..])
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
    let nonce_bytes = suncode_common::random::bytes::<NONCE_BYTES>();
    let nonce = Nonce::from_slice(&nonce_bytes);
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

    #[test]
    fn keys_are_full_length_and_nonces_are_fresh_per_encryption() {
        let key = generate_aes_key();
        assert_eq!(URL_SAFE_NO_PAD.decode(&key).unwrap().len(), AES_KEY_BYTES);
        assert_ne!(key, generate_aes_key());
        let value = json!({"text": "same"});
        let nonce = |payload: &str| {
            let bytes = URL_SAFE_NO_PAD
                .decode(payload.strip_prefix("e2e-v1:").unwrap())
                .unwrap();
            bytes[..NONCE_BYTES].to_vec()
        };
        let first = encrypt_remote_payload(&value, &key).unwrap();
        let second = encrypt_remote_payload(&value, &key).unwrap();
        assert_ne!(nonce(&first), nonce(&second));
        assert_ne!(first, second);
        assert_eq!(decrypt_remote_payload(&second, &key).unwrap(), value);
    }

    #[test]
    fn tampered_payloads_fail_authentication() {
        let key = generate_aes_key();
        let payload = encrypt_remote_payload(&json!({"text": "hello"}), &key).unwrap();
        let mut bytes = URL_SAFE_NO_PAD
            .decode(payload.strip_prefix("e2e-v1:").unwrap())
            .unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        let tampered = format!("e2e-v1:{}", URL_SAFE_NO_PAD.encode(bytes));
        assert!(decrypt_remote_payload(&tampered, &key).is_err());
    }
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
