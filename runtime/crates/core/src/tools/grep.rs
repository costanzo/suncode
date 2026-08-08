use serde_json::{json, Value};

pub fn definition() -> (&'static str, &'static str, Value) {
    (
        "grep",
        "Search project file contents for text or a regular expression.",
        json!({"type":"object","required":["pattern"],"properties":{"pattern":{"type":"string"},"path":{"type":"string"},"include":{"type":"string"},"limit":{"type":"integer"},"query":{"type":"string"},"max_results":{"type":"integer"}},"additionalProperties":false}),
    )
}
