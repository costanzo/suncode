use crate::logging::{self, Level};
use crate::{
    agent::{Agent, AgentError, TurnResponse},
    config::Config,
    credentials::{CredentialState, CredentialStore},
    domain::{
        ApprovalRecord, CheckpointItem, CheckpointManifest, Message, ProjectDependencyRecord,
        ProjectRecord, ProviderExchange, SessionCallMessage, SessionCallToolUse, SessionEvent,
        SessionRecord, SessionTraceTurn, SettingRecord,
    },
    runtime_lock::RuntimeLock,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    ffi::{CStr, CString},
    os::raw::{c_char, c_void},
    panic::{catch_unwind, AssertUnwindSafe},
    path::{Path, PathBuf},
    ptr,
    sync::{Arc, Mutex},
    thread::JoinHandle,
};
use suncode_db::{PersistenceError, Store};
use suncode_llm::{
    ModelCapabilities, ModelDescriptor, ModelLimits, ModelProviderRegistry,
    OpenAiCompatibleProvider, RegistrationError,
};
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

pub const SUNCODE_RUNTIME_SDK_ABI_VERSION: u32 = 1;

#[derive(Clone)]
struct RuntimeState {
    store: Store,
    operations: Arc<suncode_tool::Operations>,
    active_project: Arc<Mutex<Option<String>>>,
    events: broadcast::Sender<SessionEvent>,
    credentials: CredentialStore,
    agent: Agent,
    providers: Arc<ModelProviderRegistry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SdkError {
    pub code: String,
    pub message: String,
    pub details: Value,
}

impl SdkError {
    fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
            details: json!({}),
        }
    }

    fn invalid(message: impl Into<String>) -> Self {
        Self::new("invalid_arguments", message)
    }

    fn unavailable(message: impl Into<String>) -> Self {
        Self::new("runtime_unavailable", message)
    }

    fn missing(kind: &str) -> Self {
        Self::new(&format!("{kind}_not_found"), format!("{kind} not found"))
    }
}

impl std::fmt::Display for SdkError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for SdkError {}

impl From<PersistenceError> for SdkError {
    fn from(error: PersistenceError) -> Self {
        Self::unavailable(error.to_string())
    }
}

impl From<AgentError> for SdkError {
    fn from(error: AgentError) -> Self {
        Self {
            code: error.code,
            message: error.message,
            details: error.details,
        }
    }
}

pub type SdkResult<T> = Result<T, SdkError>;

#[derive(Debug, Serialize)]
pub struct HealthResult {
    pub ok: bool,
    pub runtime: &'static str,
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
}

#[derive(Debug, Serialize)]
pub struct SessionSnapshot {
    pub session: SessionRecord,
    pub messages: Vec<Message>,
    #[serde(rename = "conversationTurns")]
    pub conversation_turns: Vec<suncode_db::SessionConversationTurn>,
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

fn registry_from_store(
    store: &Store,
    keys: Arc<dyn suncode_llm::ApiKeyResolver>,
) -> SdkResult<ModelProviderRegistry> {
    let providers = store.llm_model_providers(true)?;
    let models = store.llm_models(true)?;
    let mut registry = ModelProviderRegistry::new();
    for provider in providers {
        let provider_models = models
            .iter()
            .filter(|model| model.provider_id == provider.provider_id)
            .map(|model| ModelDescriptor {
                provider: provider.provider_id.clone(),
                provider_label: provider.display_name.clone(),
                id: model.model_id.clone(),
                wire_model: model.request_model.clone(),
                api_base: provider.endpoint.clone(),
                capabilities: ModelCapabilities {
                    streaming: model.supports_streaming,
                    tool_use: model.supports_tool_use,
                    vision: model.supports_vision,
                    structured_output: model.supports_structured_output,
                    cancellation: model.supports_cancellation,
                },
                limits: ModelLimits {
                    max_input_tokens: Some(model.context_tokens),
                    auto_compact_tokens: Some(model.auto_compact_tokens),
                    max_output_tokens: model.max_output_tokens,
                },
                availability: "configured".into(),
            })
            .collect::<Vec<_>>();
        if provider_models.is_empty() {
            continue;
        }
        let adapter = match provider.adapter_type.as_str() {
            "openai" => Arc::new(OpenAiCompatibleProvider::new(
                provider.provider_id.clone(),
                provider.display_name,
                provider.endpoint,
                keys.clone(),
            )),
            adapter_type => {
                return Err(SdkError::new(
                    "provider_adapter_unsupported",
                    format!("provider adapter is not supported: {adapter_type}"),
                ));
            }
        };
        registry
            .register(provider.provider_id, adapter, provider_models)
            .map_err(|error| SdkError::new("provider_registration_failed", error.to_string()))?;
    }
    Ok(registry)
}

async fn build_state<F>(config: &Config, configure_providers: F) -> SdkResult<RuntimeState>
where
    F: FnOnce(&mut ModelProviderRegistry) -> Result<(), RegistrationError>,
{
    let store = Store::open(&config.database_path).map_err(SdkError::from)?;
    configure_logging(&store, &config.data_dir)?;
    let operations = Arc::new(
        suncode_tool::Operations::new(config.data_dir.join("operations"))
            .map_err(|error| SdkError::unavailable(error.to_string()))?,
    );
    let (events, _) = broadcast::channel(256);
    let credentials = CredentialStore::load(store.clone(), config.non_interactive);
    let mut providers = registry_from_store(&store, Arc::new(credentials.clone()))?;
    configure_providers(&mut providers)
        .map_err(|error| SdkError::new("provider_registration_failed", error.to_string()))?;
    let providers = Arc::new(providers);
    let agent = Agent::new(
        store.clone(),
        providers.clone(),
        operations.clone(),
        events.clone(),
        config.non_interactive,
    );
    let state = RuntimeState {
        store,
        operations,
        active_project: Arc::new(Mutex::new(None)),
        events,
        credentials,
        agent,
        providers,
    };
    state.agent.recover().await.map_err(SdkError::from)?;
    Ok(state)
}

fn configure_logging(store: &Store, data_dir: &Path) -> SdkResult<()> {
    let settings = store.settings(None, None)?;
    let value = |key: &str| {
        settings
            .iter()
            .find(|record| record.key == key)
            .map(|record| &record.value)
    };
    let level = value("log_level").and_then(Value::as_str).unwrap_or("INFO");
    let directory = value("log_directory").and_then(Value::as_str);
    let max_bytes = value("log_max_bytes")
        .and_then(Value::as_u64)
        .unwrap_or(10 * 1024 * 1024);
    let retention = value("log_retention")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(5);
    logging::configure(
        data_dir,
        logging::Config {
            level,
            directory,
            max_bytes,
            retention,
        },
    );
    Ok(())
}

fn validate_logging_setting(scope: &str, key: &str, value: &Value) -> SdkResult<()> {
    if key == "full_control" {
        if scope != "session" {
            return Err(SdkError::invalid("full_control is a session-only setting"));
        }
        if !value.is_boolean() {
            return Err(SdkError::invalid("full_control must be a boolean"));
        }
        return Ok(());
    }
    let is_logging_setting = matches!(
        key,
        "log_level" | "log_directory" | "log_max_bytes" | "log_retention"
    );
    if !is_logging_setting {
        return Ok(());
    }
    if scope != "global" {
        return Err(SdkError::invalid(format!("{key} is a global-only setting")));
    }
    match key {
        "log_level" => {
            let Some(level) = value.as_str() else {
                return Err(SdkError::invalid("log_level must be a string"));
            };
            if !matches!(
                level.trim().to_ascii_uppercase().as_str(),
                "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR" | "OFF"
            ) {
                return Err(SdkError::invalid(
                    "log_level must be TRACE, DEBUG, INFO, WARN, ERROR, or OFF",
                ));
            }
        }
        "log_directory" if !value.is_string() => {
            return Err(SdkError::invalid("log_directory must be a string"));
        }
        "log_max_bytes" if value.as_u64().is_none_or(|size| size < 1024) => {
            return Err(SdkError::invalid(
                "log_max_bytes must be an integer greater than or equal to 1024",
            ));
        }
        "log_retention" if value.as_u64().is_none_or(|count| count > 100) => {
            return Err(SdkError::invalid(
                "log_retention must be an integer between 0 and 100",
            ));
        }
        _ => {}
    }
    Ok(())
}

pub struct RuntimeSdk {
    _lock: Option<RuntimeLock>,
    data_dir: PathBuf,
    runtime: tokio::runtime::Runtime,
    state: RuntimeState,
}

impl RuntimeSdk {
    pub fn open_default() -> SdkResult<Self> {
        Self::open_default_with_providers(|_| Ok(()))
    }

    /// Opens the runtime after extending the built-in registry with trusted providers.
    pub fn open_default_with_providers<F>(configure_providers: F) -> SdkResult<Self>
    where
        F: FnOnce(&mut ModelProviderRegistry) -> Result<(), RegistrationError>,
    {
        let config = Config::load().map_err(SdkError::invalid)?;
        let lock = RuntimeLock::acquire(&config.data_dir).map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                SdkError::new("runtime_already_active", error.to_string())
            } else {
                SdkError::unavailable(format!("runtime lock unavailable: {error}"))
            }
        })?;
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("suncode-sdk")
            .build()
            .map_err(|error| {
                SdkError::unavailable(format!("tokio runtime unavailable: {error}"))
            })?;
        let state = runtime.block_on(build_state(&config, configure_providers))?;
        logging::write(Level::Info, "runtime", "open completed");
        Ok(Self {
            _lock: Some(lock),
            data_dir: config.data_dir,
            runtime,
            state,
        })
    }

    pub fn health(&self) -> SdkResult<HealthResult> {
        Ok(HealthResult {
            ok: true,
            runtime: "ready",
            database: self.state.store.health()?,
        })
    }

    pub fn diagnostics(&self) -> SdkResult<DiagnosticsResult> {
        Ok(DiagnosticsResult {
            health: self.health()?,
            recovery: RecoveryStatus {
                status: "ready",
                pending_operations: 0,
            },
            credentials: self.state.credentials.state(),
            active_project_id: self
                .state
                .active_project
                .lock()
                .ok()
                .and_then(|value| value.clone()),
        })
    }

    pub fn list_models(&self) -> SdkResult<ModelsResult> {
        let mut models = self.state.providers.models();
        for model in &mut models {
            model.availability = if self.state.credentials.configured(&model.provider) {
                "configured".into()
            } else {
                "unconfigured".into()
            };
        }
        Ok(ModelsResult { models })
    }

    pub fn list_credentials(&self) -> SdkResult<CredentialsResult> {
        Ok(CredentialsResult {
            credentials: self.state.credentials.state(),
        })
    }

    pub fn set_credential(&self, provider: &str, api_key: &str) -> SdkResult<CredentialUpdate> {
        if !self
            .state
            .credentials
            .state()
            .iter()
            .any(|state| state.provider == provider)
        {
            return Err(SdkError::invalid("provider is not supported"));
        }
        let provider = provider.to_string();
        self.state
            .credentials
            .set(&provider, api_key)
            .map_err(|error| SdkError::new("credential_unavailable", error))?;
        Ok(CredentialUpdate {
            provider,
            configured: true,
        })
    }

    pub fn remove_credential(&self, provider: &str) -> SdkResult<CredentialUpdate> {
        if !self
            .state
            .credentials
            .state()
            .iter()
            .any(|state| state.provider == provider)
        {
            return Err(SdkError::invalid("provider is not supported"));
        }
        let provider = provider.to_string();
        self.state
            .credentials
            .delete(&provider)
            .map_err(|error| SdkError::new("credential_unavailable", error))?;
        Ok(CredentialUpdate {
            provider,
            configured: false,
        })
    }

    pub fn list_settings(
        &self,
        project_id: Option<&str>,
        session_id: Option<&str>,
    ) -> SdkResult<SettingsResult> {
        Ok(SettingsResult {
            settings: self.state.store.settings(project_id, session_id)?,
        })
    }

    pub fn set_setting(
        &self,
        scope: &str,
        project_id: Option<&str>,
        session_id: Option<&str>,
        key: &str,
        value: &Value,
    ) -> SdkResult<SettingUpdate> {
        validate_logging_setting(scope, key, value)?;
        let scope_id = match scope {
            "global" => "global",
            "project" => project_id.ok_or_else(|| SdkError::invalid("project_id is required"))?,
            "session" => session_id.ok_or_else(|| SdkError::invalid("session_id is required"))?,
            _ => {
                return Err(SdkError::invalid(
                    "scope must be global, project, or session",
                ))
            }
        };
        self.state.store.set_setting(scope, scope_id, key, value)?;
        if scope == "session" && key == "full_control" {
            self.state.store.append_audit(
                None,
                Some(scope_id),
                None,
                "session.full_control.changed",
                &json!({"enabled": value.as_bool().unwrap_or(false), "source": "user"}),
            )?;
        }
        if scope == "global"
            && matches!(
                key,
                "log_level" | "log_directory" | "log_max_bytes" | "log_retention"
            )
        {
            configure_logging(&self.state.store, &self.data_dir)?;
        }
        Ok(SettingUpdate {
            saved: true,
            key: key.to_string(),
            scope: scope.to_string(),
            scope_id: scope_id.to_string(),
        })
    }

    pub fn list_projects(&self) -> SdkResult<ProjectsResult> {
        Ok(ProjectsResult {
            projects: self.state.store.projects(false)?,
        })
    }

    pub fn open_project(&self, path: &str, display_name: Option<&str>) -> SdkResult<ProjectRecord> {
        let result = self
            .state
            .operations
            .open_project(std::path::Path::new(path))
            .map_err(operation_error)?;
        let root = result
            .get("canonical_path")
            .and_then(Value::as_str)
            .ok_or_else(|| SdkError::unavailable("project/open did not return a canonical path"))?;
        let display_name = display_name
            .or_else(|| result.get("display_name").and_then(Value::as_str))
            .unwrap_or("Project");
        let project = self.state.store.project(root, display_name)?;
        if let Ok(mut active) = self.state.active_project.lock() {
            *active = Some(project.project_id.clone());
        }
        Ok(project)
    }

    pub fn select_project(&self, project_id: &str) -> SdkResult<ProjectRecord> {
        let project = self
            .state
            .store
            .project_by_id(project_id)?
            .ok_or_else(|| SdkError::missing("project"))?;
        self.state
            .operations
            .open_project(std::path::Path::new(&project.canonical_root))
            .map_err(operation_error)?;
        if let Ok(mut active) = self.state.active_project.lock() {
            *active = Some(project.project_id.clone());
        }
        Ok(project)
    }

    pub fn list_project_dependencies(
        &self,
        project_id: &str,
    ) -> SdkResult<ProjectDependenciesResult> {
        if self.state.store.project_by_id(project_id)?.is_none() {
            return Err(SdkError::missing("project"));
        }
        Ok(ProjectDependenciesResult {
            project_id: project_id.to_string(),
            dependencies: self
                .state
                .store
                .project_dependencies(project_id)?
                .into_iter()
                .map(Into::into)
                .collect(),
        })
    }

    pub fn add_project_dependency(
        &self,
        project_id: &str,
        path: &str,
    ) -> SdkResult<ProjectDependencyDto> {
        let project = self
            .state
            .store
            .project_by_id(project_id)?
            .ok_or_else(|| SdkError::missing("project"))?;
        let opened = self
            .state
            .operations
            .open_project(Path::new(path))
            .map_err(operation_error)?;
        let canonical_root = opened
            .get("canonical_path")
            .and_then(Value::as_str)
            .ok_or_else(|| SdkError::unavailable("dependency root was not canonicalized"))?;
        let display_name = opened
            .get("display_name")
            .and_then(Value::as_str)
            .unwrap_or("Dependency");
        let dependency_root = Path::new(canonical_root);
        let project_root = Path::new(&project.canonical_root);
        if dependency_root.starts_with(project_root) || project_root.starts_with(dependency_root) {
            return Err(SdkError::invalid(
                "dependency root must not equal, contain, or be contained by the project root",
            ));
        }
        if self
            .state
            .store
            .project_dependencies(project_id)?
            .iter()
            .map(|dependency| Path::new(&dependency.canonical_root))
            .any(|existing| {
                dependency_root.starts_with(existing) || existing.starts_with(dependency_root)
            })
        {
            return Err(SdkError::invalid(
                "dependency roots must not equal, contain, or be contained by each other",
            ));
        }
        self.state
            .store
            .add_project_dependency(project_id, canonical_root, display_name)
            .map(Into::into)
            .map_err(SdkError::from)
    }

    pub fn remove_project_dependency(
        &self,
        project_id: &str,
        dependency_id: &str,
    ) -> SdkResult<DependencyRemoval> {
        let removed = self
            .state
            .store
            .remove_project_dependency(project_id, dependency_id)?;
        if !removed {
            return Err(SdkError::missing("dependency"));
        }
        Ok(DependencyRemoval {
            dependency_id: dependency_id.to_string(),
            removed,
        })
    }

    pub fn list_project_directory(
        &self,
        project_id: &str,
        dependency_id: Option<&str>,
        path: &str,
    ) -> SdkResult<Value> {
        let root = if let Some(dependency_id) = dependency_id {
            self.state
                .store
                .project_dependency_by_id(project_id, dependency_id)?
                .ok_or_else(|| SdkError::missing("dependency"))?
                .canonical_root
        } else {
            self.state
                .store
                .project_by_id(project_id)?
                .ok_or_else(|| SdkError::missing("project"))?
                .canonical_root
        };
        let mut value = self
            .state
            .operations
            .list_directory(Path::new(&root), path, 500)
            .map_err(operation_error)?;
        value["projectId"] = json!(project_id);
        value["dependencyId"] = dependency_id.map_or(Value::Null, |value| json!(value));
        Ok(value)
    }

    pub fn list_sessions(&self, project_id: &str) -> SdkResult<SessionsResult> {
        if self.state.store.project_by_id(project_id)?.is_none() {
            return Err(SdkError::missing("project"));
        }
        Ok(SessionsResult {
            project_id: project_id.to_string(),
            sessions: self.state.store.sessions_for_project(project_id, true)?,
        })
    }

    pub fn git_status(&self, project_id: &str) -> SdkResult<GitStatusResult> {
        let project = self
            .state
            .store
            .project_by_id(project_id)?
            .ok_or_else(|| SdkError::missing("project"))?;
        let value = self
            .state
            .operations
            .execute_in_project(
                std::path::Path::new(&project.canonical_root),
                "git/status",
                json!({}),
            )
            .map_err(operation_error)?;
        decode_operation(value, "git/status")
    }

    pub fn git_diff_file(
        &self,
        project_id: &str,
        scope: &str,
        path: &str,
    ) -> SdkResult<GitDiffFileResult> {
        if !matches!(scope, "all" | "staged" | "unstaged") {
            return Err(SdkError::invalid("scope must be all, staged, or unstaged"));
        }
        if path.trim().is_empty() {
            return Err(SdkError::invalid("path is required"));
        }
        let project = self
            .state
            .store
            .project_by_id(project_id)?
            .ok_or_else(|| SdkError::missing("project"))?;
        let value = self
            .state
            .operations
            .execute_in_project(
                std::path::Path::new(&project.canonical_root),
                "git/diff-file",
                json!({"scope": scope, "path": path}),
            )
            .map_err(operation_error)?;
        decode_operation(value, "git/diff-file")
    }

    pub fn create_session(
        &self,
        project_id: &str,
        title: Option<&str>,
        model: Option<&str>,
    ) -> SdkResult<SessionRecord> {
        let selected_model = match model {
            Some(model) => Some(model.to_string()),
            None => self.state.store.project_default_model(project_id)?,
        };
        if let Some(model) = selected_model.as_deref() {
            if self.state.providers.route(model).is_none() {
                return Err(SdkError::new(
                    "model_unavailable",
                    "model is not advertised",
                ));
            }
        }
        self.state
            .store
            .create_session(project_id, title, selected_model.as_deref())
            .map_err(SdkError::from)
    }

    pub fn rename_session(&self, session_id: &str, title: &str) -> SdkResult<SessionRecord> {
        if title.trim().is_empty() {
            return Err(SdkError::invalid("title is required"));
        }
        self.state
            .store
            .rename_session(session_id, title.trim())
            .map_err(SdkError::from)
    }

    pub fn archive_session(&self, session_id: &str) -> SdkResult<SessionRecord> {
        self.state
            .store
            .set_session_archived(session_id, true)
            .map_err(SdkError::from)
    }

    pub fn set_session_pinned(&self, session_id: &str, pinned: bool) -> SdkResult<SessionRecord> {
        self.state
            .store
            .set_session_pinned(session_id, pinned)
            .map_err(SdkError::from)
    }

    pub fn reopen_session(&self, session_id: &str) -> SdkResult<SessionRecord> {
        self.state
            .store
            .set_session_archived(session_id, false)
            .map_err(SdkError::from)
    }

    pub fn session_snapshot(&self, session_id: &str, _after: i64) -> SdkResult<SessionSnapshot> {
        logging::write(
            Level::Debug,
            "session_snapshot",
            format!("begin session={session_id}"),
        );
        let session = self
            .state
            .store
            .session_by_id(session_id)?
            .ok_or_else(|| SdkError::missing("session"))?;
        let messages = self.state.store.messages(session_id)?;
        let conversation_turns = self.state.store.session_conversation_turns(session_id)?;
        logging::write(
            Level::Debug,
            "session_snapshot",
            format!("end session={session_id} messages={}", messages.len()),
        );
        Ok(SessionSnapshot {
            session,
            messages,
            conversation_turns,
        })
    }

    pub fn session_usage(&self, session_id: &str) -> SdkResult<SessionUsageResult> {
        if self.state.store.session_by_id(session_id)?.is_none() {
            return Err(SdkError::missing("session"));
        }
        let usage = self.state.store.session_usage(session_id)?;
        Ok(SessionUsageResult {
            session_id: session_id.to_string(),
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
            total_tokens: usage.total_tokens,
        })
    }

    pub fn list_provider_exchanges(&self, session_id: &str) -> SdkResult<ProviderExchangesResult> {
        if self.state.store.session_by_id(session_id)?.is_none() {
            return Err(SdkError::missing("session"));
        }
        Ok(ProviderExchangesResult {
            session_id: session_id.to_string(),
            turns: self.state.store.session_trace_turns(session_id)?,
            exchanges: self.state.store.provider_exchanges(session_id)?,
        })
    }

    pub fn provider_exchange(
        &self,
        session_id: &str,
        exchange_id: &str,
    ) -> SdkResult<ProviderExchangeDetails> {
        if exchange_id.trim().is_empty() {
            return Err(SdkError::invalid("exchange_id is required"));
        }
        let exchange = self
            .state
            .store
            .provider_exchange(session_id, exchange_id)?
            .ok_or_else(|| SdkError::missing("provider_exchange"))?;
        Ok(ProviderExchangeDetails {
            messages: self
                .state
                .store
                .session_call_messages(session_id, exchange_id)?,
            tool_uses: self.state.store.session_call_tool_uses(exchange_id)?,
            exchange,
        })
    }

    pub fn list_checkpoints(&self, session_id: &str) -> SdkResult<CheckpointsResult> {
        Ok(CheckpointsResult {
            session_id: session_id.to_string(),
            checkpoints: self.state.store.manifests(session_id)?,
        })
    }

    pub fn checkpoint_manifest(&self, manifest_id: &str) -> SdkResult<CheckpointDetails> {
        let manifest = self
            .state
            .store
            .manifest(manifest_id)?
            .ok_or_else(|| SdkError::missing("checkpoint"))?;
        Ok(CheckpointDetails {
            manifest,
            items: self.state.store.checkpoint_items(manifest_id)?,
        })
    }

    pub fn restore_checkpoint(
        &self,
        manifest_id: &str,
        session_id: &str,
    ) -> SdkResult<RestoreOutcome> {
        let manifest = self
            .state
            .store
            .manifest(manifest_id)?
            .ok_or_else(|| SdkError::missing("checkpoint"))?;
        if manifest.session_id != session_id {
            return Err(SdkError::new(
                "scope_denied",
                "checkpoint does not belong to session",
            ));
        }
        if manifest.status != "available" {
            return Err(SdkError::new(
                "checkpoint_unavailable",
                "checkpoint is not available",
            ));
        }
        let session = self
            .state
            .store
            .session_by_id(session_id)?
            .ok_or_else(|| SdkError::missing("session"))?;
        let project = self
            .state
            .store
            .project_by_id(session.project_id.as_deref().unwrap_or(""))?
            .ok_or_else(|| SdkError::missing("project"))?;
        self.state
            .store
            .set_manifest_status(manifest_id, "restoring")?;
        let items = self.state.store.checkpoint_items(manifest_id)?;
        let mut restored = 0;
        for item in &items {
            if item.status != "available" {
                continue;
            }
            match self.state.operations.execute_in_project(
                std::path::Path::new(&project.canonical_root),
                "checkpoint/restore",
                json!({"checkpoint_id": item.checkpoint_id}),
            ) {
                Ok(result) => {
                    restored += 1;
                    emit_event(
                        &self.state,
                        session_id,
                        "checkpoint.item_restored",
                        json!({
                            "manifest_id": manifest_id,
                            "checkpoint_id": item.checkpoint_id,
                            "path": result.get("path")
                        }),
                    )?;
                }
                Err(error) => {
                    let status = if restored > 0 { "partial" } else { "conflict" };
                    self.state.store.set_manifest_status(manifest_id, status)?;
                    emit_event(
                        &self.state,
                        session_id,
                        "checkpoint.restore_failed",
                        json!({"manifest_id": manifest_id, "status": status, "code": error.get("code")}),
                    )?;
                    return Err(SdkError::new(
                        error
                            .get("code")
                            .and_then(Value::as_str)
                            .unwrap_or("restore_conflict"),
                        error
                            .get("message")
                            .and_then(Value::as_str)
                            .unwrap_or("checkpoint restore failed"),
                    ));
                }
            }
        }
        self.state
            .store
            .set_manifest_status(manifest_id, "restored")?;
        emit_event(
            &self.state,
            session_id,
            "checkpoint.restored",
            json!({"manifest_id": manifest_id, "restored_items": restored}),
        )?;
        Ok(RestoreOutcome {
            manifest_id: manifest_id.to_string(),
            status: "restored",
            restored_items: restored,
        })
    }

    pub fn submit_turn(
        &self,
        session_id: &str,
        input: &str,
        idempotency_key: &str,
        model: Option<&str>,
    ) -> SdkResult<TurnResponse> {
        if input.is_empty() {
            return Err(SdkError::invalid("input is required"));
        }
        if idempotency_key.is_empty() {
            return Err(SdkError::invalid("idempotency_key is required"));
        }
        match self.runtime.block_on(self.state.agent.submit(
            session_id,
            idempotency_key,
            input,
            model,
        )) {
            Ok(response) => Ok(response),
            Err(error) if error.code == "approval_required" => Ok(TurnResponse::AwaitingApproval {
                turn_id: detail_string(&error, "turn_id")?,
                tool_call_id: detail_string(&error, "tool_call_id")?,
                approval_id: detail_string(&error, "approval_id")?,
            }),
            Err(error) => Err(SdkError::from(error)),
        }
    }

    pub fn cancel_turn(&self, _session_id: &str, turn_id: &str) -> SdkResult<CancellationOutcome> {
        if !self.state.agent.cancel(turn_id) {
            return Err(SdkError::new("conflict", "turn is not running"));
        }
        Ok(CancellationOutcome {
            turn_id: turn_id.to_string(),
            status: "cancellation_requested",
        })
    }

    pub fn get_approval(&self, approval_id: &str) -> SdkResult<ApprovalRecord> {
        self.state
            .store
            .approval(approval_id)?
            .ok_or_else(|| SdkError::missing("approval"))
    }

    pub fn resolve_approval(
        &self,
        approval_id: &str,
        decision: &str,
    ) -> SdkResult<ApprovalOutcome> {
        if !["deny", "allow_once", "allow_session"].contains(&decision) {
            return Err(SdkError::invalid("invalid approval decision"));
        }
        let resolved = self
            .runtime
            .block_on(self.state.agent.resolve_approval(approval_id, decision))?;
        if !resolved {
            return Err(SdkError::new(
                "conflict",
                "approval is missing or already resolved",
            ));
        }
        Ok(ApprovalOutcome {
            approval_id: approval_id.to_string(),
            decision: decision.to_string(),
        })
    }

    pub fn subscribe_session_events(
        &self,
        session_id: String,
        _after: i64,
        callback: SunCodeEventCallback,
        user_data: *mut c_void,
    ) -> SdkResult<RuntimeSubscription> {
        logging::write(
            Level::Debug,
            "subscribe",
            format!("begin session={session_id} after={_after}"),
        );
        if self.state.store.session_by_id(&session_id)?.is_none() {
            return Err(SdkError::missing("session"));
        }

        // Events are live-only. Hosts recover durable state by reading a fresh snapshot.
        let mut receiver = self.state.events.subscribe();
        let cancellation = CancellationToken::new();
        let cancellation_for_thread = cancellation.clone();
        let handle = self.runtime.handle().clone();
        let user_data = user_data as usize;
        let log_session_id = session_id.clone();
        let subscribed_session_id = session_id.clone();
        let join = std::thread::spawn(move || loop {
            let next = handle.block_on(async {
                tokio::select! {
                    _ = cancellation_for_thread.cancelled() => None,
                    value = receiver.recv() => Some(value),
                }
            });
            match next {
                None => {
                    logging::write(
                        Level::Debug,
                        "subscribe",
                        format!("thread_exit session={log_session_id} reason=cancelled"),
                    );
                    break;
                }
                Some(Ok(event)) if event.session_id == subscribed_session_id => {
                    emit_sdk_event(callback, user_data, &event);
                }
                Some(Ok(_)) => {}
                Some(Err(broadcast::error::RecvError::Lagged(_))) => {
                    logging::write(
                        Level::Warn,
                        "subscribe",
                        format!("lagged session={log_session_id}"),
                    );
                    let event = SessionEvent {
                        session_id: subscribed_session_id.clone(),
                        occurred_at: chrono::Utc::now()
                            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                        event_type: "resync.required".into(),
                        payload: json!({"reason":"subscriber_lagged"}),
                    };
                    emit_sdk_event(callback, user_data, &event);
                }
                Some(Err(broadcast::error::RecvError::Closed)) => {
                    logging::write(
                        Level::Debug,
                        "subscribe",
                        format!("thread_exit session={log_session_id} reason=channel_closed"),
                    );
                    break;
                }
            }
        });
        logging::write(
            Level::Info,
            "subscribe",
            format!("ready session={session_id}"),
        );
        Ok(RuntimeSubscription {
            session_id,
            cancellation,
            join: Mutex::new(Some(join)),
        })
    }

    #[cfg(test)]
    fn from_state_for_test(state: RuntimeState) -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("suncode-sdk-test")
            .build()
            .unwrap();
        Self {
            _lock: None,
            data_dir: PathBuf::new(),
            runtime,
            state,
        }
    }
}

fn operation_error(error: Value) -> SdkError {
    SdkError::new(
        error
            .get("code")
            .and_then(Value::as_str)
            .unwrap_or("operation_failed"),
        error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("operation failed"),
    )
}

fn decode_operation<T: DeserializeOwned>(value: Value, operation: &str) -> SdkResult<T> {
    serde_json::from_value(value).map_err(|error| {
        SdkError::unavailable(format!("{operation} returned an invalid result: {error}"))
    })
}

fn detail_string(error: &AgentError, name: &str) -> SdkResult<String> {
    error
        .details
        .get(name)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| SdkError::unavailable(format!("approval outcome is missing {name}")))
}

fn emit_event(
    state: &RuntimeState,
    session_id: &str,
    event_type: &str,
    payload: Value,
) -> SdkResult<()> {
    let event = state
        .store
        .append_content(session_id, event_type, &payload)?;
    let _ = state.events.send(event);
    Ok(())
}

pub type SunCodeEventCallback = unsafe extern "C" fn(*const c_char, *mut c_void);

pub struct RuntimeSubscription {
    session_id: String,
    cancellation: CancellationToken,
    join: Mutex<Option<JoinHandle<()>>>,
}

impl RuntimeSubscription {
    fn close(&self) {
        logging::write(
            Level::Debug,
            "subscription_close",
            format!("cancel_begin session={}", self.session_id),
        );
        self.cancellation.cancel();
        if let Ok(mut join) = self.join.lock() {
            if let Some(join) = join.take() {
                logging::write(
                    Level::Debug,
                    "subscription_close",
                    format!("join_begin session={}", self.session_id),
                );
                let _ = join.join();
                logging::write(
                    Level::Debug,
                    "subscription_close",
                    format!("join_end session={}", self.session_id),
                );
            }
        }
        logging::write(
            Level::Debug,
            "subscription_close",
            format!("end session={}", self.session_id),
        );
    }
}

impl Drop for RuntimeSubscription {
    fn drop(&mut self) {
        self.close();
    }
}

pub struct SunCodeRuntimeHandle {
    sdk: RuntimeSdk,
}

pub struct SunCodeRuntimeSubscriptionHandle {
    _subscription: RuntimeSubscription,
}

#[no_mangle]
pub extern "C" fn suncode_runtime_sdk_abi_version() -> u32 {
    SUNCODE_RUNTIME_SDK_ABI_VERSION
}

#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_open_default(
    error_out: *mut *mut c_char,
) -> *mut SunCodeRuntimeHandle {
    write_error_out(error_out, ptr::null_mut());
    match catch_unwind(AssertUnwindSafe(RuntimeSdk::open_default)) {
        Ok(Ok(sdk)) => Box::into_raw(Box::new(SunCodeRuntimeHandle { sdk })),
        Ok(Err(error)) => {
            write_error_out(error_out, into_c_string(error.to_string()));
            ptr::null_mut()
        }
        Err(_) => {
            write_error_out(
                error_out,
                into_c_string("runtime_unavailable: runtime initialization panicked".to_string()),
            );
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_close(handle: *mut SunCodeRuntimeHandle) {
    if !handle.is_null() {
        let _ = catch_unwind(AssertUnwindSafe(|| drop(Box::from_raw(handle))));
    }
}

macro_rules! ffi_no_args {
    ($function:ident, $method:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $function(handle: *mut SunCodeRuntimeHandle) -> *mut c_char {
            ffi_call(handle, |sdk| sdk.$method())
        }
    };
}

ffi_no_args!(suncode_runtime_sdk_health, health);
ffi_no_args!(suncode_runtime_sdk_diagnostics, diagnostics);
ffi_no_args!(suncode_runtime_sdk_list_models, list_models);
ffi_no_args!(suncode_runtime_sdk_list_credentials, list_credentials);
ffi_no_args!(suncode_runtime_sdk_list_projects, list_projects);

#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_list_project_dependencies(
    handle: *mut SunCodeRuntimeHandle,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.list_project_dependencies(&c_string(project_id, "project_id")?)
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_add_project_dependency(
    handle: *mut SunCodeRuntimeHandle,
    project_id: *const c_char,
    path: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.add_project_dependency(
            &c_string(project_id, "project_id")?,
            &c_string(path, "path")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_remove_project_dependency(
    handle: *mut SunCodeRuntimeHandle,
    project_id: *const c_char,
    dependency_id: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.remove_project_dependency(
            &c_string(project_id, "project_id")?,
            &c_string(dependency_id, "dependency_id")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_list_project_directory(
    handle: *mut SunCodeRuntimeHandle,
    project_id: *const c_char,
    dependency_id: *const c_char,
    path: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let dependency_id = optional_c_string(dependency_id, "dependency_id")?;
        sdk.list_project_directory(
            &c_string(project_id, "project_id")?,
            dependency_id.as_deref(),
            &c_string(path, "path")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_list_settings(
    handle: *mut SunCodeRuntimeHandle,
    project_id: *const c_char,
    session_id: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let project_id = optional_c_string(project_id, "project_id")?;
        let session_id = optional_c_string(session_id, "session_id")?;
        sdk.list_settings(project_id.as_deref(), session_id.as_deref())
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_set_setting(
    handle: *mut SunCodeRuntimeHandle,
    scope: *const c_char,
    project_id: *const c_char,
    session_id: *const c_char,
    key: *const c_char,
    value_json: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let scope = c_string(scope, "scope")?;
        let project_id = optional_c_string(project_id, "project_id")?;
        let session_id = optional_c_string(session_id, "session_id")?;
        let key = c_string(key, "key")?;
        let value_document = json_from_c(value_json, "value_json")?;
        let value = value_document
            .get("value")
            .cloned()
            .unwrap_or(value_document);
        sdk.set_setting(
            &scope,
            project_id.as_deref(),
            session_id.as_deref(),
            &key,
            &value,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_set_credential(
    handle: *mut SunCodeRuntimeHandle,
    provider: *const c_char,
    api_key: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.set_credential(
            &c_string(provider, "provider")?,
            &c_string(api_key, "api_key")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_remove_credential(
    handle: *mut SunCodeRuntimeHandle,
    provider: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.remove_credential(&c_string(provider, "provider")?)
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_open_project(
    handle: *mut SunCodeRuntimeHandle,
    path: *const c_char,
    display_name: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let path = c_string(path, "path")?;
        let display_name = optional_c_string(display_name, "display_name")?;
        sdk.open_project(&path, display_name.as_deref())
    })
}

macro_rules! ffi_one_string {
    ($function:ident, $method:ident, $argument:literal) => {
        #[no_mangle]
        pub unsafe extern "C" fn $function(
            handle: *mut SunCodeRuntimeHandle,
            value: *const c_char,
        ) -> *mut c_char {
            ffi_call(handle, |sdk| sdk.$method(&c_string(value, $argument)?))
        }
    };
}

ffi_one_string!(
    suncode_runtime_sdk_select_project,
    select_project,
    "project_id"
);
ffi_one_string!(suncode_runtime_sdk_git_status, git_status, "project_id");
ffi_one_string!(
    suncode_runtime_sdk_list_sessions,
    list_sessions,
    "project_id"
);
ffi_one_string!(
    suncode_runtime_sdk_archive_session,
    archive_session,
    "session_id"
);

#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_set_session_pinned(
    handle: *mut SunCodeRuntimeHandle,
    session_id: *const c_char,
    pinned: u8,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.set_session_pinned(&c_string(session_id, "session_id")?, pinned != 0)
    })
}
ffi_one_string!(
    suncode_runtime_sdk_reopen_session,
    reopen_session,
    "session_id"
);
ffi_one_string!(
    suncode_runtime_sdk_list_checkpoints,
    list_checkpoints,
    "session_id"
);
ffi_one_string!(
    suncode_runtime_sdk_session_usage,
    session_usage,
    "session_id"
);
ffi_one_string!(
    suncode_runtime_sdk_list_provider_exchanges,
    list_provider_exchanges,
    "session_id"
);
ffi_one_string!(
    suncode_runtime_sdk_checkpoint_manifest,
    checkpoint_manifest,
    "manifest_id"
);
ffi_one_string!(
    suncode_runtime_sdk_get_approval,
    get_approval,
    "approval_id"
);

#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_create_session(
    handle: *mut SunCodeRuntimeHandle,
    project_id: *const c_char,
    title: *const c_char,
    model: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let project_id = c_string(project_id, "project_id")?;
        let title = optional_c_string(title, "title")?;
        let model = optional_c_string(model, "model")?;
        sdk.create_session(&project_id, title.as_deref(), model.as_deref())
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_git_diff_file(
    handle: *mut SunCodeRuntimeHandle,
    project_id: *const c_char,
    scope: *const c_char,
    path: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.git_diff_file(
            &c_string(project_id, "project_id")?,
            &c_string(scope, "scope")?,
            &c_string(path, "path")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_rename_session(
    handle: *mut SunCodeRuntimeHandle,
    session_id: *const c_char,
    title: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.rename_session(
            &c_string(session_id, "session_id")?,
            &c_string(title, "title")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_session_snapshot(
    handle: *mut SunCodeRuntimeHandle,
    session_id: *const c_char,
    after: i64,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.session_snapshot(&c_string(session_id, "session_id")?, after)
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_provider_exchange(
    handle: *mut SunCodeRuntimeHandle,
    session_id: *const c_char,
    exchange_id: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.provider_exchange(
            &c_string(session_id, "session_id")?,
            &c_string(exchange_id, "exchange_id")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_restore_checkpoint(
    handle: *mut SunCodeRuntimeHandle,
    manifest_id: *const c_char,
    session_id: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.restore_checkpoint(
            &c_string(manifest_id, "manifest_id")?,
            &c_string(session_id, "session_id")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_submit_turn(
    handle: *mut SunCodeRuntimeHandle,
    session_id: *const c_char,
    input: *const c_char,
    idempotency_key: *const c_char,
    model: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let session_id = c_string(session_id, "session_id")?;
        let input = c_string(input, "input")?;
        let idempotency_key = c_string(idempotency_key, "idempotency_key")?;
        let model = optional_c_string(model, "model")?;
        sdk.submit_turn(&session_id, &input, &idempotency_key, model.as_deref())
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_cancel_turn(
    handle: *mut SunCodeRuntimeHandle,
    session_id: *const c_char,
    turn_id: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.cancel_turn(
            &c_string(session_id, "session_id")?,
            &c_string(turn_id, "turn_id")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_resolve_approval(
    handle: *mut SunCodeRuntimeHandle,
    approval_id: *const c_char,
    decision: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.resolve_approval(
            &c_string(approval_id, "approval_id")?,
            &c_string(decision, "decision")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_subscribe_session(
    handle: *mut SunCodeRuntimeHandle,
    session_id: *const c_char,
    after: i64,
    callback: Option<SunCodeEventCallback>,
    user_data: *mut c_void,
    error_out: *mut *mut c_char,
) -> *mut SunCodeRuntimeSubscriptionHandle {
    write_error_out(error_out, ptr::null_mut());
    let result = catch_unwind(AssertUnwindSafe(|| -> SdkResult<_> {
        let handle = handle
            .as_ref()
            .ok_or_else(|| SdkError::unavailable("runtime handle is null"))?;
        let callback = callback.ok_or_else(|| SdkError::invalid("callback is null"))?;
        handle.sdk.subscribe_session_events(
            c_string(session_id, "session_id")?,
            after,
            callback,
            user_data,
        )
    }));
    match result {
        Ok(Ok(subscription)) => Box::into_raw(Box::new(SunCodeRuntimeSubscriptionHandle {
            _subscription: subscription,
        })),
        Ok(Err(error)) => {
            write_error_out(error_out, into_c_string(error.to_string()));
            ptr::null_mut()
        }
        Err(_) => {
            write_error_out(
                error_out,
                into_c_string("runtime_unavailable: subscription panicked".to_string()),
            );
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_subscription_close(
    subscription: *mut SunCodeRuntimeSubscriptionHandle,
) {
    if !subscription.is_null() {
        let _ = catch_unwind(AssertUnwindSafe(|| drop(Box::from_raw(subscription))));
    }
}

#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_string_free(value: *mut c_char) {
    if !value.is_null() {
        drop(CString::from_raw(value));
    }
}

unsafe fn ffi_call<F, T>(handle: *mut SunCodeRuntimeHandle, call: F) -> *mut c_char
where
    F: FnOnce(&RuntimeSdk) -> SdkResult<T>,
    T: Serialize,
{
    let result = catch_unwind(AssertUnwindSafe(|| {
        let handle = handle
            .as_ref()
            .ok_or_else(|| SdkError::unavailable("runtime handle is null"))?;
        call(&handle.sdk)
    }));
    match result {
        Ok(result) => result_envelope(result),
        Err(_) => result_envelope::<T>(Err(SdkError::unavailable("SDK call panicked"))),
    }
}

fn result_envelope<T: Serialize>(result: SdkResult<T>) -> *mut c_char {
    let value = match result {
        Ok(body) => match serde_json::to_value(body) {
            Ok(body) => json!({"ok": true, "body": body}),
            Err(error) => json!({
                "ok": false,
                "error": SdkError::unavailable(error.to_string())
            }),
        },
        Err(error) => json!({"ok": false, "error": error}),
    };
    into_c_string(value.to_string())
}

fn emit_sdk_event(callback: SunCodeEventCallback, user_data: usize, event: &SessionEvent) {
    let Ok(value) = serde_json::to_string(event) else {
        return;
    };
    let Ok(value) = CString::new(value) else {
        return;
    };
    unsafe { callback(value.as_ptr(), user_data as *mut c_void) };
}

fn c_string(pointer: *const c_char, name: &str) -> SdkResult<String> {
    if pointer.is_null() {
        return Err(SdkError::invalid(format!("{name} is null")));
    }
    unsafe {
        CStr::from_ptr(pointer)
            .to_str()
            .map(str::to_string)
            .map_err(|error| SdkError::invalid(format!("{name} is not UTF-8: {error}")))
    }
}

fn optional_c_string(pointer: *const c_char, name: &str) -> SdkResult<Option<String>> {
    if pointer.is_null() {
        return Ok(None);
    }
    c_string(pointer, name).map(Some)
}

fn json_from_c(pointer: *const c_char, name: &str) -> SdkResult<Value> {
    let value = c_string(pointer, name)?;
    serde_json::from_str(&value)
        .map_err(|error| SdkError::invalid(format!("{name} is invalid: {error}")))
}

unsafe fn write_error_out(error_out: *mut *mut c_char, value: *mut c_char) {
    if !error_out.is_null() {
        *error_out = value;
    }
}

fn into_c_string(value: String) -> *mut c_char {
    CString::new(value)
        .unwrap_or_else(|_| CString::new("string contained an interior nul byte").unwrap())
        .into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_state(directory: &std::path::Path) -> RuntimeState {
        let store = Store::open_memory().unwrap();
        let operations =
            Arc::new(suncode_tool::Operations::new(directory.join("operations")).unwrap());
        let credentials = CredentialStore::memory(Some("test-key"), None, None, None, None, None);
        let (events, _) = broadcast::channel(16);
        let providers =
            Arc::new(registry_from_store(&store, Arc::new(credentials.clone())).unwrap());
        let agent = Agent::new(
            store.clone(),
            providers.clone(),
            operations.clone(),
            events.clone(),
            false,
        );
        RuntimeState {
            store,
            operations,
            active_project: Arc::new(Mutex::new(None)),
            events,
            credentials,
            agent,
            providers,
        }
    }

    #[test]
    fn named_sdk_methods_serve_project_session_and_model_dtos() {
        let directory = tempfile::tempdir().unwrap();
        let sdk = RuntimeSdk::from_state_for_test(test_state(directory.path()));
        assert!(sdk.health().unwrap().ok);
        assert_eq!(sdk.list_credentials().unwrap().credentials.len(), 6);
        assert_eq!(sdk.list_models().unwrap().models.len(), 12);
        let project = sdk
            .open_project(directory.path().to_str().unwrap(), None)
            .unwrap();
        let session = sdk
            .create_session(&project.project_id, Some("First"), Some("gpt-5.5"))
            .unwrap();
        assert_eq!(
            session.project_id.as_deref(),
            Some(project.project_id.as_str())
        );
        assert_eq!(session.model_id.as_deref(), Some("gpt-5.5"));
        let usage = sdk.session_usage(&session.session_id).unwrap();
        assert_eq!(usage.session_id, session.session_id);
        assert_eq!(usage.total_tokens, 0);
        sdk.state
            .store
            .append_content(
                &session.session_id,
                "turn.state",
                &json!({
                    "turn_id":"turn-1",
                    "state":"calling_model",
                    "model_id":"gpt-5.5"
                }),
            )
            .unwrap();
        sdk.state
            .store
            .append_content(
                &session.session_id,
                "provider.exchange.started",
                &json!({
                    "exchange_id":"exchange-1",
                    "turn_id":"turn-1",
                    "provider":"openai",
                    "model_id":"gpt-5.5",
                    "wire_model":"gpt-5.5",
                    "iteration":1,
                    "input_messages":[{"role":"user","content":[{"type":"text","text":"Inspect package.json"}]}]
                }),
            )
            .unwrap();
        let traces = sdk.list_provider_exchanges(&session.session_id).unwrap();
        assert_eq!(traces.exchanges.len(), 1);
        assert_eq!(traces.turns.len(), 1);
        assert_eq!(
            sdk.provider_exchange(&session.session_id, "exchange-1")
                .unwrap()
                .exchange
                .provider,
            "openai"
        );
        let details = sdk
            .provider_exchange(&session.session_id, "exchange-1")
            .unwrap();
        assert!(details.messages.is_empty());
        assert!(details.tool_uses.is_empty());
        assert_eq!(
            sdk.session_usage("missing-session").unwrap_err().code,
            "session_not_found"
        );
    }

    #[test]
    fn project_default_model_is_used_when_session_model_is_omitted() {
        let directory = tempfile::tempdir().unwrap();
        let sdk = RuntimeSdk::from_state_for_test(test_state(directory.path()));
        let project = sdk
            .open_project(directory.path().to_str().unwrap(), None)
            .unwrap();
        sdk.set_setting(
            "project",
            Some(&project.project_id),
            None,
            "default_model",
            &json!("gpt-5.5"),
        )
        .unwrap();

        let session = sdk.create_session(&project.project_id, None, None).unwrap();
        assert_eq!(session.model_id.as_deref(), Some("gpt-5.5"));
        sdk.set_setting(
            "session",
            None,
            Some(&session.session_id),
            "full_control",
            &json!(true),
        )
        .unwrap();
        let setting = sdk
            .list_settings(Some(&project.project_id), Some(&session.session_id))
            .unwrap()
            .settings
            .into_iter()
            .find(|setting| setting.key == "full_control")
            .unwrap();
        assert_eq!(setting.value, json!(true));
        assert_eq!(setting.scope, "session");
    }

    #[test]
    fn project_dependencies_are_read_only_and_browsed_on_demand() {
        let directory = tempfile::tempdir().unwrap();
        let project_root = directory.path().join("project");
        let dependency_root = directory.path().join("dependency");
        std::fs::create_dir_all(project_root.join("src")).unwrap();
        std::fs::create_dir_all(dependency_root.join("lib")).unwrap();
        std::fs::create_dir_all(dependency_root.join("nested")).unwrap();
        std::fs::write(dependency_root.join("lib/code.rs"), "pub fn shared() {}\n").unwrap();
        let sdk = RuntimeSdk::from_state_for_test(test_state(directory.path()));
        let project = sdk
            .open_project(project_root.to_str().unwrap(), None)
            .unwrap();
        let dependency = sdk
            .add_project_dependency(&project.project_id, dependency_root.to_str().unwrap())
            .unwrap();
        assert_eq!(
            sdk.list_project_dependencies(&project.project_id)
                .unwrap()
                .dependencies
                .len(),
            1
        );
        let root = sdk
            .list_project_directory(&project.project_id, Some(&dependency.dependency_id), ".")
            .unwrap();
        assert_eq!(root["entries"][0]["name"], "lib");
        let nested = sdk
            .list_project_directory(&project.project_id, Some(&dependency.dependency_id), "lib")
            .unwrap();
        assert_eq!(nested["entries"][0]["name"], "code.rs");
        assert!(sdk
            .add_project_dependency(&project.project_id, project_root.to_str().unwrap())
            .is_err());
        assert!(sdk
            .add_project_dependency(
                &project.project_id,
                dependency_root.join("nested").to_str().unwrap()
            )
            .is_err());
        assert!(sdk
            .add_project_dependency(&project.project_id, directory.path().to_str().unwrap())
            .is_err());
        assert!(
            sdk.remove_project_dependency(&project.project_id, &dependency.dependency_id)
                .unwrap()
                .removed
        );
    }

    #[test]
    fn logging_settings_are_global_and_typed() {
        assert!(validate_logging_setting("global", "log_level", &json!("TRACE")).is_ok());
        assert!(validate_logging_setting("global", "log_directory", &json!("")).is_ok());
        assert!(validate_logging_setting("global", "log_max_bytes", &json!(1024)).is_ok());
        assert!(validate_logging_setting("global", "log_retention", &json!(0)).is_ok());

        assert!(validate_logging_setting("project", "log_level", &json!("INFO")).is_err());
        assert!(validate_logging_setting("global", "log_level", &json!("VERBOSE")).is_err());
        assert!(validate_logging_setting("global", "log_directory", &json!(7)).is_err());
        assert!(validate_logging_setting("global", "log_max_bytes", &json!(1023)).is_err());
        assert!(validate_logging_setting("global", "log_retention", &json!(101)).is_err());
        assert!(validate_logging_setting("session", "full_control", &json!(true)).is_ok());
        assert!(validate_logging_setting("global", "full_control", &json!(true)).is_err());
        assert!(validate_logging_setting("session", "full_control", &json!("yes")).is_err());
    }

    #[test]
    fn named_git_methods_return_project_scoped_status_and_diff() {
        let directory = tempfile::tempdir().unwrap();
        let project_root = directory.path().join("project");
        git2::Repository::init(&project_root).unwrap();
        std::fs::write(project_root.join("new.txt"), "first\nsecond\n").unwrap();
        let sdk = RuntimeSdk::from_state_for_test(test_state(directory.path()));
        let project = sdk
            .open_project(project_root.to_str().unwrap(), None)
            .unwrap();
        let status = sdk.git_status(&project.project_id).unwrap();
        assert_eq!(status.changed_files, 1);
        assert_eq!(status.files[0].path, "new.txt");
        assert_eq!(status.files[0].status, "untracked");
        let diff = sdk
            .git_diff_file(&project.project_id, "all", "new.txt")
            .unwrap();
        assert_eq!(diff.path, "new.txt");
        assert_eq!(diff.additions, 2);
        assert!(!diff.hunks.is_empty());
        assert_eq!(
            sdk.git_diff_file(&project.project_id, "invalid", "new.txt")
                .unwrap_err()
                .code,
            "invalid_arguments"
        );
    }

    #[test]
    fn named_credential_methods_update_model_availability() {
        let directory = tempfile::tempdir().unwrap();
        let sdk = RuntimeSdk::from_state_for_test(test_state(directory.path()));
        sdk.set_credential("claude", "claude-key").unwrap();
        assert!(sdk
            .list_models()
            .unwrap()
            .models
            .iter()
            .any(|model| model.provider == "claude" && model.availability == "configured"));
        sdk.remove_credential("claude").unwrap();
        assert_eq!(
            sdk.set_credential("unsupported", "unused")
                .unwrap_err()
                .code,
            "invalid_arguments"
        );
    }

    #[test]
    fn ffi_exposes_a_versioned_method_oriented_boundary() {
        assert_eq!(suncode_runtime_sdk_abi_version(), 1);
        let directory = tempfile::tempdir().unwrap();
        let sdk = RuntimeSdk::from_state_for_test(test_state(directory.path()));
        let project_root = directory.path().join("project");
        git2::Repository::init(&project_root).unwrap();
        std::fs::write(project_root.join("new.txt"), "new\n").unwrap();
        let project = sdk
            .open_project(project_root.to_str().unwrap(), None)
            .unwrap();
        let session = sdk
            .create_session(&project.project_id, None, Some("gpt-5.5"))
            .unwrap();
        let handle = Box::into_raw(Box::new(SunCodeRuntimeHandle { sdk }));
        let response = unsafe { suncode_runtime_sdk_health(handle) };
        let envelope: Value =
            unsafe { serde_json::from_str(CStr::from_ptr(response).to_str().unwrap()).unwrap() };
        assert_eq!(envelope["ok"], true);
        unsafe {
            suncode_runtime_sdk_string_free(response);
            let session_id = CString::new(session.session_id).unwrap();
            let response = suncode_runtime_sdk_session_usage(handle, session_id.as_ptr());
            let envelope: Value =
                serde_json::from_str(CStr::from_ptr(response).to_str().unwrap()).unwrap();
            assert_eq!(envelope["ok"], true);
            assert_eq!(envelope["body"]["total_tokens"], 0);
            suncode_runtime_sdk_string_free(response);
            let project_id = CString::new(project.project_id).unwrap();
            let response = suncode_runtime_sdk_git_status(handle, project_id.as_ptr());
            let envelope: Value =
                serde_json::from_str(CStr::from_ptr(response).to_str().unwrap()).unwrap();
            assert_eq!(envelope["ok"], true);
            assert_eq!(envelope["body"]["changed_files"], 1);
            suncode_runtime_sdk_string_free(response);
            suncode_runtime_sdk_close(handle);
        }
    }

    #[test]
    fn session_snapshot_serializes_normalized_conversation_turns() {
        let directory = tempfile::tempdir().unwrap();
        let state = test_state(directory.path());
        let project = state
            .store
            .project(directory.path().to_str().unwrap(), "Test")
            .unwrap();
        let session = state
            .store
            .create_session(
                &project.project_id,
                Some("First"),
                Some("deepseek-v4-flash"),
            )
            .unwrap();
        state
            .store
            .append_content(
                &session.session_id,
                "turn.state",
                &json!({"turn_id":"turn-1","state":"completed"}),
            )
            .unwrap();
        state
            .store
            .append_content(
                &session.session_id,
                "message.user",
                &json!({"message_id":"user-1","turn_id":"turn-1","message":Message::text("user","inspect")}),
            )
            .unwrap();
        let sdk = RuntimeSdk::from_state_for_test(state);

        let snapshot =
            serde_json::to_value(sdk.session_snapshot(&session.session_id, 0).unwrap()).unwrap();

        assert_eq!(snapshot["messages"][0]["role"], "user");
        assert_eq!(snapshot["conversationTurns"][0]["turnId"], "turn-1");
        assert_eq!(
            snapshot["conversationTurns"][0]["messages"][0]["messageId"],
            "user-1"
        );
    }

    unsafe extern "C" fn collect_event(event_json: *const c_char, user_data: *mut c_void) {
        let sender = &*(user_data as *const std::sync::mpsc::Sender<String>);
        let value = CStr::from_ptr(event_json).to_string_lossy().to_string();
        let _ = sender.send(value);
    }

    #[test]
    fn subscription_delivers_live_events_without_replay() {
        let directory = tempfile::tempdir().unwrap();
        let state = test_state(directory.path());
        let project = state
            .store
            .project(directory.path().to_str().unwrap(), "Test")
            .unwrap();
        let session = state
            .store
            .create_session(
                &project.project_id,
                Some("First"),
                Some("deepseek-v4-flash"),
            )
            .unwrap();
        let sender_for_live = state.events.clone();
        let store_for_live = state.store.clone();
        let sdk = RuntimeSdk::from_state_for_test(state);
        let (sender, receiver) = std::sync::mpsc::channel::<String>();
        let subscription = sdk
            .subscribe_session_events(
                session.session_id.clone(),
                0,
                collect_event,
                &sender as *const _ as *mut c_void,
            )
            .unwrap();
        let live = store_for_live
            .append_content(
                &session.session_id,
                "turn.state",
                &json!({"turn_id": "turn-1", "state": "calling_model"}),
            )
            .unwrap();
        let _ = sender_for_live.send(live.clone());
        let received: SessionEvent = serde_json::from_str(
            &receiver
                .recv_timeout(std::time::Duration::from_secs(2))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(received.event_type, live.event_type);
        subscription.close();
    }
}
