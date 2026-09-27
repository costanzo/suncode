use crate::{BusinessError, Completion, ToolCall, TransferProgressDelta, Usage};
use bytes::Bytes;
use futures_util::stream;
use serde_json::Value;
use std::collections::BTreeMap;
use tokio::sync::mpsc;

const REQUEST_CHUNK_BYTES: usize = 16 * 1024;

pub fn tracked_json_body(
    value: &Value,
    progress: mpsc::UnboundedSender<TransferProgressDelta>,
) -> Result<reqwest::Body, BusinessError> {
    let bytes = Bytes::from(serde_json::to_vec(value).map_err(|error| {
        BusinessError::new(
            "provider_protocol",
            format!("request serialization failed: {error}"),
        )
    })?);
    let chunks = (0..bytes.len())
        .step_by(REQUEST_CHUNK_BYTES)
        .map(|start| bytes.slice(start..(start + REQUEST_CHUNK_BYTES).min(bytes.len())))
        .collect::<Vec<_>>();
    let body = stream::iter(chunks.into_iter().map(move |chunk| {
        let _ = progress.send(TransferProgressDelta {
            uploaded_bytes: chunk.len() as u64,
            downloaded_bytes: 0,
        });
        Ok::<_, std::io::Error>(chunk)
    }));
    Ok(reqwest::Body::wrap_stream(body))
}

struct FunctionCallState {
    call_id: String,
    name: String,
    arguments: String,
}

pub struct ResponsesSseParser {
    provider_label: String,
    buffer: String,
    pending_event: String,
    text: String,
    calls: BTreeMap<u64, FunctionCallState>,
    finish_reason: String,
    usage: Option<Usage>,
    response_id: Option<String>,
    terminal_error: Option<BusinessError>,
}

impl ResponsesSseParser {
    pub fn new(provider_label: impl Into<String>) -> Self {
        Self {
            provider_label: provider_label.into(),
            buffer: String::new(),
            pending_event: String::new(),
            text: String::new(),
            calls: BTreeMap::new(),
            finish_reason: String::new(),
            usage: None,
            response_id: None,
            terminal_error: None,
        }
    }

    pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<String>, BusinessError> {
        self.buffer.push_str(&String::from_utf8_lossy(bytes));
        let mut deltas = Vec::new();
        while let Some(index) = self.buffer.find('\n') {
            let line = self.buffer[..index].trim_end_matches('\r').to_string();
            self.buffer.drain(..=index);
            if let Some(delta) = self.line(&line)? {
                deltas.push(delta);
            }
        }
        Ok(deltas)
    }

    pub fn flush(&mut self) -> Result<Vec<String>, BusinessError> {
        let final_line = std::mem::take(&mut self.buffer);
        if final_line.trim().is_empty() {
            return Ok(Vec::new());
        }
        Ok(self.line(final_line.trim())?.into_iter().collect())
    }

    fn line(&mut self, line: &str) -> Result<Option<String>, BusinessError> {
        if let Some(event) = line.strip_prefix("event:") {
            self.pending_event = event.trim().to_owned();
            return Ok(None);
        }
        let Some(data) = line.strip_prefix("data:") else {
            return Ok(None);
        };
        let data = data.trim();
        if data == "[DONE]" || data.is_empty() {
            return Ok(None);
        }
        let chunk: Value = serde_json::from_str(data).map_err(|_| {
            BusinessError::new(
                "provider_protocol",
                format!("{} returned malformed stream JSON", self.provider_label),
            )
        })?;
        let event_type = chunk
            .get("type")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| self.pending_event.clone());
        self.pending_event.clear();
        match event_type.as_str() {
            "response.created" => {
                self.response_id = chunk
                    .pointer("/response/id")
                    .and_then(Value::as_str)
                    .map(str::to_owned);
            }
            "response.output_text.delta" => {
                let delta = chunk
                    .get("delta")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                self.text.push_str(delta);
                return Ok((!delta.is_empty()).then(|| delta.to_owned()));
            }
            "response.output_item.added" => {
                self.merge_output_item(chunk.get("item"), chunk.get("output_index"))
            }
            "response.output_item.done" => {
                self.merge_output_item(chunk.get("item"), chunk.get("output_index"))
            }
            "response.function_call_arguments.delta" => {
                let index = chunk
                    .get("output_index")
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
                let call = self
                    .calls
                    .entry(index)
                    .or_insert_with(|| FunctionCallState {
                        call_id: chunk
                            .get("call_id")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .into(),
                        name: chunk
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .into(),
                        arguments: String::new(),
                    });
                if let Some(value) = chunk.get("delta").and_then(Value::as_str) {
                    call.arguments.push_str(value);
                }
            }
            "response.function_call_arguments.done" => {
                let index = chunk
                    .get("output_index")
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
                let call = self
                    .calls
                    .entry(index)
                    .or_insert_with(|| FunctionCallState {
                        call_id: chunk
                            .get("call_id")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .into(),
                        name: chunk
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .into(),
                        arguments: String::new(),
                    });
                if let Some(value) = chunk.get("arguments").and_then(Value::as_str) {
                    call.arguments = value.into();
                }
            }
            "response.completed" => {
                self.merge_response(chunk.get("response"));
                self.finish_reason = "stop".into();
            }
            "response.incomplete" => {
                self.merge_response(chunk.get("response"));
                self.finish_reason = "incomplete".into();
            }
            "response.failed" => {
                let message = chunk
                    .pointer("/response/error/message")
                    .and_then(Value::as_str)
                    .or_else(|| chunk.pointer("/error/message").and_then(Value::as_str))
                    .unwrap_or("provider response failed");
                self.terminal_error = Some(BusinessError::provider(
                    "provider_protocol",
                    message,
                    false,
                    None,
                ));
            }
            "error" => {
                let message = chunk
                    .pointer("/error/message")
                    .and_then(Value::as_str)
                    .or_else(|| chunk.get("message").and_then(Value::as_str))
                    .unwrap_or("provider stream failed");
                self.terminal_error = Some(BusinessError::provider(
                    "provider_protocol",
                    message,
                    false,
                    None,
                ));
            }
            _ => {}
        }
        Ok(None)
    }

    fn merge_output_item(&mut self, item: Option<&Value>, output_index: Option<&Value>) {
        let Some(item) = item else { return };
        if item.get("type").and_then(Value::as_str) != Some("function_call") {
            return;
        }
        let index = output_index.and_then(Value::as_u64).unwrap_or_else(|| {
            item.get("output_index")
                .and_then(Value::as_u64)
                .unwrap_or(self.calls.len() as u64)
        });
        let call = self
            .calls
            .entry(index)
            .or_insert_with(|| FunctionCallState {
                call_id: String::new(),
                name: String::new(),
                arguments: String::new(),
            });
        if let Some(value) = item.get("call_id").and_then(Value::as_str) {
            call.call_id = value.into();
        }
        if let Some(value) = item.get("name").and_then(Value::as_str) {
            call.name = value.into();
        }
        if let Some(value) = item.get("arguments").and_then(Value::as_str) {
            call.arguments = value.into();
        }
    }

    fn merge_response(&mut self, response: Option<&Value>) {
        let Some(response) = response else { return };
        if self.response_id.is_none() {
            self.response_id = response
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_owned);
        }
        if let Some(usage) = response.get("usage") {
            self.usage = parse_usage(usage);
        }
        if let Some(output) = response.get("output").and_then(Value::as_array) {
            for (index, item) in output.iter().enumerate() {
                if self.text.is_empty()
                    && item.get("type").and_then(Value::as_str) == Some("message")
                {
                    if let Some(content) = item.get("content").and_then(Value::as_array) {
                        for part in content {
                            if let Some(text) = part.get("text").and_then(Value::as_str) {
                                self.text.push_str(text);
                            }
                        }
                    }
                }
                self.merge_output_item(Some(item), Some(&Value::from(index as u64)));
            }
        }
    }

    pub fn finish(self) -> Result<Completion, BusinessError> {
        if let Some(error) = self.terminal_error {
            return Err(error);
        }
        let tool_calls = self
            .calls
            .into_values()
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
            finish_reason: if self.finish_reason.is_empty() {
                "completed".into()
            } else {
                self.finish_reason
            },
            usage: self.usage,
            provider_request_id: None,
            provider_response_id: self.response_id,
        })
    }
}

fn parse_usage(usage: &Value) -> Option<Usage> {
    Some(Usage {
        input_tokens: usage
            .get("input_tokens")
            .or_else(|| usage.get("prompt_tokens"))
            .and_then(Value::as_u64)
            .unwrap_or(0),
        output_tokens: usage
            .get("output_tokens")
            .or_else(|| usage.get("completion_tokens"))
            .and_then(Value::as_u64)
            .unwrap_or(0),
        total_tokens: usage
            .get("total_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        cache_read_tokens: usage
            .pointer("/input_tokens_details/cached_tokens")
            .or_else(|| usage.pointer("/prompt_tokens_details/cached_tokens"))
            .or_else(|| usage.get("cached_tokens"))
            .or_else(|| usage.get("prompt_cache_hit_tokens"))
            .or_else(|| usage.get("cache_read_tokens"))
            .and_then(Value::as_u64),
        cache_miss_tokens: usage
            .pointer("/input_tokens_details/cache_miss_tokens")
            .or_else(|| usage.pointer("/prompt_tokens_details/cache_miss_tokens"))
            .or_else(|| usage.get("prompt_cache_miss_tokens"))
            .or_else(|| usage.get("cache_miss_tokens"))
            .and_then(Value::as_u64),
        cache_write_tokens: usage
            .get("cache_creation_input_tokens")
            .or_else(|| usage.get("cache_write_tokens"))
            .and_then(Value::as_u64),
        reasoning_tokens: usage
            .pointer("/output_tokens_details/reasoning_tokens")
            .or_else(|| usage.pointer("/completion_tokens_details/reasoning_tokens"))
            .or_else(|| usage.get("reasoning_tokens"))
            .and_then(Value::as_u64),
    })
}

#[cfg(test)]
mod tests {
    use super::ResponsesSseParser;

    fn parse_usage(usage: &str) -> crate::Usage {
        let usage: serde_json::Value = serde_json::from_str(usage).unwrap();
        let mut parser = ResponsesSseParser::new("Test provider");
        parser
            .push(format!("data: {}\n\n", serde_json::json!({"type":"response.completed","response":{"id":"resp-test","usage":usage}})).as_bytes())
            .unwrap();
        parser.finish().unwrap().usage.unwrap()
    }

    #[test]
    fn normalizes_kimi_cache_and_reasoning_usage() {
        let usage = parse_usage(
            r#"{
                "prompt_tokens":86,
                "completion_tokens":99,
                "total_tokens":185,
                "cached_tokens":86,
                "completion_tokens_details":{"reasoning_tokens":72},
                "prompt_tokens_details":{"cached_tokens":86}
            }"#,
        );

        assert_eq!(usage.input_tokens, 86);
        assert_eq!(usage.output_tokens, 99);
        assert_eq!(usage.cache_read_tokens, Some(86));
        assert_eq!(usage.cache_miss_tokens, None);
        assert_eq!(usage.reasoning_tokens, Some(72));
    }

    #[test]
    fn accepts_sse_event_names_when_payload_omits_type() {
        let mut parser = ResponsesSseParser::new("Test provider");
        let deltas = parser
            .push(b"event: response.output_text.delta\ndata: {\"delta\":\"hello\"}\n\n")
            .unwrap();
        assert_eq!(deltas, vec!["hello"]);
        assert_eq!(parser.finish().unwrap().text, "hello");
    }

    #[test]
    fn normalizes_deepseek_cache_hit_and_miss_usage() {
        let usage = parse_usage(
            r#"{
                "prompt_tokens":10,
                "completion_tokens":121,
                "total_tokens":131,
                "prompt_tokens_details":{"cached_tokens":0},
                "completion_tokens_details":{"reasoning_tokens":109},
                "prompt_cache_hit_tokens":0,
                "prompt_cache_miss_tokens":10
            }"#,
        );

        assert_eq!(usage.cache_read_tokens, Some(0));
        assert_eq!(usage.cache_miss_tokens, Some(10));
        assert_eq!(usage.cache_write_tokens, None);
        assert_eq!(usage.reasoning_tokens, Some(109));
    }

    #[test]
    fn accepts_top_level_cached_tokens_when_details_are_absent() {
        let usage = parse_usage(
            r#"{
                "prompt_tokens":86,
                "completion_tokens":1,
                "total_tokens":87,
                "cached_tokens":86
            }"#,
        );

        assert_eq!(usage.cache_read_tokens, Some(86));
    }
}
