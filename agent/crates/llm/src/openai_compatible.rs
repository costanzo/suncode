use crate::{
    normalize::{cancelled, wire_message},
    stream::SseParser,
    ApiKeyResolver, BusinessError, Completion, CompletionFuture, CompletionRequest, LlmProvider,
};
use futures_util::StreamExt;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

const REQUEST_ID_HEADERS: &[&str] = &[
    "x-request-id",
    "request-id",
    "x-amzn-requestid",
    "x-goog-request-id",
];

#[derive(Clone)]
pub struct OpenAiCompatibleProvider {
    insecure_client: Result<reqwest::Client, String>,
    provider_id: String,
    provider_label: String,
    endpoint: String,
    keys: Arc<dyn ApiKeyResolver>,
    verify_https_certificates: Arc<AtomicBool>,
    use_system_certificates: Arc<AtomicBool>,
    certificate_path: Arc<RwLock<Option<PathBuf>>>,
}

impl OpenAiCompatibleProvider {
    pub fn new(
        provider_id: impl Into<String>,
        provider_label: impl Into<String>,
        endpoint: impl Into<String>,
        keys: Arc<dyn ApiKeyResolver>,
    ) -> Self {
        Self::new_with_https_certificate_verification(
            provider_id,
            provider_label,
            endpoint,
            keys,
            Arc::new(AtomicBool::new(true)),
        )
    }

    pub fn new_with_https_certificate_verification(
        provider_id: impl Into<String>,
        provider_label: impl Into<String>,
        endpoint: impl Into<String>,
        keys: Arc<dyn ApiKeyResolver>,
        verify_https_certificates: Arc<AtomicBool>,
    ) -> Self {
        Self::new_with_tls_configuration(
            provider_id,
            provider_label,
            endpoint,
            keys,
            verify_https_certificates,
            Arc::new(AtomicBool::new(true)),
            Arc::new(RwLock::new(None)),
        )
    }

    pub fn new_with_tls_configuration(
        provider_id: impl Into<String>,
        provider_label: impl Into<String>,
        endpoint: impl Into<String>,
        keys: Arc<dyn ApiKeyResolver>,
        verify_https_certificates: Arc<AtomicBool>,
        use_system_certificates: Arc<AtomicBool>,
        certificate_path: Arc<RwLock<Option<PathBuf>>>,
    ) -> Self {
        let insecure_client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .build()
            .map_err(|error| error.to_string());
        Self {
            insecure_client,
            provider_id: provider_id.into(),
            provider_label: provider_label.into(),
            endpoint: endpoint.into().trim_end_matches('/').to_string(),
            keys,
            verify_https_certificates,
            use_system_certificates,
            certificate_path,
        }
    }

    fn client(&self) -> Result<reqwest::Client, BusinessError> {
        if !self.verify_https_certificates.load(Ordering::SeqCst) {
            return self.insecure_client.clone().map_err(|error| {
                BusinessError::provider(
                    "provider_client_unavailable",
                    format!(
                        "{} HTTPS client could not be created: {error}",
                        self.provider_label
                    ),
                    false,
                    None,
                )
            });
        }
        let mut builder = reqwest::Client::builder();
        if !self.use_system_certificates.load(Ordering::SeqCst) {
            builder = builder.tls_built_in_root_certs(false);
        }
        // A custom trust file is only meaningful when the system trust store
        // has explicitly been disabled. Ignore any stale persisted path in
        // the default system-certificates mode so a deleted file cannot break
        // provider calls or retries.
        if !self.use_system_certificates.load(Ordering::SeqCst) {
            if let Some(path) = self.certificate_path.read().ok().and_then(|p| p.clone()) {
                let bytes = std::fs::read(&path).map_err(|e| {
                    BusinessError::provider(
                        "certificate_unavailable",
                        format!("could not read certificate file: {e}"),
                        false,
                        None,
                    )
                })?;
                let cert = reqwest::Certificate::from_pem(&bytes)
                    .or_else(|_| reqwest::Certificate::from_der(&bytes))
                    .map_err(|e| {
                        BusinessError::provider(
                            "certificate_invalid",
                            format!("invalid certificate file: {e}"),
                            false,
                            None,
                        )
                    })?;
                builder = builder.add_root_certificate(cert);
            }
        }
        builder.build().map_err(|error| {
            BusinessError::provider(
                "provider_client_unavailable",
                error.to_string(),
                false,
                None,
            )
        })
    }

    async fn complete_inner(
        &self,
        request: CompletionRequest<'_>,
        cancellation: &CancellationToken,
        deltas: mpsc::UnboundedSender<String>,
    ) -> Result<Completion, BusinessError> {
        let key = self.keys.api_key(&self.provider_id).ok_or_else(|| {
            BusinessError::new(
                "provider_unconfigured",
                format!("{} API key is not configured", self.provider_label),
            )
        })?;
        let mut body = json!({
            "model": request.wire_model,
            "messages": request.messages.iter().map(wire_message).collect::<Vec<_>>(),
            "tools": request.tools.iter().map(|tool| json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.parameters,
                }
            })).collect::<Vec<_>>(),
            "stream": true,
            "stream_options": {"include_usage": true}
        });
        if let Some(reasoning_effort) = request.reasoning_effort {
            body["reasoning_effort"] = json!(reasoning_effort);
        }
        let client = self.client()?;
        let response = tokio::select! {
            _ = cancellation.cancelled() => return Err(cancelled()),
            value = client.post(format!("{}/chat/completions", self.endpoint)).bearer_auth(key).json(&body).send() => value.map_err(|error| BusinessError::provider(
                "transient",
                format!("{} request failed: {error}", self.provider_label),
                true,
                None,
            ))?,
        };
        let provider_request_id = REQUEST_ID_HEADERS.iter().find_map(|name| {
            response
                .headers()
                .get(*name)
                .and_then(|value| value.to_str().ok())
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
        });
        if !response.status().is_success() {
            let status = response.status();
            let message = response
                .json::<Value>()
                .await
                .ok()
                .and_then(|value| {
                    value
                        .pointer("/error/message")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                })
                .unwrap_or_else(|| {
                    format!(
                        "{} request failed with status {status}",
                        self.provider_label
                    )
                });
            let code = if status.as_u16() == 401 {
                "authentication"
            } else if status.as_u16() == 408 || status.as_u16() == 429 || status.is_server_error() {
                "transient"
            } else {
                "invalid_request"
            };
            return Err(BusinessError::provider(
                code,
                message,
                status.as_u16() == 408 || status.as_u16() == 429 || status.is_server_error(),
                provider_request_id,
            ));
        }
        let mut parser = SseParser::new(self.provider_label.clone());
        let mut stream = response.bytes_stream();
        while let Some(chunk) = tokio::select! {
            _ = cancellation.cancelled() => {
                let mut error = cancelled();
                error.provider_request_id = provider_request_id.clone();
                error.details["provider_request_id"] = serde_json::json!(error.provider_request_id);
                return Err(error);
            },
            value = stream.next() => value
        } {
            let chunk = chunk.map_err(|error| {
                BusinessError::provider(
                    "provider_protocol",
                    format!("{} stream failed: {error}", self.provider_label),
                    true,
                    provider_request_id.clone(),
                )
            })?;
            let parsed = parser.push(&chunk).map_err(|mut error| {
                error.provider_request_id = provider_request_id.clone();
                error
            })?;
            for delta in parsed {
                let _ = deltas.send(delta);
            }
        }
        for delta in parser.flush().map_err(|mut error| {
            error.provider_request_id = provider_request_id.clone();
            error
        })? {
            let _ = deltas.send(delta);
        }
        match parser.finish() {
            Ok(mut completion) => {
                completion.provider_request_id = provider_request_id;
                Ok(completion)
            }
            Err(mut error) => {
                error.provider_request_id = provider_request_id;
                Err(error)
            }
        }
    }
}

impl LlmProvider for OpenAiCompatibleProvider {
    fn complete<'a>(
        &'a self,
        request: CompletionRequest<'a>,
        cancellation: &'a CancellationToken,
        deltas: mpsc::UnboundedSender<String>,
    ) -> CompletionFuture<'a> {
        Box::pin(self.complete_inner(request, cancellation, deltas))
    }
}

#[cfg(test)]
mod tests {
    use super::OpenAiCompatibleProvider;
    use crate::{ApiKeyResolver, CompletionRequest, LlmProvider, Message, ToolDefinition};
    use axum::{
        body::Bytes,
        http::{header, HeaderMap},
        response::IntoResponse,
        routing::post,
        Router,
    };
    use serde_json::{json, Value};
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::{path::PathBuf, sync::RwLock};
    use tokio::sync::mpsc;
    use tokio_util::sync::CancellationToken;

    struct TestKeys;

    impl ApiKeyResolver for TestKeys {
        fn api_key(&self, provider_id: &str) -> Option<String> {
            (provider_id == "enterprise").then(|| "enterprise-test-key".into())
        }
    }

    async fn mock_chat(headers: HeaderMap, body: Bytes) -> impl IntoResponse {
        assert_eq!(
            headers
                .get(header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok()),
            Some("Bearer enterprise-test-key")
        );
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["model"], "company-model-v1");
        assert_eq!(body["reasoning_effort"], "high");
        assert_eq!(body["tools"][0]["function"]["name"], "read");
        let response = concat!(
            "data: {\"id\":\"chatcmpl-response-1\",\"choices\":[{\"delta\":{\"content\":\"hello\"}}]}\n\n",
            "data: {\"id\":\"chatcmpl-response-1\",\"choices\":[{\"delta\":{\"content\":\" world\"},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":3,\"completion_tokens\":2,\"total_tokens\":5,\"prompt_tokens_details\":{\"cached_tokens\":2}}}\n\n",
            "data: [DONE]\n\n"
        );
        (
            [
                (header::CONTENT_TYPE.as_str(), "text/event-stream"),
                ("x-request-id", "request-1"),
            ],
            response,
        )
    }

    #[tokio::test]
    async fn supports_a_custom_openai_compatible_endpoint() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(
                listener,
                Router::new().route("/chat/completions", post(mock_chat)),
            )
            .await
            .unwrap();
        });
        let provider = OpenAiCompatibleProvider::new(
            "enterprise",
            "Enterprise Gateway",
            format!("http://{address}"),
            Arc::new(TestKeys),
        );
        let messages = vec![Message::text("user", "hello")];
        let tools = vec![ToolDefinition {
            name: "read".into(),
            description: "Read a file".into(),
            parameters: json!({"type": "object"}),
        }];
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let result = provider
            .complete(
                CompletionRequest {
                    messages: &messages,
                    wire_model: "company-model-v1",
                    tools: &tools,
                    reasoning_effort: Some("high"),
                },
                &CancellationToken::new(),
                sender,
            )
            .await
            .unwrap();
        assert_eq!(result.text, "hello world");
        assert_eq!(result.provider_request_id.as_deref(), Some("request-1"));
        assert_eq!(
            result.provider_response_id.as_deref(),
            Some("chatcmpl-response-1")
        );
        let usage = result.usage.unwrap();
        assert_eq!(usage.total_tokens, 5);
        assert_eq!(usage.cache_read_tokens, Some(2));
        assert_eq!(usage.cache_miss_tokens, None);
        assert_eq!(usage.cache_write_tokens, None);
        assert_eq!(usage.reasoning_tokens, None);
        assert_eq!(receiver.recv().await.as_deref(), Some("hello"));
        server.abort();
    }

    #[test]
    fn selects_https_client_from_live_verification_policy() {
        let verify = Arc::new(AtomicBool::new(true));
        let provider = OpenAiCompatibleProvider::new_with_https_certificate_verification(
            "enterprise",
            "Enterprise Gateway",
            "https://example.test",
            Arc::new(TestKeys),
            verify.clone(),
        );

        assert!(provider.client().is_ok());
        verify.store(false, Ordering::SeqCst);
        assert!(provider.client().is_ok());
    }

    #[test]
    fn ignores_custom_certificate_path_when_system_certificates_are_enabled() {
        let verify = Arc::new(AtomicBool::new(true));
        let use_system = Arc::new(AtomicBool::new(true));
        let certificate_path = Arc::new(RwLock::new(Some(PathBuf::from(
            "/path/that/does/not/exist.pem",
        ))));
        let provider = OpenAiCompatibleProvider::new_with_tls_configuration(
            "enterprise",
            "Enterprise Gateway",
            "https://example.test",
            Arc::new(TestKeys),
            verify,
            use_system,
            certificate_path,
        );

        assert!(provider.client().is_ok());
    }
}
