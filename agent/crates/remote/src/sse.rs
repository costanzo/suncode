use serde::Deserialize;
use serde_json::{json, Value};

use crate::{crypto::decrypt_remote_payload, events::redact_remote_value, logging};

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RemoteRequest {
    #[serde(alias = "requestId")]
    pub(crate) request_id: String,
    #[serde(alias = "command", default)]
    pub(crate) operation: String,
    #[serde(default, alias = "payload")]
    pub(crate) arguments: Value,
}

/// Largest SSE frame accepted from the relay before the stream is dropped.
pub(crate) const MAX_SSE_FRAME_BYTES: usize = 2 * 1024 * 1024;

pub(crate) fn log_sse_frame(frame: &[u8]) {
    let text = String::from_utf8_lossy(frame);
    let rendered = text
        .lines()
        .map(|line| {
            line.strip_prefix("data:")
                .and_then(|data| serde_json::from_str::<Value>(data.trim()).ok())
                .map(|value| format!("data: {}", redact_remote_value(&value)))
                .unwrap_or_else(|| line.to_owned())
        })
        .collect::<Vec<_>>()
        .join("\\n");
    logging::debug(
        "remote",
        format!("SSE event received frame={}", rendered.trim()),
    );
}

pub(crate) fn is_sse_comment(frame: &[u8]) -> bool {
    frame
        .split(|byte| *byte == b'\n' || *byte == b'\r')
        .map(|line| line.strip_prefix(b" ").unwrap_or(line))
        .filter(|line| !line.is_empty())
        .all(|line| line.starts_with(b":"))
}

/// Parses one relay SSE frame into a request.
///
/// With `e2e_required`, request bodies must arrive as `encPayload`; plaintext
/// bodies and the legacy `desktop.*` envelope are rejected so the relay cannot
/// inject arguments. Path and query values stay visible to the relay by design.
pub(crate) fn parse_sse_request(
    frame: &[u8],
    aes_key: &str,
    e2e_required: bool,
) -> Option<RemoteRequest> {
    let text = std::str::from_utf8(frame).ok()?;
    let mut event = String::new();
    let mut id = String::new();
    let mut data_lines = Vec::new();
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("event:") {
            event = value.trim().into();
        }
        if let Some(value) = line.strip_prefix("id:") {
            id = value.trim().into();
        }
        if let Some(value) = line.strip_prefix("data:") {
            data_lines.push(value.strip_prefix(' ').unwrap_or(value).to_owned());
        }
    }
    if event.starts_with("mobile.") {
        if data_lines.len() != 2 || id.trim().is_empty() {
            return None;
        }
        let routing: Value = serde_json::from_str(&data_lines[0]).ok()?;
        let path = routing.get("pathParam")?.as_object()?;
        let query = routing.get("queryParam")?.as_object()?;
        let body: Value = serde_json::from_str(&data_lines[1]).ok()?;
        let operation = mobile_event_operation(&event)?;
        let encrypted_payload = body
            .get("encPayload")
            .and_then(Value::as_str)
            .filter(|payload| !payload.trim().is_empty());
        let mut arguments = if let Some(enc_payload) = encrypted_payload {
            decrypt_remote_payload(enc_payload, aes_key).ok()?
        } else if e2e_required && !is_empty_body(&body) {
            logging::debug(
                "remote",
                "rejected plaintext request body while E2E is enabled",
            );
            return None;
        } else {
            body
        };
        if !arguments.is_object() {
            arguments = json!({});
        }
        if let Some(object) = arguments.as_object_mut() {
            for (key, value) in path {
                object.entry(key.clone()).or_insert_with(|| value.clone());
            }
        }
        if let Some(object) = arguments.as_object_mut() {
            for (key, value) in query {
                object.entry(key.clone()).or_insert_with(|| value.clone());
            }
        }
        return Some(RemoteRequest {
            request_id: id,
            operation: operation.into(),
            arguments,
        });
    }
    if e2e_required || (event != "desktop.request" && event != "desktop.command") {
        return None;
    }
    let mut value: Value = serde_json::from_str(&data_lines.join("\n")).ok()?;
    if value.get("request_id").is_none() && value.get("requestId").is_none() {
        value["request_id"] = Value::String(id);
    }
    serde_json::from_value(value)
        .ok()
        .filter(|request: &RemoteRequest| !request.request_id.trim().is_empty())
}

pub(crate) fn is_empty_body(body: &Value) -> bool {
    body.is_null() || body.as_object().is_some_and(serde_json::Map::is_empty)
}

pub(crate) fn mobile_event_operation(event: &str) -> Option<&'static str> {
    Some(match event {
        "mobile.projects.list" => "projects.list",
        "mobile.sessions.list" => "sessions.list",
        "mobile.sessions.create" => "session.create",
        "mobile.sessions.get" => "session.get",
        "mobile.sessions.messages.send" => "session.message",
        "mobile.sessions.approvals.resolve" => "approval.resolve",
        "mobile.sessions.questions.reply" => "question.reply",
        "mobile.sessions.cancel" => "turn.cancel",
        "mobile.sessions.retry" => "turn.retry",
        _ => return None,
    })
}

pub(crate) fn sse_frame_boundary(buffer: &[u8]) -> Option<(usize, usize)> {
    if let Some(position) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
        return Some((position, 4));
    }
    buffer
        .windows(2)
        .position(|window| window == b"\n\n")
        .map(|position| (position, 2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        crypto::{encrypt_remote_payload, generate_aes_key},
        RemoteResult,
    };
    use suncode_common::BusinessError;

    fn normalize_question_answers(answers: Value) -> RemoteResult<Value> {
        let Some(values) = answers.as_array() else {
            return Err(BusinessError::invalid("answers must be an array"));
        };
        if values.iter().all(Value::is_string) {
            return Ok(Value::Array(
                values
                    .iter()
                    .map(|value| Value::Array(vec![value.clone()]))
                    .collect(),
            ));
        }
        Ok(answers)
    }

    #[test]
    fn parses_only_sse_request_frames_with_request_ids() {
        let request = parse_sse_request(b"event: desktop.request\ndata: {\"request_id\":\"req-1\",\"operation\":\"projects.list\"}\n\n", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", false).unwrap();
        assert_eq!(request.request_id, "req-1");
        assert!(parse_sse_request(b"event: desktop.request\r\ndata: {\"request_id\":\"req-2\",\"operation\":\"projects.list\"}\r\n\r\n", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", false).is_some());
        assert!(parse_sse_request(
            b"event: heartbeat\ndata: {}\n\n",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            false
        )
        .is_none());
        assert!(parse_sse_request(
            b"event: desktop.request\ndata: {\"operation\":\"projects.list\"}\n\n",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            false
        )
        .is_none());
    }

    #[test]
    fn accepts_java_desktop_command_envelope_and_camel_case_payload() {
        let request = parse_sse_request(
            b"event: desktop.command\ndata: {\"requestId\":\"req-1\",\"hostId\":\"host-1\",\"sessionId\":\"session-1\",\"command\":\"session.message\",\"payload\":{\"text\":\"hello\"}}\n\n", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            false,
        )
        .unwrap();
        assert_eq!(request.request_id, "req-1");
        assert_eq!(request.operation, "session.message");
        assert_eq!(request.arguments["text"], "hello");
    }

    #[test]
    fn keeps_desktop_sse_routing_values_with_plaintext_body() {
        let request = parse_sse_request(
            b"event: mobile.sessions.approvals.resolve\nid: req-1\ndata: {\"pathParam\":{\"sessionId\":\"session-1\",\"approvalId\":\"approval-1\"},\"queryParam\":{\"revision\":7}}\ndata: {\"action\":\"allow_once\"}\n\n",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            false,
        ).unwrap();
        assert_eq!(request.operation, "approval.resolve");
        assert_eq!(request.arguments["sessionId"], "session-1");
        assert_eq!(request.arguments["approvalId"], "approval-1");
        assert_eq!(request.arguments["revision"], 7);
        assert_eq!(request.arguments["action"], "allow_once");
    }

    #[test]
    fn rejects_mobile_sse_frames_without_both_data_lines() {
        assert!(parse_sse_request(
            b"event: mobile.sessions.messages.send\nid: req-1\ndata: {\"pathParam\":{},\"queryParam\":{}}\n\n",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            false,
        )
        .is_none());
    }

    #[test]
    fn decrypts_top_level_encrypted_mobile_payload_on_desktop() {
        let key = generate_aes_key();
        let encrypted = encrypt_remote_payload(&json!({"text": "hello"}), &key).unwrap();
        let routing = json!({
            "pathParam": {"sessionId": "session-1"},
            "queryParam": {"expectedRevision": 7},
        });
        let body = json!({"encPayload": encrypted});
        let frame = format!(
            "event: mobile.sessions.messages.send\nid: req-1\ndata: {}\ndata: {}\n\n",
            routing, body
        );
        let request = parse_sse_request(frame.as_bytes(), &key, true).unwrap();
        assert_eq!(request.operation, "session.message");
        assert_eq!(request.arguments["text"], "hello");
        assert_eq!(request.arguments["sessionId"], "session-1");
        assert_eq!(request.arguments["expectedRevision"], 7);
    }

    #[test]
    fn rejects_plaintext_bodies_and_legacy_envelopes_when_e2e_is_required() {
        let key = generate_aes_key();
        assert!(parse_sse_request(
            b"event: mobile.sessions.approvals.resolve\nid: req-1\ndata: {\"pathParam\":{\"sessionId\":\"s\",\"approvalId\":\"a\"},\"queryParam\":{}}\ndata: {\"action\":\"allow_once\"}\n\n",
            &key,
            true,
        )
        .is_none());
        assert!(parse_sse_request(
            b"event: desktop.command\ndata: {\"requestId\":\"req-1\",\"command\":\"session.message\",\"payload\":{\"text\":\"hi\"}}\n\n",
            &key,
            true,
        )
        .is_none());
        // Bodyless requests carry no arguments and remain routable.
        assert!(parse_sse_request(
            b"event: mobile.projects.list\nid: req-2\ndata: {\"pathParam\":{},\"queryParam\":{}}\ndata: {}\n\n",
            &key,
            true,
        )
        .is_some());
    }

    #[test]
    fn parses_mobile_projects_list_event() {
        let request = parse_sse_request(
            b"event: mobile.projects.list\nid: 1791094808946\ndata: {\"pathParam\":{\"hostId\":\"BwyviZPiGD\"},\"queryParam\":{}}\ndata: {}\n\n",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            false,
        )
        .expect("mobile projects list event should be accepted");
        assert_eq!(request.request_id, "1791094808946");
        assert_eq!(request.operation, "projects.list");
        assert_eq!(request.arguments["hostId"], "BwyviZPiGD");
    }

    #[test]
    fn normalizes_java_question_answers_to_sdk_shape() {
        assert_eq!(
            normalize_question_answers(serde_json::json!(["yes", "no"])).unwrap(),
            serde_json::json!([["yes"], ["no"]])
        );
        assert_eq!(
            normalize_question_answers(serde_json::json!([["yes"], ["no"]])).unwrap(),
            serde_json::json!([["yes"], ["no"]])
        );
    }
}
