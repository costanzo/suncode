use serde_json::{json, Value};

pub fn definition() -> (&'static str, &'static str, Value) {
    (
        "read",
        "Read a bounded UTF-8 project file. Relative paths resolve inside the current project.",
        json!({"type":"object","required":["path"],"properties":{"path":{"type":"string"},"offset":{"type":"integer"},"limit":{"type":"integer"},"max_bytes":{"type":"integer"}},"additionalProperties":false}),
    )
}
