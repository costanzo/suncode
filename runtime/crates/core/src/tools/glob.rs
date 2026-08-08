use serde_json::{json, Value};

pub fn definition() -> (&'static str, &'static str, Value) {
    (
        "glob",
        "List project files matching a glob pattern.",
        json!({"type":"object","required":["pattern"],"properties":{"pattern":{"type":"string"},"path":{"type":"string"},"limit":{"type":"integer"},"max_results":{"type":"integer"}},"additionalProperties":false}),
    )
}
