//! Built-in tool declarations. Execution remains behind `suncode-operations`.
//!
//! The model-facing names follow OpenCode's built-in tool names. Internal
//! operation methods stay narrower and are translated by the agent before
//! execution.

mod apply_patch;
mod bash;
mod edit;
mod glob;
mod grep;
mod read;
mod write;

use serde_json::{json, Value};

pub fn definitions() -> Vec<Value> {
    [
        read::definition(),
        glob::definition(),
        grep::definition(),
        write::definition(),
        edit::definition(),
        apply_patch::definition(),
        bash::definition(),
    ]
    .into_iter()
    .map(|(name, description, parameters)| {
        json!({"type":"function","function":{"name":name,"description":description,"parameters":parameters}})
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::definitions;
    use std::collections::HashSet;

    #[test]
    fn built_in_tool_names_are_unique() {
        let definitions = definitions();
        let names = definitions
            .iter()
            .filter_map(|value| {
                value
                    .pointer("/function/name")
                    .and_then(|name| name.as_str())
            })
            .collect::<HashSet<_>>();
        assert_eq!(names.len(), definitions.len());
    }
}
