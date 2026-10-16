//! The contract between the agent core and tool providers.
//!
//! Providers live in their own crates (browser, MCP, LSP, ...) and implement
//! [`ToolProvider`]; the core owns policy, approval, auditing, and events and
//! only sees this interface. Everything here uses `std` alone so the crates
//! that depend on `suncode-common` do not pull in an async runtime.

use crate::BusinessError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{future::Future, pin::Pin};

/// A boxed, sendable future; identical to `futures_util::future::BoxFuture`.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// A model-facing tool declaration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

/// What a tool call can affect. The core maps a risk to allow, approval, or
/// deny; providers only classify their own tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Risk {
    ReadOnly,
    ProjectWrite,
    ProcessExecution,
    NetworkAccess,
    ExternalTool,
    BrowserAccess,
    ComputerObserve,
    ComputerInput,
}

/// A cancellation signal supplied by the caller. The core implements it over
/// its async runtime's token.
pub trait Cancellation: Send + Sync {
    /// Non-blocking check, for blocking code that polls.
    fn is_cancelled(&self) -> bool;
    /// Completes once cancellation is requested, for async code to select on.
    fn cancelled(&self) -> BoxFuture<'_, ()>;
}

/// Where a model-supplied path resolves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopedPath {
    /// Inside the opened project.
    Project { relative: String },
    /// Inside a registered read-only source dependency.
    Dependency {
        dependency_id: String,
        root: std::path::PathBuf,
        relative: String,
    },
}

/// Resolves model-supplied paths, including `dependency:<id>/...` aliases,
/// to the root a tool may access. Implemented by the core.
pub trait PathScopes: Send + Sync {
    fn resolve(&self, path: &str) -> Result<ScopedPath, BusinessError>;
}

/// Receives bounded live output chunks while a call runs. Best effort: a
/// provider must never fail or block because of the sink.
pub trait ToolOutput: Send + Sync {
    fn chunk(&self, stream: &str, bytes: &[u8]);
}

/// Everything a provider may use while running one call.
pub struct ToolContext<'a> {
    pub project_id: &'a str,
    /// Canonical root of the opened project.
    pub project_root: &'a std::path::Path,
    /// Unique and stable for this call across retries and resumption;
    /// providers that mutate state use it as their idempotency key.
    pub call_key: &'a str,
    /// Upper bound the host places on this call's runtime, if any.
    pub timeout_ceiling: Option<std::time::Duration>,
    pub paths: &'a dyn PathScopes,
    /// Shared so blocking work on another thread can stream through it.
    pub output: std::sync::Arc<dyn ToolOutput>,
    pub cancellation: &'a dyn Cancellation,
}

/// One tool call routed to a provider.
/// One tool call routed to a provider.
pub struct ToolInvocation<'a> {
    pub name: &'a str,
    pub arguments: Value,
    /// The pin from this call's [`ApprovalTarget`], if it needed approval.
    pub approval_pin: Option<Value>,
}

/// What a user approves for one call.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalTarget {
    /// Short operation label shown to the user.
    pub operation: String,
    /// Arguments shown to the user.
    pub arguments: Value,
    /// Opaque provider state captured at approval time and handed back on
    /// execution, so a provider can refuse a call whose target changed
    /// after the user approved it.
    pub pin: Option<Value>,
}

/// A source of model-facing tools.
pub trait ToolProvider: Send + Sync {
    /// Whether this provider handles the tool `name`.
    fn owns(&self, name: &str) -> bool;

    /// Tools currently advertised for `project_id`.
    fn catalog<'a>(&'a self, project_id: &'a str) -> BoxFuture<'a, Vec<ToolDefinition>>;

    /// Validates arguments before policy evaluation and approval.
    fn validate(&self, name: &str, arguments: &Value) -> Result<(), BusinessError>;

    /// Risk of calling `name`; `None` means the call is denied.
    fn risk(&self, name: &str) -> Option<Risk>;

    /// What the user approves for this call. The default shows the tool name
    /// and its arguments; providers whose target can change between approval
    /// and execution return a pin.
    fn approval_target<'a>(
        &'a self,
        _project_id: &'a str,
        name: &'a str,
        arguments: &'a Value,
    ) -> BoxFuture<'a, Result<ApprovalTarget, BusinessError>> {
        Box::pin(async move {
            Ok(ApprovalTarget {
                operation: name.to_string(),
                arguments: arguments.clone(),
                pin: None,
            })
        })
    }

    /// Whether `name` accepts `dependency:` paths. Dependencies are read-only,
    /// so only tools that never write may opt in.
    fn allows_dependency_paths(&self, _name: &str) -> bool {
        false
    }

    /// Runs the call.
    fn execute<'a>(
        &'a self,
        invocation: ToolInvocation<'a>,
        context: ToolContext<'a>,
    ) -> BoxFuture<'a, Result<Value, BusinessError>>;
}
