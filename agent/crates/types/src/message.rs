use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContentPart {
    #[serde(rename = "type")]
    pub kind: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    pub call_id: String,
    pub name: String,
    pub arguments: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toolset_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub role: String,
    pub content: Vec<ContentPart>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn text(role: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: vec![ContentPart {
                kind: "text".into(),
                text: text.into(),
            }],
            tool_calls: Vec::new(),
            tool_call_id: None,
        }
    }

    pub fn text_content(&self) -> String {
        self.content
            .iter()
            .filter(|part| part.kind == "text")
            .map(|part| part.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn keeps_the_persisted_message_shape() {
        let mut message = Message::text("assistant", "done");
        assert_eq!(
            serde_json::to_value(&message).unwrap(),
            json!({"role": "assistant", "content": [{"type": "text", "text": "done"}]})
        );
        message.tool_calls.push(ToolCall {
            call_id: "call-1".into(),
            name: "read".into(),
            arguments: json!({}),
            toolset_name: None,
        });
        let value = serde_json::to_value(&message).unwrap();
        assert_eq!(
            value["tool_calls"],
            json!([{"call_id": "call-1", "name": "read", "arguments": {}}])
        );
        assert_eq!(serde_json::from_value::<Message>(value).unwrap(), message);
    }
}
