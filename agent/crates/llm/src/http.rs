//! HTTP plumbing shared by the built-in provider adapters: client
//! construction and caching, proxy configuration, request-id extraction, and
//! non-2xx response handling.

use crate::{
    errors::{retry_after_ms, retryable_status},
    normalize::{cancelled, is_context_overflow},
    BusinessError, TransferProgressDelta,
};
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, RwLock,
};
use suncode_common::{logging, HttpProxyConfiguration};
use suncode_http::ApplyHttpSettings;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

/// Error bodies beyond this size are truncated before parsing.
pub(crate) const MAX_ERROR_BODY_BYTES: usize = 64 * 1024;

const REQUEST_ID_HEADERS: &[&str] = &[
    "request-id",
    "x-request-id",
    "x-amzn-requestid",
    "x-goog-request-id",
];

/// The effective inputs to client construction. Two equal keys always build
/// equivalent clients, so the cache only rebuilds when this changes.
#[derive(Clone, PartialEq, Eq)]
struct ClientKey {
    verify_certificates: bool,
    use_system_certificates: bool,
    certificate_path: Option<PathBuf>,
    proxy: HttpProxyConfiguration,
}

/// Live network settings plus a cached client built from them. The settings
/// are shared with the host and may change at runtime; every `client()` call
/// re-reads them and rebuilds only when they differ from the cached key.
#[derive(Clone)]
pub(crate) struct HttpClientFactory {
    verify_https_certificates: Arc<AtomicBool>,
    use_system_certificates: Arc<AtomicBool>,
    certificate_path: Arc<RwLock<Option<PathBuf>>>,
    proxy_configuration: Arc<RwLock<HttpProxyConfiguration>>,
    cache: Arc<Mutex<Option<(ClientKey, reqwest::Client)>>>,
    #[cfg(test)]
    builds: Arc<std::sync::atomic::AtomicUsize>,
}

impl HttpClientFactory {
    pub(crate) fn new(
        verify_https_certificates: Arc<AtomicBool>,
        use_system_certificates: Arc<AtomicBool>,
        certificate_path: Arc<RwLock<Option<PathBuf>>>,
        proxy_configuration: Arc<RwLock<HttpProxyConfiguration>>,
    ) -> Self {
        Self {
            verify_https_certificates,
            use_system_certificates,
            certificate_path,
            proxy_configuration,
            cache: Arc::new(Mutex::new(None)),
            #[cfg(test)]
            builds: Arc::default(),
        }
    }

    fn key(&self) -> ClientKey {
        let verify_certificates = self.verify_https_certificates.load(Ordering::SeqCst);
        let use_system_certificates = self.use_system_certificates.load(Ordering::SeqCst);
        // A custom trust file is only meaningful when the system trust store
        // has explicitly been disabled. Ignore any stale persisted path in
        // the default system-certificates mode so a deleted file cannot break
        // provider calls or retries.
        let certificate_path = if verify_certificates && !use_system_certificates {
            self.certificate_path.read().ok().and_then(|p| p.clone())
        } else {
            None
        };
        let proxy = self
            .proxy_configuration
            .read()
            .map(|configuration| configuration.clone())
            .unwrap_or_default();
        ClientKey {
            verify_certificates,
            use_system_certificates,
            certificate_path,
            proxy,
        }
    }

    /// Returns the cached client, rebuilding it if the settings changed.
    /// Build failures are not cached so a corrected setting takes effect on
    /// the next call.
    pub(crate) fn client(&self) -> Result<reqwest::Client, BusinessError> {
        let key = self.key();
        let mut cache = self.cache.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((cached_key, client)) = cache.as_ref() {
            if *cached_key == key {
                return Ok(client.clone());
            }
        }
        let client = build_client(&key)?;
        #[cfg(test)]
        self.builds.fetch_add(1, Ordering::SeqCst);
        *cache = Some((key, client.clone()));
        Ok(client)
    }
}

fn build_client(key: &ClientKey) -> Result<reqwest::Client, BusinessError> {
    let trust = suncode_http::trust_roots(
        key.verify_certificates,
        key.use_system_certificates,
        key.certificate_path.as_deref(),
    )
    .map_err(provider_setup_error)?;
    let proxy = suncode_http::proxy_route(&key.proxy).map_err(|error| {
        BusinessError::provider("provider_client_unavailable", error.message, false, None)
    })?;
    reqwest::Client::builder()
        .retry(reqwest::retry::never())
        .connect_timeout(crate::CONNECT_TIMEOUT)
        .read_timeout(crate::READ_TIMEOUT)
        .apply_trust(trust)
        .apply_proxy(proxy)
        .build()
        .map_err(|error| {
            BusinessError::provider(
                "provider_client_unavailable",
                error.to_string(),
                false,
                None,
            )
        })
}

/// Keeps the shared error code and message but marks it as a provider error.
fn provider_setup_error(error: BusinessError) -> BusinessError {
    BusinessError::provider(error.code, error.message, false, None)
}

/// Returns the first non-empty provider request id header.
pub(crate) fn request_id(headers: &reqwest::header::HeaderMap) -> Option<String> {
    REQUEST_ID_HEADERS.iter().find_map(|name| {
        headers
            .get(*name)
            .and_then(|value| value.to_str().ok())
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    })
}

/// Maps a non-2xx status to a stable error code and retryability.
pub(crate) fn classify_status(status: reqwest::StatusCode, message: &str) -> (&'static str, bool) {
    let retryable = retryable_status(status.as_u16());
    let code = if is_context_overflow(status.as_u16(), message) {
        "context_overflow"
    } else if status.as_u16() == 401 {
        "authentication"
    } else if retryable {
        "transient"
    } else {
        "invalid_request"
    };
    (code, retryable)
}

/// Drains a non-2xx response (up to `MAX_ERROR_BODY_BYTES`), logs the
/// failure, and converts it to a provider error carrying any `Retry-After`
/// hint. Returns the cancellation error if the turn is cancelled mid-read.
pub(crate) async fn read_error_response(
    response: reqwest::Response,
    provider_label: &str,
    wire_model: &str,
    provider_request_id: Option<String>,
    transfer_progress: &mpsc::UnboundedSender<TransferProgressDelta>,
    cancellation: &CancellationToken,
) -> BusinessError {
    let status = response.status();
    let retry_after_ms = retry_after_ms(
        response
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|value| value.to_str().ok()),
        std::time::SystemTime::now(),
    );
    let Some(body) =
        read_capped_body(response.bytes_stream(), transfer_progress, cancellation).await
    else {
        return cancelled();
    };
    logging::error(
        "llm",
        format!("{provider_label}/{wire_model} request failed with status {status}"),
    );
    let mut error = error_from_body(status, &body, provider_label, provider_request_id);
    error.details["retry_after_ms"] = serde_json::json!(retry_after_ms);
    error
}

/// Reads at most `MAX_ERROR_BODY_BYTES`, stopping at the cap or on the first
/// stream error. An error body is best-effort context, so neither condition
/// fails the call. Returns `None` if cancelled.
async fn read_capped_body<S, E>(
    mut stream: S,
    transfer_progress: &mpsc::UnboundedSender<TransferProgressDelta>,
    cancellation: &CancellationToken,
) -> Option<Vec<u8>>
where
    S: Stream<Item = Result<Bytes, E>> + Unpin,
{
    let mut body = Vec::new();
    while let Some(Ok(chunk)) = tokio::select! {
        _ = cancellation.cancelled() => return None,
        chunk = stream.next() => chunk,
    } {
        let _ = transfer_progress.send(TransferProgressDelta {
            uploaded_bytes: 0,
            downloaded_bytes: chunk.len() as u64,
        });
        let remaining = MAX_ERROR_BODY_BYTES - body.len();
        body.extend_from_slice(&chunk[..chunk.len().min(remaining)]);
        if body.len() >= MAX_ERROR_BODY_BYTES {
            break;
        }
    }
    Some(body)
}

fn error_from_body(
    status: reqwest::StatusCode,
    body: &[u8],
    provider_label: &str,
    provider_request_id: Option<String>,
) -> BusinessError {
    let message = serde_json::from_slice::<Value>(body)
        .ok()
        .and_then(|value| {
            value
                .pointer("/error/message")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .unwrap_or_else(|| format!("{provider_label} request failed with status {status}"));
    let (code, retryable) = classify_status(status, &message);
    BusinessError::provider(code, message, retryable, provider_request_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::StatusCode;
    use suncode_common::HttpProxyMode;

    fn factory() -> (
        HttpClientFactory,
        Arc<AtomicBool>,
        Arc<RwLock<HttpProxyConfiguration>>,
    ) {
        let verify = Arc::new(AtomicBool::new(true));
        let proxy = Arc::new(RwLock::new(HttpProxyConfiguration::default()));
        let factory = HttpClientFactory::new(
            verify.clone(),
            Arc::new(AtomicBool::new(true)),
            Arc::new(RwLock::new(None)),
            proxy.clone(),
        );
        (factory, verify, proxy)
    }

    fn cached_key(factory: &HttpClientFactory) -> Option<ClientKey> {
        factory
            .cache
            .lock()
            .unwrap()
            .as_ref()
            .map(|(key, _)| key.clone())
    }

    fn builds(factory: &HttpClientFactory) -> usize {
        factory.builds.load(Ordering::SeqCst)
    }

    #[test]
    fn reuses_client_until_settings_change() {
        let (factory, verify, proxy) = factory();
        factory.client().unwrap();
        factory.client().unwrap();
        // Clones share the cache, so a provider clone does not rebuild.
        factory.clone().client().unwrap();
        assert_eq!(builds(&factory), 1);

        verify.store(false, Ordering::SeqCst);
        factory.client().unwrap();
        assert_eq!(builds(&factory), 2);
        assert!(!cached_key(&factory).unwrap().verify_certificates);

        proxy.write().unwrap().mode = HttpProxyMode::NoProxy;
        factory.client().unwrap();
        factory.client().unwrap();
        assert_eq!(builds(&factory), 3);
        assert!(cached_key(&factory).unwrap().proxy.mode == HttpProxyMode::NoProxy);
    }

    #[test]
    fn does_not_cache_failed_builds() {
        let (factory, _, proxy) = factory();
        factory.client().unwrap();
        {
            let mut proxy = proxy.write().unwrap();
            proxy.mode = HttpProxyMode::Custom;
            proxy.url = "not a url".into();
        }
        let error = factory.client().unwrap_err();
        assert_eq!(error.code, "provider_client_unavailable");
        assert_eq!(error.message, "custom proxy URL is invalid");
        proxy.write().unwrap().mode = HttpProxyMode::System;
        factory.client().unwrap();
        assert_eq!(builds(&factory), 2);
    }

    #[test]
    fn classifies_statuses() {
        let cases = [
            (401, "nope", "authentication", false),
            (408, "slow", "transient", true),
            (429, "busy", "transient", true),
            (500, "boom", "transient", true),
            (503, "down", "transient", true),
            (501, "unsupported", "invalid_request", false),
            (400, "bad", "invalid_request", false),
            (403, "forbidden", "invalid_request", false),
            (413, "big", "context_overflow", false),
        ];
        for (status, message, code, retryable) in cases {
            let status = StatusCode::from_u16(status).unwrap();
            assert_eq!(
                classify_status(status, message),
                (code, retryable),
                "{status}"
            );
        }
    }

    #[test]
    fn extracts_error_message_or_falls_back_to_status() {
        let status = StatusCode::BAD_REQUEST;
        let error = error_from_body(
            status,
            br#"{"error":{"message":"bad tool"}}"#,
            "Gateway",
            Some("req_1".into()),
        );
        assert_eq!(error.code, "invalid_request");
        assert_eq!(error.message, "bad tool");
        assert_eq!(error.provider_request_id.as_deref(), Some("req_1"));

        let error = error_from_body(StatusCode::BAD_GATEWAY, b"<html>", "Gateway", None);
        assert_eq!(error.code, "transient");
        assert!(error.retryable);
        assert_eq!(
            error.message,
            "Gateway request failed with status 502 Bad Gateway"
        );
    }

    #[tokio::test]
    async fn truncates_error_body_at_cap() {
        let chunk = Bytes::from(vec![b'x'; 40 * 1024]);
        let chunks = (0..4).map(|_| Ok::<_, std::io::Error>(chunk.clone()));
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let body = read_capped_body(
            futures_util::stream::iter(chunks),
            &sender,
            &CancellationToken::new(),
        )
        .await
        .unwrap();
        assert_eq!(body.len(), MAX_ERROR_BODY_BYTES);
        // Reading stops at the cap; the remaining chunks are never pulled.
        let mut downloaded = 0;
        while let Ok(delta) = receiver.try_recv() {
            downloaded += delta.downloaded_bytes;
        }
        assert_eq!(downloaded, 80 * 1024);
    }

    #[tokio::test]
    async fn keeps_partial_error_body_on_stream_error() {
        let chunks = vec![
            Ok(Bytes::from_static(b"partial")),
            Err(std::io::Error::other("reset")),
            Ok(Bytes::from_static(b"ignored")),
        ];
        let (sender, _receiver) = mpsc::unbounded_channel();
        let body = read_capped_body(
            futures_util::stream::iter(chunks),
            &sender,
            &CancellationToken::new(),
        )
        .await;
        assert_eq!(body.unwrap(), b"partial");
    }

    #[tokio::test]
    async fn stops_reading_error_body_when_cancelled() {
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let (sender, _receiver) = mpsc::unbounded_channel();
        // A stream that never yields would hang without cancellation.
        let stream = futures_util::stream::pending::<Result<Bytes, std::io::Error>>();
        let body = read_capped_body(stream, &sender, &cancellation).await;
        assert!(body.is_none());
    }

    #[test]
    fn prefers_first_non_empty_request_id_header() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("request-id", "".parse().unwrap());
        headers.insert("x-request-id", "req_2".parse().unwrap());
        assert_eq!(request_id(&headers).as_deref(), Some("req_2"));
        assert_eq!(request_id(&reqwest::header::HeaderMap::new()), None);
    }
}
