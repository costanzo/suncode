//! Model-facing declarations for the built-in tools.

mod conversation;
mod documents;
mod filesystem;
mod lsp;
mod network;
mod process;
mod skills;

pub use suncode_common::tool::ToolDefinition;

pub fn all() -> Vec<ToolDefinition> {
    [
        filesystem::read(),
        filesystem::glob(),
        filesystem::grep(),
        documents::inspect(),
        documents::read(),
        documents::search(),
        documents::spreadsheet(),
        lsp::diagnostics(),
        lsp::definition(),
        lsp::references(),
        lsp::hover(),
        lsp::symbols(),
        conversation::question(),
        conversation::todowrite(),
        filesystem::write(),
        filesystem::edit(),
        filesystem::delete(),
        process::bash(),
        skills::skill(),
        network::webfetch(),
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
    use super::all;
    use std::collections::BTreeSet;

    #[test]
    fn edit_single_and_batch_use_the_same_text_fields() {
        let definitions = all();
        let edit = definitions.iter().find(|tool| tool.name == "edit").unwrap();
        let properties = &edit.parameters["properties"];
        let item = &properties["edits"]["items"];
        assert_eq!(item["required"], serde_json::json!(["oldString", "newString"]));
        assert_eq!(item["additionalProperties"], false);
        assert_eq!(item["properties"].as_object().unwrap().len(), 2);
        for fields in [properties, &item["properties"]] {
            assert_eq!(fields["oldString"]["type"], "string");
            assert_eq!(fields["oldString"]["minLength"], 1);
            assert_eq!(fields["newString"]["type"], "string");
            assert!(fields.get("oldText").is_none());
            assert!(fields.get("newText").is_none());
        }
    }

    #[test]
    fn built_in_tool_names_match_the_model_contract() {
        let definitions = all();
        let names = definitions
            .iter()
            .map(|definition| definition.name.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            names,
            BTreeSet::from([
                "bash",
                "document_inspect",
                "document_read",
                "document_search",
                "edit",
                "delete",
                "glob",
                "grep",
                "lsp_definition",
                "lsp_diagnostics",
                "lsp_hover",
                "lsp_references",
                "lsp_symbols",
                "question",
                "read",
                "skill",
                "spreadsheet_read",
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
