use serde_json::{json, Value};

pub fn definition() -> (&'static str, &'static str, Value) {
    (
        "glob",
        "List project files matching a glob pattern. Results respect ignore rules and are bounded to 100 entries. Set path to a project subdirectory or dependency:<dependencyId> for a registered read-only dependency.",
        json!({"type":"object","required":["pattern"],"properties":{"pattern":{"type":"string","description":"Glob pattern such as **/*.rs"},"path":{"type":"string","description":"Directory to search, relative to the project"},"limit":{"type":"integer","minimum":1,"maximum":1000,"description":"Maximum number of paths to return"}},"additionalProperties":false}),
    )
}
