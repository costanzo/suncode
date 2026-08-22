use serde_json::{json, Value};

pub fn definition() -> (&'static str, &'static str, Value) {
    (
        "glob",
        "List files matching a glob pattern. Set path to dependency:<dependencyId> or a subdirectory below it to search a registered read-only dependency.",
        json!({"type":"object","required":["pattern"],"properties":{"pattern":{"type":"string"},"path":{"type":"string"},"limit":{"type":"integer"},"max_results":{"type":"integer"}},"additionalProperties":false}),
    )
}
