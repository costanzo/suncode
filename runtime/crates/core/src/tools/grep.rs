use serde_json::{json, Value};

pub fn definition() -> (&'static str, &'static str, Value) {
    (
        "grep",
        "Search file contents for text or a regular expression. Set path to dependency:<dependencyId> or a subdirectory below it to search a registered read-only dependency.",
        json!({"type":"object","required":["pattern"],"properties":{"pattern":{"type":"string"},"path":{"type":"string"},"include":{"type":"string"},"limit":{"type":"integer"},"query":{"type":"string"},"max_results":{"type":"integer"}},"additionalProperties":false}),
    )
}
