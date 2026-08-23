use serde_json::{json, Value};

pub fn definition() -> (&'static str, &'static str, Value) {
    (
        "process",
        "Run one bounded non-interactive program with an explicit argument array after approval. This does not invoke a shell; use bash when pipes, redirection, or shell syntax are required.",
        json!({
            "type":"object",
            "required":["program"],
            "properties":{
                "program":{"type":"string"},
                "args":{"type":"array","items":{"type":"string"}},
                "workdir":{"type":"string"},
                "cwd":{"type":"string"},
                "timeout":{"type":"number","description":"Timeout in seconds (maximum 600)"},
                "env":{"type":"object"}
            },
            "additionalProperties":false
        }),
    )
}
