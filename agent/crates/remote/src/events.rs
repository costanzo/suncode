use reqwest::Client;
use serde::Serialize;
use serde_json::{json, Value};
use std::time::Duration;

use crate::{
    client::{log_http_request, request_headers, resolve_endpoint},
    config::PersistedRemote,
    crypto::remote_request_body,
    logging,
};

pub(crate) const REMOTE_DELTA_MAX_CHARS: usize = 160;
pub(crate) const REMOTE_DELTA_FLUSH_INTERVAL: Duration = Duration::from_millis(120);

pub(crate) const UPLOAD_EVENT_TYPES: &[&str] = &[
    "turn.state",
    "tool.state",
    "tool.requested",
    "tool.result",
    "message.user",
    "message.assistant",
    "assistant.delta",
    "message.tool",
    "turn.queued",
    "turn.completed",
    "usage.updated",
    "context.compacted",
    "approval.requested",
    "approval.resolved",
    "question.asked",
    "question.replied",
    "question.rejected",
    "todo.updated",
    "checkpoint.captured",
    "checkpoint.item_restored",
    "checkpoint.restore_failed",
    "checkpoint.restored",
];

pub fn should_upload_event(event_type: &str) -> bool {
    UPLOAD_EVENT_TYPES.contains(&event_type)
}

#[derive(Debug, Serialize)]
pub(crate) struct RemoteEventEnvelope {
    #[serde(rename = "session_id")]
    pub(crate) session_id: String,
    #[serde(rename = "occurred_at")]
    pub(crate) occurred_at: String,
    #[serde(rename = "event_type")]
    pub(crate) event_type: String,
    pub(crate) payload: Value,
}

#[derive(Default)]
pub(crate) struct RemoteAssistantDeltaBuffer {
    pub(crate) turn_id: Option<String>,
    pub(crate) text: String,
    pub(crate) occurred_at: String,
}

impl RemoteAssistantDeltaBuffer {
    pub(crate) fn push(&mut self, payload: Value, occurred_at: &str) -> Vec<Value> {
        let text = payload
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if text.is_empty() {
            return Vec::new();
        }
        let turn_id = payload
            .get("turn_id")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let mut ready = Vec::new();
        if self.turn_id.is_some() && self.turn_id != turn_id {
            if let Some(payload) = self.flush() {
                ready.push(payload);
            }
        }
        self.turn_id = turn_id;
        self.text.push_str(text);
        self.occurred_at = occurred_at.to_owned();

        while let Some(end) = self.next_chunk_end() {
            ready.push(self.take_chunk(end));
        }
        ready
    }

    pub(crate) fn flush(&mut self) -> Option<Value> {
        if self.text.is_empty() {
            return None;
        }
        let text = std::mem::take(&mut self.text);
        Some(json!({"turn_id": self.turn_id, "text": text}))
    }

    pub(crate) fn occurred_at(&self) -> &str {
        &self.occurred_at
    }

    pub(crate) fn next_chunk_end(&self) -> Option<usize> {
        let max_end = self
            .text
            .char_indices()
            .nth(REMOTE_DELTA_MAX_CHARS)
            .map(|(index, _)| index)
            .or_else(|| {
                (self.text.chars().count() >= REMOTE_DELTA_MAX_CHARS).then_some(self.text.len())
            });
        if let Some(end) = paragraph_end(&self.text) {
            if max_end.is_none_or(|limit| end <= limit) {
                return Some(end);
            }
        }
        max_end
    }

    pub(crate) fn take_chunk(&mut self, end: usize) -> Value {
        let remaining = self.text.split_off(end);
        let text = std::mem::replace(&mut self.text, remaining);
        json!({"turn_id": self.turn_id, "text": text})
    }
}

pub(crate) fn paragraph_end(text: &str) -> Option<usize> {
    let lf = text.find("\n\n").map(|index| index + 2);
    let crlf = text.find("\r\n\r\n").map(|index| index + 4);
    match (lf, crlf) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(end), None) | (None, Some(end)) => Some(end),
        (None, None) => None,
    }
}

pub(crate) async fn flush_remote_assistant_delta(
    buffer: &mut RemoteAssistantDeltaBuffer,
    client: &Client,
    persisted: &PersistedRemote,
    host_id: &str,
    session_id: &str,
) {
    if let Some(payload) = buffer.flush() {
        let occurred_at = buffer.occurred_at().to_owned();
        upload_remote_session_event(
            client,
            persisted,
            host_id,
            session_id,
            "assistant.delta",
            &occurred_at,
            payload,
        )
        .await;
    }
}

pub(crate) async fn upload_remote_session_event(
    client: &Client,
    persisted: &PersistedRemote,
    host_id: &str,
    session_id: &str,
    event_type: &str,
    occurred_at: &str,
    payload: Value,
) {
    let event_id = suncode_common::id::new(suncode_common::id::IdKind::RemoteEvent);
    let event = RemoteEventEnvelope {
        session_id: session_id.to_owned(),
        occurred_at: occurred_at.to_owned(),
        event_type: event_type.to_owned(),
        payload,
    };
    let url = resolve_endpoint(&persisted.config.server_url, "/v1/desktop/events");
    let body = match remote_request_body(
        &serde_json::to_value(&event).unwrap_or_else(|_| json!({})),
        persisted,
    ) {
        Ok(body) => body,
        Err(error) => {
            logging::warn(
                "remote",
                format!("event encryption failed: {}", error.message),
            );
            return;
        }
    };
    log_http_request("POST", &url, &body);
    match request_headers(client.post(url), persisted, host_id)
        .header("X-Request-Id", &event_id)
        .json(&body)
        .send()
        .await
    {
        Ok(response) if !response.status().is_success() => logging::warn(
            "remote",
            format!("event upload returned HTTP {}", response.status()),
        ),
        Err(error) => logging::warn("remote", format!("event upload failed: {error}")),
        _ => {}
    }
}

pub(crate) fn redact_remote_value(value: &Value) -> String {
    let mut value = value.clone();
    redact_remote_object(&mut value);
    serde_json::to_string(&value).unwrap_or_else(|_| "<invalid-json>".into())
}

pub(crate) fn redact_remote_object(value: &mut Value) {
    match value {
        Value::Object(object) => {
            for (key, value) in object.iter_mut() {
                if matches!(
                    key.as_str(),
                    "encPayload"
                        | "accessToken"
                        | "refreshToken"
                        | "apiKey"
                        | "password"
                        | "pairingCode"
                        | "pairing_code"
                        | "code"
                        | "k"
                ) {
                    *value = Value::String("<redacted>".into());
                } else {
                    redact_remote_object(value);
                }
            }
        }
        Value::Array(values) => values.iter_mut().for_each(redact_remote_object),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowlist_uploads_complete_assistant_and_coalesced_delta_events() {
        assert!(should_upload_event("message.assistant"));
        assert!(should_upload_event("message.user"));
        assert!(should_upload_event("assistant.delta"));
        assert!(!should_upload_event("provider.exchange.progress"));
        assert!(!should_upload_event("unknown.event"));
    }

    #[test]
    fn remote_delta_buffer_combines_fragments_until_flush() {
        let mut buffer = RemoteAssistantDeltaBuffer::default();
        assert!(buffer
            .push(json!({"turn_id": "turn-1", "text": "Hello"}), "t1")
            .is_empty());
        assert!(buffer
            .push(json!({"turn_id": "turn-1", "text": " world"}), "t2")
            .is_empty());
        assert_eq!(
            buffer.flush(),
            Some(json!({"turn_id": "turn-1", "text": "Hello world"}))
        );
        assert_eq!(buffer.occurred_at(), "t2");
    }

    #[test]
    fn remote_delta_buffer_splits_at_unicode_character_limit() {
        let mut buffer = RemoteAssistantDeltaBuffer::default();
        let text = "你".repeat(REMOTE_DELTA_MAX_CHARS + 2);
        let chunks = buffer.push(json!({"turn_id": "turn-1", "text": text}), "t1");
        assert_eq!(chunks.len(), 1);
        assert_eq!(
            chunks[0]["text"].as_str().unwrap().chars().count(),
            REMOTE_DELTA_MAX_CHARS
        );
        assert_eq!(
            buffer.flush().unwrap()["text"]
                .as_str()
                .unwrap()
                .chars()
                .count(),
            2
        );
    }

    #[test]
    fn remote_delta_buffer_caps_long_paragraphs_at_character_limit() {
        let mut buffer = RemoteAssistantDeltaBuffer::default();
        let text = format!(
            "{}\n\nLater paragraph",
            "x".repeat(REMOTE_DELTA_MAX_CHARS + 20)
        );
        let chunks = buffer.push(json!({"turn_id": "turn-1", "text": text}), "t1");
        assert!(chunks.len() >= 2);
        assert!(
            chunks
                .iter()
                .all(|chunk| chunk["text"].as_str().unwrap().chars().count()
                    <= REMOTE_DELTA_MAX_CHARS)
        );
    }

    #[test]
    fn remote_delta_buffer_prefers_paragraph_boundary_and_flushes_on_turn_change() {
        let mut buffer = RemoteAssistantDeltaBuffer::default();
        let chunks = buffer.push(
            json!({"turn_id": "turn-1", "text": "First paragraph.\n\nSecond"}),
            "t1",
        );
        assert_eq!(
            chunks,
            vec![json!({"turn_id": "turn-1", "text": "First paragraph.\n\n"})]
        );
        let chunks = buffer.push(json!({"turn_id": "turn-2", "text": "Next turn"}), "t2");
        assert_eq!(chunks, vec![json!({"turn_id": "turn-1", "text": "Second"})]);
        assert_eq!(
            buffer.flush(),
            Some(json!({"turn_id": "turn-2", "text": "Next turn"}))
        );
    }
}
