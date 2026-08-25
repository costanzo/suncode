use serde_json::{json, Value};

pub fn definition() -> (&'static str, &'static str, Value) {
    (
        "edit",
        "Replace exact text in one project file after approval. For multiple independent edits, provide edits with oldText and newText entries.",
        json!({"type":"object","required":["path","expected_base64"],"properties":{"path":{"type":"string","description":"File path relative to the project"},"oldString":{"type":"string","description":"Exact text to replace"},"newString":{"type":"string","description":"Replacement text"},"edits":{"type":"array","items":{"type":"object","required":["oldText","newText"],"properties":{"oldText":{"type":"string"},"newText":{"type":"string"}},"additionalProperties":false}},"replaceAll":{"type":"boolean"},"expected_base64":{"type":"string","description":"Previous file bytes from read"}},"additionalProperties":false}),
    )
}
