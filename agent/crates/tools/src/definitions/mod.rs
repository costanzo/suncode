//! Model-facing declarations for the built-in tools.

mod edit;
mod glob;
mod grep;
mod question;
mod read;
mod shell;
mod todowrite;
mod webfetch;
mod write;

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: Value,
}

pub fn all() -> Vec<ToolDefinition> {
    [
        read::definition(),
        glob::definition(),
        grep::definition(),
        question::definition(),
        todowrite::definition(),
        write::definition(),
        edit::definition(),
        shell::definition(),
        webfetch::definition(),
    ]
    .into_iter()
    .map(|(name, description, parameters)| ToolDefinition {
        name,
        description,
        parameters,
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::all;
    use std::collections::BTreeSet;

    #[test]
    fn built_in_tool_names_match_the_model_contract() {
        let definitions = all();
        let names = definitions
            .iter()
            .map(|definition| definition.name)
            .collect::<BTreeSet<_>>();
        assert_eq!(
            names,
            BTreeSet::from([
                "bash",
                "edit",
                "glob",
                "grep",
                "question",
                "read",
                "todowrite",
                "webfetch",
                "write",
            ])
        );
        assert_eq!(definitions.len(), names.len());
        let question = definitions
            .iter()
            .find(|definition| definition.name == "question")
            .unwrap();
        assert_eq!(
            question.parameters["required"],
            serde_json::json!(["questions"])
        );
        let todowrite = definitions
            .iter()
            .find(|definition| definition.name == "todowrite")
            .unwrap();
        assert_eq!(
            todowrite.parameters["required"],
            serde_json::json!(["todos"])
        );
        assert_eq!(
            todowrite.parameters["properties"]["todos"]["items"]["required"],
            serde_json::json!(["content", "status", "priority"])
        );
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
        assert_eq!(glob.parameters["properties"]["limit"]["minimum"], 1);
        let grep = definitions
            .iter()
            .find(|definition| definition.name == "grep")
            .unwrap();
        assert!(grep.description.contains("Do not use bash"));
        assert!(grep.description.contains("counting matches"));
        assert!(grep.parameters["properties"].get("query").is_none());
        assert_eq!(grep.parameters["properties"]["limit"]["maximum"], 500);
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
