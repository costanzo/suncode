use crate::{BusinessError, Message};
use serde_json::{json, Value};

pub fn wire_message(message: &Message) -> Value {
    let content = if message.content.iter().all(|part| part.kind == "text") {
        json!(message.text_content())
    } else {
        Value::Array(
            message
                .content
                .iter()
                .filter_map(|part| match part.kind.as_str() {
                    "text" => Some(json!({"type":"text","text":part.text})),
                    "image_url" => Some(json!({"type":"image_url","image_url":{"url":part.text}})),
                    _ => None,
                })
                .collect(),
        )
    };
    let mut value = json!({"role": message.role, "content": content});
    if !message.tool_calls.is_empty() {
        value["tool_calls"] = Value::Array(message.tool_calls.iter().filter(|call| call.toolset_name.is_none()).map(|call| json!({"id":call.call_id,"type":"function","function":{"name":call.name,"arguments":call.arguments.to_string()}})).collect());
    }
    if let Some(id) = &message.tool_call_id {
        value["tool_call_id"] = json!(id);
    }
    value
}

pub fn cancelled() -> BusinessError {
    BusinessError::new("cancelled", "Turn was cancelled")
}

pub(crate) fn is_context_overflow(status: u16, message: &str) -> bool {
    if status == 413 {
        return true;
    }
    if !matches!(status, 400 | 413 | 422) {
        return false;
    }
    let message = message.to_ascii_lowercase();
    [
        "context length",
        "context window",
        "context limit",
        "too many tokens",
        "maximum tokens",
        "prompt tokens",
    ]
    .iter()
    .any(|needle| message.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ContentPart;

    #[test]
    fn text_only_messages_keep_the_compatible_string_shape() {
        let message = Message::text("user", "hello");
        assert_eq!(
            wire_message(&message),
            json!({"role":"user","content":"hello"})
        );
    }

    #[test]
    fn multimodal_messages_emit_openai_compatible_content_parts() {
        let message = Message {
            role: "user".into(),
            content: vec![
                ContentPart {
                    kind: "text".into(),
                    text: "inspect this".into(),
                },
                ContentPart {
                    kind: "image_url".into(),
                    text: "data:image/png;base64,cG5n".into(),
                },
            ],
            tool_calls: vec![],
            tool_call_id: None,
        };

        assert_eq!(
            wire_message(&message),
            json!({
                "role":"user",
                "content":[
                    {"type":"text","text":"inspect this"},
                    {"type":"image_url","image_url":{"url":"data:image/png;base64,cG5n"}}
                ]
            })
        );
    }

    #[test]
    fn overflow_errors_are_classified_without_matching_unrelated_failures() {
        assert!(is_context_overflow(400, "maximum context length exceeded"));
        assert!(is_context_overflow(413, "request too large"));
        assert!(is_context_overflow(422, "too many tokens"));
        assert!(!is_context_overflow(401, "context length"));
        assert!(!is_context_overflow(400, "invalid model"));
    }
}
