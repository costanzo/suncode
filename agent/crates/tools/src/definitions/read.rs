use serde_json::{json, Value};

pub fn definition() -> (&'static str, &'static str, Value) {
    (
        "read",
        "Read a bounded text file. Use offset (1-indexed) and limit to continue through large files. Relative paths resolve inside the current project; registered read-only dependencies use dependency:<dependencyId>/<path>.",
        json!({"type":"object","required":["path"],"properties":{"path":{"type":"string","description":"File path relative to the project"},"offset":{"type":"integer","minimum":1,"description":"1-indexed line to start reading from"},"limit":{"type":"integer","minimum":1,"description":"Maximum number of lines to return"},"max_bytes":{"type":"integer","minimum":1,"description":"Maximum number of bytes to return before creating an artifact"}},"additionalProperties":false}),
    )
}
