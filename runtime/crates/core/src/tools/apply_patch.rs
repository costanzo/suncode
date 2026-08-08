use serde_json::{json, Value};

pub fn definition() -> (&'static str, &'static str, Value) {
    (
        "apply_patch",
        "Apply a text patch to one project file after approval.",
        json!({"type":"object","required":["path","patchText","expected_base64"],"properties":{"path":{"type":"string"},"patchText":{"type":"string"},"expected_base64":{"type":"string"}},"additionalProperties":false}),
    )
}
