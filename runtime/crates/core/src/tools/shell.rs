use serde_json::{json, Value};

pub fn definition() -> (&'static str, &'static str, Value) {
    (
        "bash",
        "Execute one shell command string in the project after approval. Use the active host shell syntax.",
        json!({
            "type":"object",
            "required":["command"],
            "properties":{
                "command":{"type":"string","description":"Shell command string to execute"},
                "timeout":{"type":"integer","description":"Timeout in milliseconds. Defaults to 120000 and may not exceed 600000."},
                "workdir":{"type":"string","description":"Working directory for the command. Defaults to the active project directory."}
            },
            "additionalProperties":false
        }),
    )
}
