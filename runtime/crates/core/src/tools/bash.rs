use serde_json::{json, Value};

pub fn definition() -> (&'static str, &'static str, Value) {
    (
        "bash",
        "Run a bounded non-interactive shell command after approval.",
        json!({"type":"object","required":["command"],"properties":{"command":{"type":"string"},"workdir":{"type":"string"},"cwd":{"type":"string"},"timeout":{"type":"integer"},"timeout_ms":{"type":"integer"},"env":{"type":"object"}},"additionalProperties":false}),
    )
}
