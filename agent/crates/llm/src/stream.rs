use crate::{BusinessError, Completion, ToolCall, Usage};
use serde_json::Value;
use std::collections::BTreeMap;

pub struct SseParser {
    provider_label: String,
    buffer: String,
    text: String,
    calls: BTreeMap<u64, (String, String, String)>,
    finish_reason: String,
    usage: Option<Usage>,
    response_id: Option<String>,
}

impl SseParser {
    pub fn new(provider_label: impl Into<String>) -> Self {
        Self {
            provider_label: provider_label.into(),
            buffer: String::new(),
            text: String::new(),
            calls: BTreeMap::new(),
            finish_reason: String::new(),
            usage: None,
            response_id: None,
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
        if self.response_id.is_none() {
            self.response_id = chunk
                .get("id")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .map(str::to_owned);
        }
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
                cache_read_tokens: usage
                    .pointer("/prompt_tokens_details/cached_tokens")
                    .or_else(|| usage.pointer("/input_tokens_details/cached_tokens"))
                    .or_else(|| usage.get("cached_tokens"))
                    .or_else(|| usage.get("prompt_cache_hit_tokens"))
                    .or_else(|| usage.get("cache_read_input_tokens"))
                    .or_else(|| usage.get("cache_read_tokens"))
                    .and_then(Value::as_u64),
                cache_miss_tokens: usage
                    .pointer("/prompt_tokens_details/cache_miss_tokens")
                    .or_else(|| usage.pointer("/input_tokens_details/cache_miss_tokens"))
                    .or_else(|| usage.get("prompt_cache_miss_tokens"))
                    .or_else(|| usage.get("cache_miss_tokens"))
                    .and_then(Value::as_u64),
                cache_write_tokens: usage
                    .get("cache_creation_input_tokens")
                    .or_else(|| usage.get("cache_write_tokens"))
                    .and_then(Value::as_u64),
                reasoning_tokens: usage
                    .pointer("/completion_tokens_details/reasoning_tokens")
                    .or_else(|| usage.pointer("/output_tokens_details/reasoning_tokens"))
                    .or_else(|| usage.get("reasoning_tokens"))
                    .and_then(Value::as_u64),
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

    pub fn finish(self) -> Result<Completion, BusinessError> {
        let tool_calls = self
            .calls
            .into_values()
            .map(|(id, name, args)| {
                Ok(ToolCall {
                    call_id: id,
                    name,
                    arguments: serde_json::from_str(&args).map_err(|_| {
                        BusinessError::new(
                            "malformed_tool_call",
                            "Provider returned invalid tool arguments",
                        )
                    })?,
                })
            })
            .collect::<Result<Vec<_>, BusinessError>>()?;
        Ok(Completion {
            text: self.text,
            tool_calls,
            finish_reason: self.finish_reason,
            usage: self.usage,
            provider_request_id: None,
            provider_response_id: self.response_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::SseParser;

    fn parse_usage(usage: &str) -> crate::Usage {
        let usage: serde_json::Value = serde_json::from_str(usage).unwrap();
        let mut parser = SseParser::new("Test provider");
        parser
            .push(format!("data: {}\n\n", serde_json::json!({"usage": usage})).as_bytes())
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
