use crate::{
    domain::{ToolCall, Usage},
    llm::{Completion, ProviderError},
};
use serde_json::Value;
use std::collections::BTreeMap;

pub struct SseParser {
    provider_label: &'static str,
    buffer: String,
    text: String,
    calls: BTreeMap<u64, (String, String, String)>,
    finish_reason: String,
    usage: Option<Usage>,
}

impl SseParser {
    pub fn new(provider_label: &'static str) -> Self {
        Self {
            provider_label,
            buffer: String::new(),
            text: String::new(),
            calls: BTreeMap::new(),
            finish_reason: String::new(),
            usage: None,
        }
    }

    pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<String>, ProviderError> {
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

    pub fn flush(&mut self) -> Result<Vec<String>, ProviderError> {
        let final_line = std::mem::take(&mut self.buffer);
        if final_line.trim().is_empty() {
            return Ok(Vec::new());
        }
        Ok(self.line(final_line.trim())?.into_iter().collect())
    }

    fn line(&mut self, line: &str) -> Result<Option<String>, ProviderError> {
        let Some(data) = line.strip_prefix("data:") else {
            return Ok(None);
        };
        let data = data.trim();
        if data == "[DONE]" || data.is_empty() {
            return Ok(None);
        }
        let chunk: Value = serde_json::from_str(data).map_err(|_| ProviderError {
            code: "provider_protocol",
            message: format!("{} returned malformed stream JSON", self.provider_label),
            retryable: false,
        })?;
        if let Some(usage) = chunk.get("usage") {
            self.usage = Some(Usage {
                input_tokens: usage
                    .get("prompt_tokens")
                    .or_else(|| usage.get("input_tokens"))
                    .and_then(Value::as_u64)
                    .unwrap_or(0),
                output_tokens: usage
                    .get("completion_tokens")
                    .or_else(|| usage.get("output_tokens"))
                    .and_then(Value::as_u64)
                    .unwrap_or(0),
                total_tokens: usage
                    .get("total_tokens")
                    .and_then(Value::as_u64)
                    .unwrap_or(0),
            });
        }
        if let Some(choice) = chunk
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|v| v.first())
        {
            let text_delta = choice
                .get("delta")
                .and_then(|v| v.get("content"))
                .and_then(Value::as_str);
            if let Some(value) = text_delta {
                self.text.push_str(value);
            }
            if let Some(calls) = choice
                .pointer("/delta/tool_calls")
                .and_then(Value::as_array)
            {
                for call in calls {
                    let index = call.get("index").and_then(Value::as_u64).unwrap_or(0);
                    let entry = self.calls.entry(index).or_default();
                    if let Some(v) = call.get("id").and_then(Value::as_str) {
                        entry.0 = v.into()
                    }
                    if let Some(v) = call.pointer("/function/name").and_then(Value::as_str) {
                        entry.1.push_str(v)
                    }
                    if let Some(v) = call.pointer("/function/arguments").and_then(Value::as_str) {
                        entry.2.push_str(v)
                    }
                }
            }
            if let Some(reason) = choice.get("finish_reason").and_then(Value::as_str) {
                self.finish_reason = reason.into()
            }
            return Ok(text_delta.map(str::to_string));
        }
        Ok(None)
    }

    pub fn finish(self) -> Result<Completion, ProviderError> {
        let tool_calls = self
            .calls
            .into_values()
            .map(|(id, name, args)| {
                Ok(ToolCall {
                    call_id: id,
                    name,
                    arguments: serde_json::from_str(&args).map_err(|_| ProviderError {
                        code: "malformed_tool_call",
                        message: "Provider returned invalid tool arguments".into(),
                        retryable: false,
                    })?,
                })
            })
            .collect::<Result<Vec<_>, ProviderError>>()?;
        Ok(Completion {
            text: self.text,
            tool_calls,
            finish_reason: self.finish_reason,
            usage: self.usage,
        })
    }
}
