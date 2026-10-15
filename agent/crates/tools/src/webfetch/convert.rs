//! HTML to Markdown or plain-text conversion.

use super::{fetch_failure, OutputFormat};
use html2md_rs::{
    parser::safe_parse_html,
    structs::{Node, NodeType, ToMdConfig},
    to_md::to_md_with_config,
};
use suncode_common::BusinessError;

pub(super) fn convert_html(input: &str, format: OutputFormat) -> Result<String, BusinessError> {
    if format == OutputFormat::Html {
        return Ok(input.to_string());
    }
    let root = safe_parse_html(input.to_string())
        .map_err(|_| fetch_failure("HTML response could not be parsed", false))?;
    if format == OutputFormat::Markdown {
        let config = ToMdConfig {
            ignore_rendering: ignored_nodes(),
        };
        return Ok(to_md_with_config(root, &config).trim().to_string());
    }
    Ok(extract_text(&root))
}

fn ignored_nodes() -> Vec<NodeType> {
    vec![
        NodeType::Head,
        NodeType::Style,
        NodeType::Link,
        NodeType::Script,
        NodeType::Meta,
        NodeType::Title,
        NodeType::Unknown("noscript".into()),
        NodeType::Unknown("iframe".into()),
        NodeType::Unknown("object".into()),
        NodeType::Unknown("embed".into()),
    ]
}

fn extract_text(root: &Node) -> String {
    fn walk(node: &Node, output: &mut String) {
        let ignored = node
            .tag_name
            .as_ref()
            .is_some_and(|tag| ignored_nodes().contains(tag));
        if ignored {
            return;
        }
        if node.tag_name == Some(NodeType::Text) {
            if let Some(value) = node.value.as_deref() {
                let decoded = html_escape::decode_html_entities(value);
                append_words(output, &decoded);
            }
            return;
        }
        let block = node.tag_name.as_ref().is_some_and(|tag| {
            matches!(
                tag,
                NodeType::Body
                    | NodeType::Div
                    | NodeType::P
                    | NodeType::H1
                    | NodeType::H2
                    | NodeType::H3
                    | NodeType::H4
                    | NodeType::H5
                    | NodeType::H6
                    | NodeType::Li
                    | NodeType::Pre
                    | NodeType::Blockquote
                    | NodeType::Br
                    | NodeType::Hr
            )
        });
        if block {
            append_break(output);
        }
        for child in &node.children {
            walk(child, output);
        }
        if block {
            append_break(output);
        }
    }

    let mut output = String::new();
    walk(root, &mut output);
    output.trim().to_string()
}

fn append_words(output: &mut String, value: &str) {
    for word in value.split_whitespace() {
        if !output.is_empty() && !output.ends_with([' ', '\n']) {
            output.push(' ');
        }
        output.push_str(word);
    }
}

fn append_break(output: &mut String) {
    while output.ends_with(' ') {
        output.pop();
    }
    if !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }
}
