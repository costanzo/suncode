use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A model-facing tool declaration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}
