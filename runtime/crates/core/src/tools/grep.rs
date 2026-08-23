use serde_json::{json, Value};

pub fn definition() -> (&'static str, &'static str, Value) {
    (
        "grep",
        "Search file contents with a regular expression. Results are bounded to 100 matches. Set path to a project subdirectory or dependency:<dependencyId> for a registered read-only dependency.",
        json!({"type":"object","required":["pattern"],"properties":{"pattern":{"type":"string","description":"Regular expression to search for"},"path":{"type":"string","description":"Directory or file to search"},"include":{"type":"string","description":"Optional file glob such as **/*.rs"}},"additionalProperties":false}),
    )
}
