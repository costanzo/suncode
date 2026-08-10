use crate::{
    credentials::{CredentialStore, ProviderKind},
    domain::Message,
    llm::{Completion, CompletionFuture, LlmProvider, ProviderError},
    tools,
};
use futures_util::StreamExt;
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::normalize::{cancelled, wire_message};
use super::stream::SseParser;

#[derive(Clone)]
pub struct OpenAiCompatibleProvider {
    client: reqwest::Client,
    provider_kind: ProviderKind,
    provider_label: &'static str,
    endpoint: String,
    wire_model: String,
    credentials: CredentialStore,
}

impl OpenAiCompatibleProvider {
    pub fn new(
        provider_kind: ProviderKind,
        provider_label: &'static str,
        endpoint: String,
        wire_model: String,
        credentials: CredentialStore,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            provider_kind,
            provider_label,
            endpoint,
            wire_model,
            credentials,
        }
    }

    async fn complete_inner(
        &self,
        messages: &[Message],
        cancellation: &CancellationToken,
        deltas: mpsc::UnboundedSender<String>,
    ) -> Result<Completion, ProviderError> {
        let key = self
            .credentials
            .api_key(self.provider_kind)
            .ok_or_else(|| ProviderError {
                code: "provider_unconfigured",
                message: format!("{} API key is not configured", self.provider_label),
                retryable: false,
            })?;
        let body = json!({
            "model": self.wire_model,
            "messages": messages.iter().map(wire_message).collect::<Vec<_>>(),
            "tools": tools::definitions(),
            "stream": true,
            "stream_options": {"include_usage": true}
        });
        let response = tokio::select! {
            _ = cancellation.cancelled() => return Err(cancelled()),
            value = self.client.post(format!("{}/chat/completions", self.endpoint)).bearer_auth(key).json(&body).send() => value.map_err(|error| ProviderError { code: "transient", message: format!("{} request failed: {error}", self.provider_label), retryable: true })?,
        };
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
                },
                message,
                retryable: status.as_u16() == 408
                    || status.as_u16() == 429
                    || status.is_server_error(),
            });
        }
        let mut parser = SseParser::new(self.provider_label);
        let mut stream = response.bytes_stream();
        while let Some(chunk) = tokio::select! { _ = cancellation.cancelled() => return Err(cancelled()), value = stream.next() => value }
        {
            for delta in parser.push(&chunk.map_err(|error| ProviderError {
                code: "provider_protocol",
                message: format!("{} stream failed: {error}", self.provider_label),
                retryable: true,
            })?)? {
                let _ = deltas.send(delta);
            }
        }
        for delta in parser.flush()? {
            let _ = deltas.send(delta);
        }
        parser.finish()
    }
}

impl LlmProvider for OpenAiCompatibleProvider {
    fn complete<'a>(
        &'a self,
        messages: &'a [Message],
        cancellation: &'a CancellationToken,
        deltas: mpsc::UnboundedSender<String>,
    ) -> CompletionFuture<'a> {
        Box::pin(self.complete_inner(messages, cancellation, deltas))
    }
}

#[cfg(test)]
mod tests {
    use super::OpenAiCompatibleProvider;
    use crate::{
        credentials::{CredentialStore, ProviderKind},
        domain::Message,
        llm::LlmProvider,
    };
    use axum::{
        body::Bytes,
        http::{header, HeaderMap},
        response::IntoResponse,
        routing::post,
        Router,
    };
    use serde_json::Value;
    use tokio::sync::mpsc;
    use tokio_util::sync::CancellationToken;

    async fn mock_chat(headers: HeaderMap, body: Bytes) -> impl IntoResponse {
        assert_eq!(
            headers
                .get(header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok()),
            Some("Bearer zhipu-test-key")
        );
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["model"], "glm-5.2");
        assert_eq!(body["stream"], true);
        assert_eq!(body["stream_options"]["include_usage"], true);
        let response = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"hello\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\" world\"},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":3,\"completion_tokens\":2,\"total_tokens\":5}}\n\n",
            "data: [DONE]\n\n"
        );
        ([(header::CONTENT_TYPE, "text/event-stream")], response)
    }

    #[tokio::test]
    async fn streams_openai_compatible_completion() {
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
            ProviderKind::Zhipu,
            "Zhipu GLM",
            format!("http://{address}"),
            "glm-5.2".into(),
            CredentialStore::memory(None, Some("zhipu-test-key"), None),
        );
        let (tx, mut rx) = mpsc::unbounded_channel();
        let completion = provider
            .complete(
                &[Message::text("user", "hi")],
                &CancellationToken::new(),
                tx,
            )
            .await
            .unwrap();
        let mut deltas = Vec::new();
        while let Ok(delta) = rx.try_recv() {
            deltas.push(delta);
        }
        assert_eq!(deltas, vec!["hello", " world"]);
        assert_eq!(completion.text, "hello world");
        assert_eq!(completion.finish_reason, "stop");
        assert_eq!(completion.usage.unwrap().total_tokens, 5);
        server.abort();
    }
}
