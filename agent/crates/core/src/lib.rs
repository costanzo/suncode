//! The SunCode agent core: turn execution, policy, approval, events, and
//! tool routing. Data and shared types are used from their defining crates
//! (`suncode_data`, `suncode_common`, `suncode_llm`) directly.

pub mod agent;
mod agent_lock;
mod context;
mod policy;

pub use agent::events::*;
pub use agent::{
    AgentAttentionEvent, AgentEventSubscription, AgentEventSubscriptionControl, AttentionEventHub,
    AttentionEventSubscription, AttentionEventSubscriptionControl, AttentionKind,
    AttentionReceiveError, BrowserInstallationState, BrowserRuntimeInfo, BrowserRuntimeState,
    BrowserVisibilityCapability, ComputerRuntimeInfo, EventReceiveError,
    LanguageServerRuntimeState, LanguageServerRuntimeStatus, PreviewState, PreviewStatus,
    SessionEventHub, TurnResponse,
};
pub use agent_lock::AgentLock;

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
