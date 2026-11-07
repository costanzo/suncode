use serde_json::{json, Value};

pub fn definition() -> (&'static str, &'static str, Value) {
    (
        "delete",
        "Delete one project file after approval. Read the file first and pass its expected_base64. Use this tool instead of bash rm so the deletion appears in the turn's changes. Directories cannot be deleted.",
        json!({"type":"object","required":["path","expected_base64"],"properties":{"path":{"type":"string","description":"File path relative to the project"},"expected_base64":{"type":"string","description":"Previous file bytes from read"}},"additionalProperties":false}),
    )
}