use serde_json::{json, Value};

pub fn definition() -> (&'static str, &'static str, Value) {
    (
        "write",
        "Write UTF-8 content to one project file after approval. Use the expected_base64 returned by read when updating an existing file.",
        json!({"type":"object","required":["path","content"],"properties":{"path":{"type":"string","description":"File path relative to the project"},"content":{"type":"string","description":"Complete file content"},"expected_base64":{"type":["string","null"],"description":"Previous file bytes from read; omit for a new file"}},"additionalProperties":false}),
    )
}
