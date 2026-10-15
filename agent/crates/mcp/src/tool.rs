use crate::error::mcp_error;
use rmcp::model::{CallToolResult, ContentBlock, Tool};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use suncode_common::BusinessError;

const MAX_RESULT_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    pub remote_name: String,
    pub title: Option<String>,
    pub description: String,
    pub input_schema: Value,
}

impl ToolDefinition {
    pub(crate) fn from_rmcp(tool: Tool) -> Result<Self, BusinessError> {
        let input_schema = Value::Object((*tool.input_schema).clone());
        if input_schema.get("type").and_then(Value::as_str) != Some("object") {
            return Err(mcp_error(
                "mcp_schema_invalid",
                "MCP tool input schema must describe an object",
            ));
        }
        Ok(Self {
            remote_name: tool.name.into_owned(),
            title: tool.title,
            description: tool
                .description
                .map(|value| value.into_owned())
                .unwrap_or_default(),
            input_schema,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ToolResult {
    pub content: Vec<String>,
    pub structured_content: Option<Value>,
    pub is_error: bool,
    pub truncated: bool,
}

impl ToolResult {
    /// Accepts text-only results and truncates them to `MAX_RESULT_BYTES`.
    pub(crate) fn from_rmcp(result: CallToolResult) -> Result<Self, BusinessError> {
        let mut content = Vec::new();
        for block in result.content {
            match block {
                ContentBlock::Text(text) => content.push(text.text),
                _ => {
                    return Err(mcp_error(
                        "mcp_content_unsupported",
                        "MCP tool returned unsupported non-text content",
                    ))
                }
            }
        }
        let mut normalized = Self {
            content,
            structured_content: result.structured_content,
            is_error: result.is_error.unwrap_or(false),
            truncated: false,
        };
        let encoded = serde_json::to_vec(&normalized)
            .map_err(|_| mcp_error("mcp_result_invalid", "MCP result could not be encoded"))?;
        if encoded.len() > MAX_RESULT_BYTES {
            normalized.truncate();
        }
        Ok(normalized)
    }

    fn truncate(&mut self) {
        let mut remaining = MAX_RESULT_BYTES / 2;
        for value in &mut self.content {
            if value.len() > remaining {
                let mut boundary = remaining.min(value.len());
                while !value.is_char_boundary(boundary) {
                    boundary -= 1;
                }
                value.truncate(boundary);
            }
            remaining = remaining.saturating_sub(value.len());
        }
        self.structured_content = self
            .structured_content
            .take()
            .map(|_| json!({"truncated": true}));
        self.truncated = true;
    }
}

pub fn tool_result_value(result: ToolResult) -> Value {
    let mut value = Map::new();
    value.insert(
        "content".into(),
        Value::Array(result.content.into_iter().map(Value::String).collect()),
    );
    if let Some(structured) = result.structured_content {
        value.insert("structuredContent".into(), structured);
    }
    value.insert("isError".into(), Value::Bool(result.is_error));
    value.insert("truncated".into(), Value::Bool(result.truncated));
    Value::Object(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_normalization_rejects_binary_content() {
        let result = CallToolResult::success(vec![ContentBlock::image("aGVsbG8=", "image/png")]);
        assert_eq!(
            ToolResult::from_rmcp(result).unwrap_err().code,
            "mcp_content_unsupported"
        );
    }
}
