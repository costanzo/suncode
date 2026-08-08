use serde_json::{json, Value};

pub fn definition() -> (&'static str, &'static str, Value) {
    (
        "edit",
        "Replace exact text in one project file after approval.",
        json!({"type":"object","required":["path","oldString","newString","expected_base64"],"properties":{"path":{"type":"string"},"oldString":{"type":"string"},"newString":{"type":"string"},"replaceAll":{"type":"boolean"},"expected_base64":{"type":"string"}},"additionalProperties":false}),
    )
}
