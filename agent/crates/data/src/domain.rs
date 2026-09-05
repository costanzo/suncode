use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LlmModelProviderRecord {
    pub provider_id: String,
    pub display_name: String,
    pub endpoint: String,
    pub default_endpoint: String,
    pub adapter_type: String,
    pub api_key_configured: bool,
    pub enabled: bool,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LlmModelRecord {
    pub model_id: String,
    pub provider_id: String,
    pub display_name: String,
    pub request_model: String,
    pub context_tokens: u64,
    pub auto_compact_tokens: u64,
    pub max_output_tokens: Option<u64>,
    pub supports_streaming: bool,
    pub supports_tool_use: bool,
    pub supports_vision: bool,
    pub supports_structured_output: bool,
    pub supports_cancellation: bool,
    pub supports_reasoning_effort: bool,
    /// Comma-separated values persisted in SQLite, exposed as normalized efforts.
    pub reasoning_efforts: Vec<String>,
    pub enabled: bool,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct LlmModelProviderInput<'a> {
    pub provider_id: &'a str,
    pub display_name: &'a str,
    pub endpoint: &'a str,
    pub default_endpoint: &'a str,
    pub adapter_type: &'a str,
    pub enabled: bool,
    pub sort_order: i64,
}

#[derive(Debug, Clone)]
pub struct LlmModelInput<'a> {
    pub model_id: &'a str,
    pub provider_id: &'a str,
    pub display_name: &'a str,
    pub request_model: &'a str,
    pub context_tokens: u64,
    pub auto_compact_tokens: u64,
    pub max_output_tokens: Option<u64>,
    pub supports_streaming: bool,
    pub supports_tool_use: bool,
    pub supports_vision: bool,
    pub supports_structured_output: bool,
    pub supports_cancellation: bool,
    pub supports_reasoning_effort: bool,
    pub reasoning_efforts: &'a str,
    pub enabled: bool,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContentPart {
    #[serde(rename = "type")]
    pub kind: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    pub call_id: String,
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub role: String,
    pub content: Vec<ContentPart>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn text(role: &str, text: impl Into<String>) -> Self {
        Self {
            role: role.to_string(),
            content: vec![ContentPart {
                kind: "text".to_string(),
                text: text.into(),
            }],
            tool_calls: Vec::new(),
            tool_call_id: None,
        }
    }

    pub fn text_content(&self) -> String {
        self.content
            .iter()
            .filter(|part| part.kind == "text")
            .map(|part| part.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

impl Usage {
    pub fn add(&mut self, other: &Usage) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.total_tokens += other.total_tokens;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    pub session_id: String,
    pub occurred_at: String,
    pub event_type: String,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderExchange {
    pub exchange_id: String,
    pub session_id: String,
    pub turn_id: String,
    pub provider: String,
    pub model_id: String,
    pub wire_model: String,
    pub provider_request_id: Option<String>,
    pub provider_response_id: Option<String>,
    pub state: String,
    pub iteration: i64,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub input_messages: Value,
    pub output_message: Option<Value>,
    pub tool_calls: Value,
    pub usage: Option<Value>,
    pub finish_reason: Option<String>,
    pub error: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionTraceTurn {
    pub turn_id: String,
    pub session_id: String,
    pub state: String,
    pub model_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub error_code: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCallMessage {
    pub message_id: String,
    pub session_id: String,
    pub turn_id: Option<String>,
    pub session_call_id: Option<String>,
    pub role: String,
    pub message: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCallToolUse {
    pub turn_id: String,
    pub tool_call_id: String,
    pub session_call_id: Option<String>,
    pub name: String,
    pub request: Option<Value>,
    pub result: Option<Value>,
    pub state: String,
    pub ordinal: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionTurnTodo {
    pub turn_id: String,
    pub ordinal: i64,
    pub content: String,
    pub status: String,
    pub priority: String,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionConversationTurn {
    pub turn_id: String,
    pub state: String,
    pub created_at: String,
    pub messages: Vec<SessionCallMessage>,
    pub tool_uses: Vec<SessionCallToolUse>,
    pub todos: Vec<SessionTurnTodo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SessionImageRecord {
    pub image_id: String,
    pub session_id: String,
    pub display_name: String,
    pub source_kind: String,
    pub original_path: Option<String>,
    pub storage_path: String,
    pub thumbnail_base64: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRecord {
    pub project_id: String,
    pub canonical_root: String,
    pub display_name: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_opened_at: String,
    pub archived_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDependencyRecord {
    pub dependency_id: String,
    pub project_id: String,
    pub canonical_root: String,
    pub display_name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRecord {
    pub session_id: String,
    pub project_id: Option<String>,
    pub title: Option<String>,
    pub model_id: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_activity_at: String,
    pub archived_at: Option<String>,
    pub pin_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalRecord {
    pub approval_id: String,
    pub project_id: Option<String>,
    pub session_id: String,
    pub turn_id: String,
    pub tool_call_id: String,
    pub operation: String,
    pub arguments: Value,
    pub status: String,
    pub decision: Option<String>,
    pub decision_source: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspendedTurn {
    pub approval_id: String,
    pub session_id: String,
    pub turn_id: String,
    pub snapshot: Value,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointManifest {
    pub manifest_id: String,
    pub session_id: String,
    pub turn_id: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub expires_at: String,
    pub restored_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointItem {
    pub checkpoint_id: String,
    pub manifest_id: Option<String>,
    pub session_id: String,
    pub turn_id: Option<String>,
    pub tool_call_id: Option<String>,
    pub relative_path: Option<String>,
    pub status: String,
    pub created_at: String,
    pub restored_at: Option<String>,
    pub invalidated_at: Option<String>,
    pub ordinal: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct TurnAdmission {
    pub created: bool,
    pub turn_id: String,
    pub status: String,
    pub response: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SettingRecord {
    pub key: String,
    pub value: Value,
    pub scope: String,
    pub scope_id: String,
}
