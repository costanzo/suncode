use crate::{
    credentials::{CredentialStore, ProviderKind},
    domain::Message,
    llm::{Completion, CompletionFuture, LlmProvider, ProviderError},
    model_provider::normalize::{cancelled, wire_message},
    tools,
};
use futures_util::StreamExt;
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::model_provider::stream::SseParser;

#[derive(Clone)]
pub struct DeepSeekProvider {
    client: reqwest::Client,
    endpoint: String,
    wire_model: String,
    credentials: CredentialStore,
}

impl DeepSeekProvider {
    pub fn new(endpoint: String, wire_model: String, credentials: CredentialStore) -> Self {
        Self {
            client: reqwest::Client::new(),
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
            .api_key(ProviderKind::DeepSeek)
            .ok_or_else(|| ProviderError {
                code: "provider_unconfigured",
                message: "DeepSeek API key is not configured".into(),
                retryable: false,
            })?;
        let body = json!({"model": self.wire_model, "messages": messages.iter().map(wire_message).collect::<Vec<_>>(), "tools": tools::definitions(), "stream": true, "stream_options": {"include_usage": true}});
        let response = tokio::select! {
            _ = cancellation.cancelled() => return Err(cancelled()),
            value = self.client.post(format!("{}/chat/completions", self.endpoint)).bearer_auth(key).json(&body).send() => value.map_err(|error| ProviderError { code: "transient", message: format!("DeepSeek request failed: {error}"), retryable: true })?,
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
                .unwrap_or_else(|| format!("DeepSeek request failed with status {status}"));
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
        let mut parser = SseParser::new("DeepSeek");
        let mut stream = response.bytes_stream();
        while let Some(chunk) = tokio::select! { _ = cancellation.cancelled() => return Err(cancelled()), value = stream.next() => value }
        {
            for delta in parser.push(&chunk.map_err(|error| ProviderError {
                code: "provider_protocol",
                message: format!("DeepSeek stream failed: {error}"),
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

impl LlmProvider for DeepSeekProvider {
    fn complete<'a>(
        &'a self,
        messages: &'a [Message],
        cancellation: &'a CancellationToken,
        deltas: mpsc::UnboundedSender<String>,
    ) -> CompletionFuture<'a> {
        Box::pin(self.complete_inner(messages, cancellation, deltas))
    }
}
