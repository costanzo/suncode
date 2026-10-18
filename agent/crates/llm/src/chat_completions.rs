use crate::{
    http::{read_error_response, request_id, HttpClientFactory},
    normalize::wire_message,
    stream::tracked_json_body,
    ApiKeyResolver, BusinessError, Completion, CompletionFuture, CompletionRequest, LlmProvider,
    ToolCall, Usage,
};
use futures_util::StreamExt;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::{atomic::AtomicBool, Arc, RwLock};
use suncode_common::HttpProxyConfiguration;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct ChatCompletionsProvider {
    provider_id: String,
    provider_label: String,
    endpoint: String,
    keys: Arc<dyn ApiKeyResolver>,
    http: HttpClientFactory,
}

impl ChatCompletionsProvider {
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
                    "{} does not support native client toolsets through the Chat Completions adapter",
                    self.provider_label
                ),
                false,
                None,
            ));
        }
        let key = self.keys.api_key(&self.provider_id)?.ok_or_else(|| {
            BusinessError::new(
                "provider_unconfigured",
                format!("{} API key is not configured", self.provider_label),
            )
        })?;
        let messages = request
            .messages
            .iter()
            .map(wire_message)
            .collect::<Vec<_>>();
        let tools = request
            .tools
            .iter()
            .map(|tool| {
                json!({
                    "type": "function",
                    "function": {
                        "name": tool.name,
                        "description": tool.description,
                        "parameters": tool.parameters,
                    }
                })
            })
            .collect::<Vec<_>>();
        let mut body = json!({
            "model": request.wire_model,
            "messages": messages,
            "stream": true,
            "stream_options": {"include_usage": true},
        });
        if !tools.is_empty() {
            body["tools"] = Value::Array(tools);
        }
        // Tencent Cloud's public Hy4 route uses the model's default reasoning
        // policy and does not document a Chat Completions override.
        if let Some(max_output_tokens) = request.max_output_tokens {
            body["max_tokens"] = json!(max_output_tokens);
        }
        let transfer_progress = request.transfer_progress.clone();
        let request_body = tracked_json_body(&body, transfer_progress.clone())?;
        let client = self.http.client()?;
        let response = tokio::select! {
            _ = cancellation.cancelled() => return Err(crate::normalize::cancelled()),
            value = client.post(format!("{}/chat/completions", self.endpoint))
                .bearer_auth(key)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(request_body)
                .send() => value.map_err(|error| crate::errors::transport_error(error, &self.provider_label, None, false))?,
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
        let mut parser = ChatCompletionsSseParser::new(self.provider_label.clone());
        let mut stream = response.bytes_stream();
        while let Some(chunk) = tokio::select! {
            _ = cancellation.cancelled() => return Err(crate::normalize::cancelled()),
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
            let parsed = parser.push(&chunk)?;
            for delta in parsed {
                let _ = deltas.send(delta);
            }
        }
        let mut completion = parser.finish()?;
        completion.provider_request_id = provider_request_id;
        Ok(completion)
    }
}

impl LlmProvider for ChatCompletionsProvider {
    fn complete<'a>(
        &'a self,
        request: CompletionRequest<'a>,
        cancellation: &'a CancellationToken,
        deltas: mpsc::UnboundedSender<String>,
    ) -> CompletionFuture<'a> {
        Box::pin(self.complete_inner(request, cancellation, deltas))
    }
}

struct ToolCallState {
    call_id: String,
    name: String,
    arguments: String,
}

struct ChatCompletionsSseParser {
    provider_label: String,
    buffer: Vec<u8>,
    text: String,
    calls: Vec<ToolCallState>,
    finish_reason: Option<String>,
    usage: Option<Usage>,
    response_id: Option<String>,
}

impl ChatCompletionsSseParser {
    fn new(provider_label: String) -> Self {
        Self {
            provider_label,
            buffer: Vec::new(),
            text: String::new(),
            calls: Vec::new(),
            finish_reason: None,
            usage: None,
            response_id: None,
        }
    }

    fn response_started(&self) -> bool {
        !self.text.is_empty() || !self.calls.is_empty()
    }

    fn push(&mut self, bytes: &[u8]) -> Result<Vec<String>, BusinessError> {
        self.buffer.extend_from_slice(bytes);
        let mut deltas = Vec::new();
        let mut consumed = 0;
        while let Some(offset) = self.buffer[consumed..]
            .iter()
            .position(|byte| *byte == b'\n')
        {
            let end = consumed + offset;
            let line = String::from_utf8_lossy(&self.buffer[consumed..end])
                .trim()
                .to_string();
            consumed = end + 1;
            if let Some(delta) = self.line(&line)? {
                deltas.push(delta);
            }
        }
        self.buffer.drain(..consumed);
        Ok(deltas)
    }

    fn line(&mut self, line: &str) -> Result<Option<String>, BusinessError> {
        let Some(data) = line.strip_prefix("data:").map(str::trim) else {
            return Ok(None);
        };
        if data.is_empty() || data == "[DONE]" {
            return Ok(None);
        }
        let chunk: Value = serde_json::from_str(data)
            .map_err(|_| self.protocol("returned malformed stream JSON"))?;
        if self.response_id.is_none() {
            self.response_id = chunk.get("id").and_then(Value::as_str).map(str::to_owned);
        }
        if let Some(usage) = chunk.get("usage") {
            self.usage = parse_usage(usage);
        }
        let choice = chunk
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first());
        let Some(choice) = choice else {
            return Ok(None);
        };
        if let Some(reason) = choice.get("finish_reason").and_then(Value::as_str) {
            self.finish_reason = Some(reason.to_owned());
        }
        let Some(delta) = choice.get("delta") else {
            return Ok(None);
        };
        if let Some(value) = delta.get("content").and_then(Value::as_str) {
            self.text.push_str(value);
            return Ok((!value.is_empty()).then(|| value.to_owned()));
        }
        if let Some(calls) = delta.get("tool_calls").and_then(Value::as_array) {
            for call in calls {
                let index = call
                    .get("index")
                    .and_then(Value::as_u64)
                    .unwrap_or(self.calls.len() as u64) as usize;
                while self.calls.len() <= index {
                    self.calls.push(ToolCallState {
                        call_id: String::new(),
                        name: String::new(),
                        arguments: String::new(),
                    });
                }
                let state = &mut self.calls[index];
                if let Some(id) = call.get("id").and_then(Value::as_str) {
                    state.call_id = id.to_owned();
                }
                if let Some(function) = call.get("function") {
                    if let Some(name) = function.get("name").and_then(Value::as_str) {
                        state.name.push_str(name);
                    }
                    if let Some(arguments) = function.get("arguments").and_then(Value::as_str) {
                        state.arguments.push_str(arguments);
                    }
                }
            }
        }
        Ok(None)
    }

    fn finish(self) -> Result<Completion, BusinessError> {
        let response_started = self.response_started();
        let response_id = self.response_id.clone();
        let finish_reason = self.finish_reason.clone().ok_or_else(|| {
            BusinessError::provider(
                "provider_protocol",
                format!(
                    "{} stream ended before a finish reason",
                    self.provider_label
                ),
                !response_started,
                response_id,
            )
        })?;
        let tool_calls = self
            .calls
            .into_iter()
            .map(|call| {
                Ok(ToolCall {
                    call_id: call.call_id,
                    name: call.name,
                    arguments: serde_json::from_str(&call.arguments).map_err(|_| {
                        BusinessError::new(
                            "malformed_tool_call",
                            "Provider returned invalid tool arguments",
                        )
                    })?,
                    toolset_name: None,
                })
            })
            .collect::<Result<Vec<_>, BusinessError>>()?;
        Ok(Completion {
            text: self.text,
            tool_calls,
            finish_reason,
            usage: self.usage,
            provider_request_id: None,
            provider_response_id: self.response_id,
        })
    }

    fn protocol(&self, message: &str) -> BusinessError {
        BusinessError::provider(
            "provider_protocol",
            format!("{} {}", self.provider_label, message),
            false,
            self.response_id.clone(),
        )
    }
}

fn parse_usage(value: &Value) -> Option<Usage> {
    let input_tokens = value
        .get("prompt_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let output_tokens = value
        .get("completion_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    Some(Usage {
        input_tokens,
        output_tokens,
        total_tokens: value
            .get("total_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(input_tokens.saturating_add(output_tokens)),
        cache_read_tokens: value
            .pointer("/prompt_tokens_details/cached_tokens")
            .and_then(Value::as_u64),
        cache_miss_tokens: None,
        cache_write_tokens: None,
        reasoning_tokens: value
            .pointer("/completion_tokens_details/reasoning_tokens")
            .and_then(Value::as_u64),
    })
}

#[cfg(test)]
mod tests {
    use super::{ChatCompletionsProvider, ChatCompletionsSseParser};
    use crate::{ApiKeyResolver, CompletionRequest, LlmProvider, Message, TransferProgressDelta};
    use axum::{body::Bytes, http::HeaderMap, response::IntoResponse, routing::post, Router};
    use serde_json::Value;
    use std::sync::{atomic::AtomicBool, Arc, RwLock};
    use suncode_common::HttpProxyConfiguration;
    use tokio::sync::mpsc;
    use tokio_util::sync::CancellationToken;

    struct TestKeys;
    impl ApiKeyResolver for TestKeys {
        fn api_key(
            &self,
            provider_id: &str,
        ) -> Result<Option<String>, suncode_common::BusinessError> {
            Ok((provider_id == "tencent").then(|| "tencent-test-key".into()))
        }
    }

    async fn mock_chat(headers: HeaderMap, body: Bytes) -> impl IntoResponse {
        assert_eq!(
            headers
                .get("authorization")
                .and_then(|value| value.to_str().ok()),
            Some("Bearer tencent-test-key")
        );
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["model"], "hy4-preview");
        assert_eq!(body["stream"], true);
        (
            [("content-type", "text/event-stream")],
            "data: {\"id\":\"chat-1\",\"choices\":[{\"delta\":{\"content\":\"ok\"},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":1,\"completion_tokens\":1,\"total_tokens\":2}}\n\ndata: [DONE]\n\n",
        )
    }

    #[tokio::test]
    async fn calls_tencent_chat_completions_endpoint() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(
                listener,
                Router::new().route("/v1/chat/completions", post(mock_chat)),
            )
            .await
            .unwrap();
        });
        let provider = ChatCompletionsProvider::new_with_network_configuration(
            "tencent",
            "Tencent Hunyuan",
            format!("http://{address}/v1"),
            Arc::new(TestKeys),
            Arc::new(AtomicBool::new(true)),
            Arc::new(AtomicBool::new(true)),
            Arc::new(RwLock::new(None)),
            Arc::new(RwLock::new(HttpProxyConfiguration::default())),
        );
        let (delta_sender, mut deltas) = mpsc::unbounded_channel();
        let (progress, _progress_receiver) = mpsc::unbounded_channel::<TransferProgressDelta>();
        let completion = provider
            .complete(
                CompletionRequest {
                    messages: &[Message::text("user", "hello")],
                    wire_model: "hy4-preview",
                    tools: &[],
                    client_toolsets: &[],
                    reasoning_effort: None,
                    max_output_tokens: Some(1024),
                    transfer_progress: progress,
                },
                &CancellationToken::new(),
                delta_sender,
            )
            .await
            .unwrap();
        assert_eq!(completion.text, "ok");
        assert_eq!(deltas.recv().await.as_deref(), Some("ok"));
        server.abort();
    }

    #[test]
    fn parses_text_tool_calls_and_usage_from_stream() {
        let mut parser = ChatCompletionsSseParser::new("Tencent Hunyuan".into());
        let stream = concat!(
            "data: {\"id\":\"chat-1\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":\"hello\"},\"finish_reason\":null}]}\n\n",
            "data: {\"id\":\"chat-1\",\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call-1\",\"function\":{\"name\":\"read\",\"arguments\":\"{\\\"path\\\":\\\"README.md\\\"}\"}}]},\"finish_reason\":null}]}\n\n",
            "data: {\"id\":\"chat-1\",\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"tool_calls\"}],\"usage\":{\"prompt_tokens\":3,\"completion_tokens\":2,\"total_tokens\":5}}\n\n",
            "data: [DONE]\n\n"
        );
        let deltas = parser.push(stream.as_bytes()).unwrap();
        let completion = parser.finish().unwrap();
        assert_eq!(deltas, ["hello"]);
        assert_eq!(completion.text, "hello");
        assert_eq!(completion.provider_response_id.as_deref(), Some("chat-1"));
        assert_eq!(completion.tool_calls[0].call_id, "call-1");
        assert_eq!(completion.tool_calls[0].name, "read");
        assert_eq!(completion.tool_calls[0].arguments["path"], "README.md");
        assert_eq!(completion.usage.unwrap().total_tokens, 5);
    }

    #[test]
    fn rejects_stream_without_finish_reason() {
        let mut parser = ChatCompletionsSseParser::new("Tencent Hunyuan".into());
        parser
            .push(b"data: {\"choices\":[{\"delta\":{\"content\":\"partial\"}}]}\n\n")
            .unwrap();
        let error = parser.finish().unwrap_err();
        assert_eq!(error.code, "provider_protocol");
        assert!(!error.retryable);
    }
}
