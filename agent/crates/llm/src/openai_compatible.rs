use crate::{
    http::{read_error_response, request_id, HttpClientFactory},
    normalize::{cancelled, responses_input_items},
    stream::{tracked_json_body, ResponsesSseParser},
    ApiKeyResolver, BusinessError, Completion, CompletionFuture, CompletionRequest, LlmProvider,
};
use futures_util::StreamExt;
use serde_json::json;
use std::path::PathBuf;
use std::sync::{atomic::AtomicBool, Arc, RwLock};
use suncode_common::HttpProxyConfiguration;
use suncode_common::{self, logging};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct OpenAiCompatibleProvider {
    provider_id: String,
    provider_label: String,
    endpoint: String,
    keys: Arc<dyn ApiKeyResolver>,
    http: HttpClientFactory,
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
        Self::new_with_network_configuration(
            provider_id,
            provider_label,
            endpoint,
            keys,
            verify_https_certificates,
            use_system_certificates,
            certificate_path,
            Arc::new(RwLock::new(HttpProxyConfiguration::default())),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_network_configuration(
        provider_id: impl Into<String>,
        provider_label: impl Into<String>,
        endpoint: impl Into<String>,
        keys: Arc<dyn ApiKeyResolver>,
        verify_https_certificates: Arc<AtomicBool>,
        use_system_certificates: Arc<AtomicBool>,
        certificate_path: Arc<RwLock<Option<PathBuf>>>,
        proxy_configuration: Arc<RwLock<HttpProxyConfiguration>>,
    ) -> Self {
        Self {
            provider_id: provider_id.into(),
            provider_label: provider_label.into(),
            endpoint: endpoint.into().trim_end_matches('/').to_string(),
            keys,
            http: HttpClientFactory::new(
                verify_https_certificates,
                use_system_certificates,
                certificate_path,
                proxy_configuration,
            ),
        }
    }

    fn client(&self) -> Result<reqwest::Client, BusinessError> {
        self.http.client()
    }

    async fn complete_inner(
        &self,
        request: CompletionRequest<'_>,
        cancellation: &CancellationToken,
        deltas: mpsc::UnboundedSender<String>,
    ) -> Result<Completion, BusinessError> {
        if !request.client_toolsets.is_empty() {
            return Err(BusinessError::provider(
                "provider_capability_unsupported",
                format!(
                    "{} does not support native client toolsets through the OpenAI-compatible adapter",
                    self.provider_label
                ),
                false,
                None,
            ));
        }
        let key = self.keys.api_key(&self.provider_id).ok_or_else(|| {
            BusinessError::new(
                "provider_unconfigured",
                format!("{} API key is not configured", self.provider_label),
            )
        })?;
        let mut body = json!({
            "model": request.wire_model,
            "input": responses_input_items(request.messages),
            "tools": request.tools.iter().map(|tool| json!({
                "type": "function",
                "name": tool.name,
                "description": tool.description,
                "parameters": tool.parameters,
                "strict": false,
            })).collect::<Vec<_>>(),
            "stream": true,
            "store": false
        });
        if let Some(reasoning_effort) = request.reasoning_effort {
            body["reasoning"] = json!({"effort": reasoning_effort});
        }
        if let Some(max_output_tokens) = request.max_output_tokens {
            body["max_output_tokens"] = json!(max_output_tokens);
        }
        logging::debug(
            "llm",
            format!(
                "{}/{} request: messages={} tools={} client_toolsets={}",
                self.provider_label,
                request.wire_model,
                request.messages.len(),
                request.tools.len(),
                request.client_toolsets.len(),
            ),
        );
        let transfer_progress = request.transfer_progress.clone();
        let request_body = tracked_json_body(&body, transfer_progress.clone())?;
        let client = self.client()?;
        let response = tokio::select! {
            _ = cancellation.cancelled() => return Err(cancelled()),
            value = client.post(format!("{}/responses", self.endpoint)).bearer_auth(key).header(reqwest::header::CONTENT_TYPE, "application/json").body(request_body).send() => value.map_err(|error| crate::errors::transport_error(error, &self.provider_label, None, false))?,
        };
        let provider_request_id = request_id(response.headers());
        if !response.status().is_success() {
            return Err(read_error_response(
                response,
                &self.provider_label,
                request.wire_model,
                provider_request_id,
                &transfer_progress,
                cancellation,
            )
            .await);
        }
        let mut parser = ResponsesSseParser::new(self.provider_label.clone());
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
                crate::errors::transport_error(
                    error,
                    &self.provider_label,
                    provider_request_id.clone(),
                    parser.response_started(),
                )
            })?;
            let _ = transfer_progress.send(crate::TransferProgressDelta {
                uploaded_bytes: 0,
                downloaded_bytes: chunk.len() as u64,
            });
            let parsed = parser.push(&chunk).map_err(|mut error| {
                error.provider_request_id = provider_request_id.clone();
                error.details["response_started"] = parser.response_started().into();
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
    use crate::{
        ApiKeyResolver, CompletionRequest, LlmProvider, Message, ToolDefinition,
        TransferProgressDelta,
    };
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
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::sync::mpsc;
    use tokio_util::sync::CancellationToken;

    struct TestKeys;

    impl ApiKeyResolver for TestKeys {
        fn api_key(&self, provider_id: &str) -> Option<String> {
            (provider_id == "enterprise").then(|| "enterprise-test-key".into())
        }
    }

    async fn mock_responses(headers: HeaderMap, body: Bytes) -> impl IntoResponse {
        assert_eq!(
            headers
                .get(header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok()),
            Some("Bearer enterprise-test-key")
        );
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["model"], "company-model-v1");
        assert_eq!(body["reasoning"]["effort"], "high");
        assert_eq!(body["tools"][0]["name"], "read");
        assert_eq!(body["tools"][1]["name"], "computer_screenshot");
        let response = concat!(
            "data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp-response-1\"}}\n\n",
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"hello\"}\n\n",
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\" world\"}\n\n",
            "data: {\"type\":\"response.completed\",\"response\":{\"id\":\"resp-response-1\",\"status\":\"completed\",\"usage\":{\"input_tokens\":3,\"output_tokens\":2,\"total_tokens\":5,\"input_tokens_details\":{\"cached_tokens\":2}}}}\n\n"
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
    async fn connection_refusal_is_retryable_before_output() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        let provider = OpenAiCompatibleProvider::new(
            "enterprise",
            "Enterprise Gateway",
            format!("http://{address}"),
            Arc::new(TestKeys),
        );
        let (sender, _receiver) = mpsc::unbounded_channel();
        let (progress, _progress_receiver) = mpsc::unbounded_channel();
        let error = provider
            .complete(
                CompletionRequest {
                    messages: &[Message::text("user", "hello")],
                    wire_model: "company-model-v1",
                    tools: &[],
                    client_toolsets: &[],
                    reasoning_effort: None,
                    max_output_tokens: None,
                    transfer_progress: progress,
                },
                &CancellationToken::new(),
                sender,
            )
            .await
            .unwrap_err();
        assert_eq!(error.code, "transient");
        assert!(error.retryable);
        assert_eq!(error.details["response_started"], false);
    }
    #[tokio::test]
    async fn supports_a_custom_openai_compatible_endpoint() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(
                listener,
                Router::new().route("/responses", post(mock_responses)),
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
        let tools = vec![
            ToolDefinition {
                name: "read".into(),
                description: "Read a file".into(),
                parameters: json!({"type": "object"}),
            },
            ToolDefinition {
                name: "computer_screenshot".into(),
                description: "Capture the primary display".into(),
                parameters: json!({"type": "object", "properties": {}, "additionalProperties": false}),
            },
        ];
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let (transfer_sender, mut transfer_receiver) = mpsc::unbounded_channel();
        let result = provider
            .complete(
                CompletionRequest {
                    messages: &messages,
                    wire_model: "company-model-v1",
                    tools: &tools,
                    client_toolsets: &[],
                    reasoning_effort: Some("high"),
                    max_output_tokens: Some(4096),
                    transfer_progress: transfer_sender,
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
            Some("resp-response-1")
        );
        let usage = result.usage.unwrap();
        assert_eq!(usage.total_tokens, 5);
        assert_eq!(usage.cache_read_tokens, Some(2));
        assert_eq!(usage.cache_miss_tokens, None);
        assert_eq!(usage.cache_write_tokens, None);
        assert_eq!(usage.reasoning_tokens, None);
        assert_eq!(receiver.recv().await.as_deref(), Some("hello"));
        let mut uploaded = 0_u64;
        let mut downloaded = 0_u64;
        while let Ok(progress) = transfer_receiver.try_recv() {
            uploaded += progress.uploaded_bytes;
            downloaded += progress.downloaded_bytes;
        }
        assert!(uploaded > 0);
        assert!(downloaded > 0);
        server.abort();
    }

    #[tokio::test]
    async fn routes_provider_requests_through_the_custom_proxy() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = vec![0_u8; 16 * 1024];
            let read = stream.read(&mut request).await.unwrap();
            let request = String::from_utf8_lossy(&request[..read]);
            assert!(request.starts_with("POST http://provider.example.test/responses HTTP/1.1"));
            assert!(request.to_ascii_lowercase().contains(
                "proxy-authorization: basic cHJveHktdXNlcjpwcm94eS1wYXNz"
                    .to_ascii_lowercase()
                    .as_str()
            ));
            let body = concat!(
                "data: {\"type\":\"response.output_text.delta\",\"delta\":\"proxied\"}\n\n",
                "data: {\"type\":\"response.completed\",\"response\":{\"id\":\"resp-proxy\",\"status\":\"completed\"}}\n\n"
            );
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });
        let provider = OpenAiCompatibleProvider::new_with_network_configuration(
            "enterprise",
            "Enterprise Gateway",
            "http://provider.example.test",
            Arc::new(TestKeys),
            Arc::new(AtomicBool::new(true)),
            Arc::new(AtomicBool::new(true)),
            Arc::new(RwLock::new(None)),
            Arc::new(RwLock::new(suncode_common::HttpProxyConfiguration {
                mode: suncode_common::HttpProxyMode::Custom,
                url: format!("http://{address}"),
                username: "proxy-user".into(),
                password: "proxy-pass".into(),
                bypass: Vec::new(),
            })),
        );
        let messages = vec![Message::text("user", "hello")];
        let tools = Vec::<ToolDefinition>::new();
        let (sender, _receiver) = mpsc::unbounded_channel();
        let (transfer_sender, _transfer_receiver) =
            mpsc::unbounded_channel::<TransferProgressDelta>();
        let result = provider
            .complete(
                CompletionRequest {
                    messages: &messages,
                    wire_model: "company-model-v1",
                    tools: &tools,
                    client_toolsets: &[],
                    reasoning_effort: None,
                    max_output_tokens: None,
                    transfer_progress: transfer_sender,
                },
                &CancellationToken::new(),
                sender,
            )
            .await
            .unwrap();
        server.await.unwrap();
        assert_eq!(result.text, "proxied");
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
