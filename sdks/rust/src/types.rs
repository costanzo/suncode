use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::os::raw::{c_char, c_void};
use suncode_agent::domain::{
    CheckpointItem, CheckpointManifest, Message, ProjectDependencyRecord, ProjectRecord,
    ProviderExchange, SessionCallMessage, SessionCallToolUse, SessionImageRecord, SessionRecord,
    SessionTraceTurn, SettingRecord,
};
use suncode_common::BusinessError;
use suncode_llm::ModelDescriptor;

pub const SUNCODE_AGENT_SDK_ABI_VERSION: u32 = 4;
pub type SdkResult<T> = Result<T, BusinessError>;
pub type SunCodeEventCallback = unsafe extern "C" fn(*const c_char, *mut c_void);

#[derive(Debug, Serialize)]
pub struct VersionResult {
    pub version: &'static str,
}

#[derive(Debug, Serialize)]
pub struct HealthResult {
    pub ok: bool,
    pub agent: &'static str,
    pub database: Value,
}

#[derive(Debug, Serialize)]
pub struct RecoveryStatus {
    pub status: &'static str,
    pub pending_operations: usize,
}

#[derive(Debug, Serialize)]
pub struct DiagnosticsResult {
    pub health: HealthResult,
    pub recovery: RecoveryStatus,
    pub credentials: Vec<CredentialState>,
    pub active_project_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ModelsResult {
    pub models: Vec<ModelDescriptor>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CredentialState {
    pub provider: String,
    pub configured: bool,
}

#[derive(Debug, Serialize)]
pub struct CredentialsResult {
    pub credentials: Vec<CredentialState>,
}

#[derive(Debug, Serialize)]
pub struct CredentialUpdate {
    pub provider: String,
    pub configured: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderEndpointUpdate {
    pub provider_id: String,
    pub endpoint: String,
}

#[derive(Debug, Serialize)]
pub struct SettingsResult {
    pub settings: Vec<SettingRecord>,
}

#[derive(Debug, Serialize)]
pub struct SettingUpdate {
    pub saved: bool,
    pub key: String,
    pub scope: String,
    pub scope_id: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectsResult {
    pub projects: Vec<ProjectRecord>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDependencyDto {
    pub dependency_id: String,
    pub project_id: String,
    pub display_name: String,
    pub created_at: String,
}

impl From<ProjectDependencyRecord> for ProjectDependencyDto {
    fn from(value: ProjectDependencyRecord) -> Self {
        Self {
            dependency_id: value.dependency_id,
            project_id: value.project_id,
            display_name: value.display_name,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDependenciesResult {
    pub project_id: String,
    pub dependencies: Vec<ProjectDependencyDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyRemoval {
    pub dependency_id: String,
    pub removed: bool,
}

#[derive(Debug, Serialize)]
pub struct SessionsResult {
    pub project_id: String,
    pub sessions: Vec<SessionRecord>,
    #[serde(rename = "sessionStates")]
    pub session_states: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct SessionImagesResult {
    pub session_id: String,
    pub images: Vec<SessionImageRecord>,
}

#[derive(Debug, Serialize)]
pub struct SessionImageRemoval {
    pub session_id: String,
    pub image_id: String,
    pub removed: bool,
}

#[derive(Debug, Serialize)]
pub struct SessionSnapshot {
    pub session: SessionRecord,
    pub messages: Vec<Message>,
    #[serde(rename = "conversationTurns")]
    pub conversation_turns: Vec<suncode_data::SessionConversationTurn>,
    pub images: Vec<SessionImageRecord>,
    #[serde(rename = "pendingQuestion", skip_serializing_if = "Option::is_none")]
    pub pending_question: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct SessionUsageResult {
    pub session_id: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Debug, Serialize)]
pub struct ProviderExchangesResult {
    pub session_id: String,
    pub turns: Vec<SessionTraceTurn>,
    pub exchanges: Vec<ProviderExchange>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderExchangeDetails {
    #[serde(flatten)]
    pub exchange: ProviderExchange,
    pub messages: Vec<SessionCallMessage>,
    pub tool_uses: Vec<SessionCallToolUse>,
}

#[derive(Debug, Serialize)]
pub struct CheckpointsResult {
    pub session_id: String,
    pub checkpoints: Vec<CheckpointManifest>,
}

#[derive(Debug, Serialize)]
pub struct CheckpointDetails {
    pub manifest: CheckpointManifest,
    pub items: Vec<CheckpointItem>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitFileStatus {
    pub path: String,
    pub old_path: Option<String>,
    pub status: String,
    pub index_status: Option<String>,
    pub worktree_status: Option<String>,
    pub staged: bool,
    pub unstaged: bool,
    pub conflicted: bool,
    pub binary: bool,
    pub additions: u64,
    pub deletions: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitStatusResult {
    pub repository: bool,
    pub branch: Option<String>,
    pub detached: bool,
    pub head_oid: Option<String>,
    pub changed_files: usize,
    pub additions: u64,
    pub deletions: u64,
    pub conflicts: usize,
    pub files: Vec<GitFileStatus>,
    pub truncated: bool,
    pub unsupported_paths: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitDiffLine {
    pub kind: String,
    pub old_line: Option<u32>,
    pub new_line: Option<u32>,
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitDiffHunk {
    pub header: String,
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<GitDiffLine>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitDiffFileResult {
    pub scope: String,
    pub path: String,
    pub old_path: Option<String>,
    pub status: String,
    pub binary: bool,
    pub additions: usize,
    pub deletions: usize,
    pub hunks: Vec<GitDiffHunk>,
    pub patch: String,
    pub truncated: bool,
}

#[derive(Debug, Serialize)]
pub struct RestoreOutcome {
    pub manifest_id: String,
    pub status: &'static str,
    pub restored_items: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AddSessionImageRequest {
    pub(crate) display_name: String,
    pub(crate) source_kind: String,
    pub(crate) original_path: Option<String>,
    pub(crate) extension: String,
    pub(crate) bytes_base64: String,
    pub(crate) thumbnail_base64: String,
}

#[derive(Debug, Serialize)]
pub struct CancellationOutcome {
    pub turn_id: String,
    pub status: &'static str,
}

#[derive(Debug, Serialize)]
pub struct ApprovalOutcome {
    pub approval_id: String,
    pub decision: String,
}

#[derive(Debug, Serialize)]
pub struct QuestionOutcome {
    pub request_id: String,
    pub status: String,
}
