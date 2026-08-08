use serde_json::{json, Value};

pub fn definition() -> (&'static str, &'static str, Value) {
    (
        "write",
        "Write UTF-8 content to one project file after approval.",
        json!({"type":"object","required":["path","content"],"properties":{"path":{"type":"string"},"content":{"type":"string"},"expected_base64":{"type":["string","null"]}},"additionalProperties":false}),
    )
}
