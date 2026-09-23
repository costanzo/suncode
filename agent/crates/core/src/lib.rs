pub mod agent;
mod agent_lock;
mod context;
pub mod domain;
pub mod logging;
mod policy;

pub use agent::events::*;
pub use agent::{
    AgentAttentionEvent, AgentEventSubscription, AgentEventSubscriptionControl, AttentionEventHub,
    AttentionEventSubscription, AttentionEventSubscriptionControl, AttentionKind,
    AttentionReceiveError, BrowserInstallationState, BrowserRuntimeInfo, BrowserRuntimeState,
    BrowserVisibilityCapability, ComputerRuntimeInfo, EventReceiveError,
    LanguageServerRuntimeState, LanguageServerRuntimeStatus, SessionEventHub, TurnResponse,
};
pub use agent_lock::AgentLock;
pub use domain::{
    ApprovalRecord, CheckpointItem, CheckpointManifest, Message, ProjectDependencyRecord,
    ProjectRecord, SessionEvent, SessionImageRecord, SessionRecord, SettingRecord,
    SubagentInvocationRecord,
};
pub use suncode_common::BusinessError;
pub use suncode_config::Config;
pub use suncode_llm::{ModelCapabilities, ModelDescriptor, ModelLimits};

pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    #[test]
    fn version_matches_the_agent_core_package() {
        assert_eq!(super::version(), env!("CARGO_PKG_VERSION"));
    }
}
