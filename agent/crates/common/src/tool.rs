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

/// One tool call routed to a provider.
pub struct ToolInvocation<'a> {
    pub project_id: &'a str,
    pub name: &'a str,
    pub arguments: Value,
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

    /// Runs the call.
    fn execute<'a>(
        &'a self,
        invocation: ToolInvocation<'a>,
        cancellation: &'a dyn Cancellation,
    ) -> BoxFuture<'a, Result<Value, BusinessError>>;
}
