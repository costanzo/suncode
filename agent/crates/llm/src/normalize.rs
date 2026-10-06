use crate::{BusinessError, Message};
use serde_json::{json, Value};

#[cfg(test)]
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

/// Converts SunCode's canonical transcript into Responses API input items.
///
/// Responses represents function calls and their outputs as distinct items,
/// rather than fields on assistant/tool messages.
pub fn responses_input_items(messages: &[Message]) -> Vec<Value> {
    let mut items = Vec::new();
    for message in messages {
        if message.role == "tool" {
            items.push(json!({
                "type": "function_call_output",
                "call_id": message.tool_call_id.clone().unwrap_or_default(),
                "output": response_tool_output(message),
            }));
            continue;
        }

        if !message.tool_calls.is_empty() {
            if !message.text_content().is_empty() {
                items.push(json!({
                    "role": message.role,
                    "content": if message.role == "assistant" {
                        json!([{"type":"output_text","text":message.text_content()}])
                    } else {
                        json!(message.text_content())
                    },
                }));
            }
            for call in &message.tool_calls {
                items.push(json!({
                    "type": "function_call",
                    "call_id": call.call_id,
                    "name": call.name,
                    "arguments": call.arguments.to_string(),
                }));
            }
            continue;
        }

        let content = if message.content.iter().all(|part| part.kind == "text") {
            if message.role == "assistant" {
                json!([{"type":"output_text","text":message.text_content()}])
            } else {
                json!(message.text_content())
            }
        } else {
            Value::Array(
                message
                    .content
                    .iter()
                    .filter_map(|part| match part.kind.as_str() {
                        "text" => Some(json!({"type":"input_text","text":part.text})),
                        "image_url" => Some(json!({"type":"input_image","image_url":part.text})),
                        _ => None,
                    })
                    .collect(),
            )
        };
        items.push(json!({"role": message.role, "content": content}));
    }
    items
}

fn response_tool_output(message: &Message) -> Value {
    if message.content.iter().all(|part| part.kind == "text") {
        return Value::String(message.text_content());
    }
    Value::Array(
        message
            .content
            .iter()
            .filter_map(|part| match part.kind.as_str() {
                "text" => Some(json!({"type": "input_text", "text": part.text})),
                "image_url" => Some(json!({"type": "input_image", "image_url": part.text})),
                _ => None,
            })
            .collect(),
    )
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

    #[test]
    fn responses_items_separate_function_calls_and_outputs() {
        let assistant = Message {
            role: "assistant".into(),
            content: Vec::new(),
            tool_calls: vec![crate::ToolCall {
                call_id: "call-1".into(),
                name: "read".into(),
                arguments: json!({"path":"README.md"}),
                toolset_name: None,
            }],
            tool_call_id: None,
        };
        let mut output = Message::text("tool", "hello");
        output.tool_call_id = Some("call-1".into());
        assert_eq!(
            responses_input_items(&[assistant, output]),
            vec![
                json!({"type":"function_call","call_id":"call-1","name":"read","arguments":"{\"path\":\"README.md\"}"}),
                json!({"type":"function_call_output","call_id":"call-1","output":"hello"}),
            ]
        );
    }

    #[test]
    fn responses_function_outputs_preserve_screenshot_images() {
        let mut output = Message::text("tool", "Computer screenshot captured.");
        output.content.push(ContentPart {
            kind: "image_url".into(),
            text: "data:image/png;base64,cG5n".into(),
        });
        output.tool_call_id = Some("call-1".into());
        assert_eq!(
            responses_input_items(&[output]),
            vec![json!({
                "type":"function_call_output",
                "call_id":"call-1",
                "output":[
                    {"type":"input_text","text":"Computer screenshot captured."},
                    {"type":"input_image","image_url":"data:image/png;base64,cG5n"}
                ]
            })]
        );
    }
}
