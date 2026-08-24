mod agent;
mod agent_lock;
mod config;
mod context;
mod credentials;
mod domain;
mod logging;
mod policy;
mod sdk;
mod tools;

pub use sdk::{
    AgentSdk, ApprovalOutcome, CancellationOutcome, CheckpointDetails, CheckpointsResult,
    CredentialUpdate, CredentialsResult, DependencyRemoval, DiagnosticsResult, HealthResult,
    ModelsResult, ProjectDependenciesResult, ProjectDependencyDto, ProjectsResult, RecoveryStatus,
    RestoreOutcome, SdkError, SdkResult, SessionSnapshot, SessionUsageResult, SessionsResult,
    SettingUpdate, SettingsResult, SunCodeEventCallback, SUNCODE_AGENT_SDK_ABI_VERSION,
};

pub use agent::TurnResponse;
pub use credentials::CredentialState;
pub use domain::{
    ApprovalRecord, CheckpointItem, CheckpointManifest, Message, ProjectDependencyRecord,
    ProjectRecord, SessionEvent, SessionRecord, SettingRecord,
};
pub use suncode_llm::{ModelCapabilities, ModelDescriptor, ModelLimits};
