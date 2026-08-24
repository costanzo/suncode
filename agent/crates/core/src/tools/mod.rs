//! Built-in tool declarations. Execution remains behind `suncode-tool`.
//!
//! The model-facing names follow OpenCode's built-in tool names. Internal
//! operation methods stay narrower and are translated by the agent before
//! execution.

mod edit;
mod glob;
mod grep;
mod read;
mod shell;
mod webfetch;
mod write;

use suncode_llm::ToolDefinition;

pub fn definitions() -> Vec<ToolDefinition> {
    [
        read::definition(),
        glob::definition(),
        grep::definition(),
        write::definition(),
        edit::definition(),
        shell::definition(),
        webfetch::definition(),
    ]
    .into_iter()
    .map(|(name, description, parameters)| ToolDefinition {
        name: name.into(),
        description: description.into(),
        parameters,
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::definitions;
    use std::collections::BTreeSet;

    #[test]
    fn built_in_tool_names_match_the_model_contract() {
        let definitions = definitions();
        let names = definitions
            .iter()
            .map(|definition| definition.name.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            names,
            BTreeSet::from(["bash", "edit", "glob", "grep", "read", "webfetch", "write",])
        );
        assert_eq!(definitions.len(), names.len());
        let bash = definitions
            .iter()
            .find(|definition| definition.name == "bash")
            .unwrap();
        assert_eq!(bash.parameters["required"], serde_json::json!(["command"]));
        assert!(bash.parameters["properties"]["command"].is_object());
        assert_eq!(bash.parameters["properties"]["timeout"]["type"], "integer");
        assert_eq!(bash.parameters["properties"]["workdir"]["type"], "string");
        let glob = definitions
            .iter()
            .find(|definition| definition.name == "glob")
            .unwrap();
        assert!(glob.parameters["properties"].get("limit").is_none());
        let grep = definitions
            .iter()
            .find(|definition| definition.name == "grep")
            .unwrap();
        assert!(grep.description.contains("Do not use bash"));
        assert!(grep.description.contains("counting matches"));
        assert!(grep.parameters["properties"].get("query").is_none());
        assert!(grep.parameters["properties"].get("max_results").is_none());
        assert!(bash
            .description
            .contains("Do not use this tool for file operations"));
        let webfetch = definitions
            .iter()
            .find(|definition| definition.name == "webfetch")
            .unwrap();
        assert_eq!(webfetch.parameters["required"], serde_json::json!(["url"]));
        assert_eq!(
            webfetch.parameters["properties"]["format"]["enum"],
            serde_json::json!(["text", "markdown", "html"])
        );
    }
}
