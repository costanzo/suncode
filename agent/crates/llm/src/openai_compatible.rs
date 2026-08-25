use crate::{
    normalize::{cancelled, wire_message},
    stream::SseParser,
    ApiKeyResolver, Completion, CompletionFuture, CompletionRequest, LlmProvider, ProviderError,
};
use futures_util::StreamExt;
use serde_json::{json, Value};
use std::sync::Arc;
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
    client: reqwest::Client,
    provider_id: String,
    provider_label: String,
    endpoint: String,
    keys: Arc<dyn ApiKeyResolver>,
}

impl OpenAiCompatibleProvider {
    pub fn new(
        provider_id: impl Into<String>,
        provider_label: impl Into<String>,
        endpoint: impl Into<String>,
        keys: Arc<dyn ApiKeyResolver>,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            provider_id: provider_id.into(),
            provider_label: provider_label.into(),
            endpoint: endpoint.into().trim_end_matches('/').to_string(),
            keys,
        }
    }

    async fn complete_inner(
        &self,
        request: CompletionRequest<'_>,
        cancellation: &CancellationToken,
        deltas: mpsc::UnboundedSender<String>,
    ) -> Result<Completion, ProviderError> {
        let key = self
            .keys
            .api_key(&self.provider_id)
            .ok_or_else(|| ProviderError {
                code: "provider_unconfigured".into(),
                message: format!("{} API key is not configured", self.provider_label),
                retryable: false,
                provider_request_id: None,
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
        let response = tokio::select! {
            _ = cancellation.cancelled() => return Err(cancelled()),
            value = self.client.post(format!("{}/chat/completions", self.endpoint)).bearer_auth(key).json(&body).send() => value.map_err(|error| ProviderError {
                code: "transient".into(),
                message: format!("{} request failed: {error}", self.provider_label),
                retryable: true,
                provider_request_id: None,
            })?,
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
            return Err(ProviderError {
                code: if status.as_u16() == 401 {
                    "authentication"
                } else if status.as_u16() == 408
                    || status.as_u16() == 429
                    || status.is_server_error()
                {
                    "transient"
                } else {
                    "invalid_request"
                }
                .into(),
                message,
                retryable: status.as_u16() == 408
                    || status.as_u16() == 429
                    || status.is_server_error(),
                provider_request_id,
            });
        }
        let mut parser = SseParser::new(self.provider_label.clone());
        let mut stream = response.bytes_stream();
        while let Some(chunk) = tokio::select! {
            _ = cancellation.cancelled() => {
                let mut error = cancelled();
                error.provider_request_id = provider_request_id.clone();
                return Err(error);
            },
            value = stream.next() => value
        } {
            let chunk = chunk.map_err(|error| ProviderError {
                code: "provider_protocol".into(),
                message: format!("{} stream failed: {error}", self.provider_label),
                retryable: true,
                provider_request_id: provider_request_id.clone(),
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
    use std::sync::Arc;
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
}
