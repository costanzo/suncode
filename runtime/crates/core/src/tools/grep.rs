use serde_json::{json, Value};

pub fn definition() -> (&'static str, &'static str, Value) {
    (
        "grep",
        "Fast content search for regular expressions. Use this tool whenever you need to find files or matching lines. Do not use bash to run grep or rg for normal file search; use bash with rg only when you explicitly need raw terminal behavior such as counting matches. Results are bounded to 100 matches. Set path to a project subdirectory or dependency:<dependencyId> for a registered read-only dependency.",
        json!({"type":"object","required":["pattern"],"properties":{"pattern":{"type":"string","description":"Regular expression to search for"},"path":{"type":"string","description":"Directory or file to search"},"include":{"type":"string","description":"Optional file glob such as **/*.rs"}},"additionalProperties":false}),
    )
}
