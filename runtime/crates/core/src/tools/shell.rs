use serde_json::{json, Value};

pub fn definition() -> (&'static str, &'static str, Value) {
    (
        "shell",
        shell_description(),
        json!({
            "type":"object",
            "required":["script"],
            "properties":{
                "script":{"type":"string"},
                "workdir":{"type":"string"},
                "cwd":{"type":"string"},
                "timeout":{"type":"number","description":"Timeout in seconds (maximum 600)"},
                "env":{"type":"object"}
            },
            "additionalProperties":false
        }),
    )
}

#[cfg(target_os = "windows")]
fn shell_description() -> &'static str {
    "Run a bounded non-interactive Windows PowerShell script in the project after approval. Use PowerShell syntax."
}

#[cfg(not(target_os = "windows"))]
fn shell_description() -> &'static str {
    "Run a bounded non-interactive POSIX shell script in the project after approval. Use POSIX sh syntax."
}
