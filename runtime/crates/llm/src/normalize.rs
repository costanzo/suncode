use crate::{Message, ProviderError};
use serde_json::{json, Value};

pub fn wire_message(message: &Message) -> Value {
    let mut value = json!({"role": message.role, "content": message.text_content()});
    if !message.tool_calls.is_empty() {
        value["tool_calls"] = Value::Array(message.tool_calls.iter().map(|call| json!({"id":call.call_id,"type":"function","function":{"name":call.name,"arguments":call.arguments.to_string()}})).collect());
    }
    if let Some(id) = &message.tool_call_id {
        value["tool_call_id"] = json!(id);
    }
    value
}

pub fn cancelled() -> ProviderError {
    ProviderError {
        code: "cancelled".into(),
        message: "Turn was cancelled".into(),
        retryable: false,
    }
}
