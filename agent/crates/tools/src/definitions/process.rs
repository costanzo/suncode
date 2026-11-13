use serde_json::{json, Value};

pub(super) fn bash() -> (&'static str, &'static str, Value) {
    (
        "bash",
        "Execute one terminal shell command string in the project after approval. Do not use this tool for file operations such as reading, writing, editing, finding, or searching; use the specialized read, write, edit, glob, and grep tools instead. Use the active host shell syntax.",
        json!({
            "type": "object",
            "required": ["command"],
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Shell command string to execute"
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in milliseconds. Defaults to 120000 and may not exceed 600000."
                },
                "workdir": {
                    "type": "string",
                    "description": "Working directory relative to the main project, or an absolute path inside the main project or a registered dependency. Defaults to the main project. Use real filesystem paths in the command string; legacy dependency aliases are accepted only in this workdir field."
                }
            },
            "additionalProperties": false
        }),
    )
}
