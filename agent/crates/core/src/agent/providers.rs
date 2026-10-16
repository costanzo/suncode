//! Tool providers registered with the agent.
//!
//! External tool sources implement `suncode_common::tool::ToolProvider` in
//! their own crates. The core routes calls to them by name and keeps policy,
//! approval, auditing, and events. Built-in tools are not providers: their
//! execution depends on turn context (dependency scopes, the bash timeout
//! ceiling, idempotency keys, live output events) that only the core has.

use std::sync::Arc;
use suncode_common::tool::{BoxFuture, Cancellation, ToolDefinition, ToolProvider};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Default)]
pub(super) struct ToolProviders(Vec<Arc<dyn ToolProvider>>);

impl ToolProviders {
    pub(super) fn new(providers: Vec<Arc<dyn ToolProvider>>) -> Self {
        Self(providers)
    }

    /// The provider that handles `name`, if any.
    pub(super) fn find(&self, name: &str) -> Option<&dyn ToolProvider> {
        self.0
            .iter()
            .find(|provider| provider.owns(name))
            .map(Arc::as_ref)
    }

    /// Tools advertised by every provider for `project_id`, in registration order.
    pub(super) async fn catalog(&self, project_id: &str) -> Vec<ToolDefinition> {
        let mut definitions = Vec::new();
        for provider in &self.0 {
            definitions.extend(provider.catalog(project_id).await);
        }
        definitions
    }
}

/// Adapts the turn's Tokio token to the runtime-neutral provider interface.
pub(super) struct TokenCancellation(pub(super) CancellationToken);

impl Cancellation for TokenCancellation {
    fn is_cancelled(&self) -> bool {
        self.0.is_cancelled()
    }

    fn cancelled(&self) -> BoxFuture<'_, ()> {
        Box::pin(self.0.cancelled())
    }
}
