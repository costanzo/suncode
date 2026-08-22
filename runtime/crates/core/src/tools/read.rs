use serde_json::{json, Value};

pub fn definition() -> (&'static str, &'static str, Value) {
    (
        "read",
        "Read a bounded UTF-8 file. Relative paths resolve inside the current project; registered read-only dependencies use dependency:<dependencyId>/<path>.",
        json!({"type":"object","required":["path"],"properties":{"path":{"type":"string"},"offset":{"type":"integer"},"limit":{"type":"integer"},"max_bytes":{"type":"integer"}},"additionalProperties":false}),
    )
}
