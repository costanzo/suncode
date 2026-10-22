use crate::{
    context,
    policy::{Decision, Risk, evaluate, tool_risk},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use futures_util::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, VecDeque},
    fs,
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use suncode_common::BusinessError;
use suncode_common::logging;
use suncode_config::{Config, RuntimeConfig};
use suncode_data::{ApprovalInput, Store};
use suncode_data::{Message, ToolCall, Usage};
use suncode_llm::{CompletionRequest, ModelProviderRegistry, ModelRoute};
use tokio::sync::{Mutex as AsyncMutex, mpsc};
use tokio_util::sync::CancellationToken;

use events::*;
pub use suncode_events::payload as events;
pub use suncode_events::{
    AgentAttentionEvent, AgentEventSubscription, AgentEventSubscriptionControl, AttentionEventHub,
    AttentionEventSubscription, AttentionEventSubscriptionControl, AttentionKind,
    AttentionReceiveError, EventReceiveError, SessionEventHub,
};
mod lsp;
mod system_prompt;
pub use lsp::{
    LanguageServerRuntimeState, LanguageServerRuntimeStatus, match_root_marker,
    validate_root_marker,
};
mod browser;
mod preview;
pub use browser::{
    BrowserHostCallback, BrowserInstallationState, BrowserRuntimeInfo, BrowserRuntimeState,
    BrowserVisibilityCapability,
};
pub use preview::{PreviewState, PreviewStatus};
mod computer;
pub use computer::ComputerRuntimeInfo;
mod mcp;
pub use mcp::{McpLoadProgress, McpRuntimeState, McpRuntimeStatus};
pub use suncode_agents::{AgentDefinition, ModelPolicy, ToolScope};

const DEFAULT_TOOL_CALL_LIMIT: u32 = 64;
const MAX_INSTRUCTION_FILE_BYTES: u64 = 32 * 1024;
const MAX_NEARBY_INSTRUCTION_FILES: usize = 16;
const MAX_NEARBY_INSTRUCTION_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentHostCapabilities {
    pub browser_use: bool,
    pub computer_use: bool,
}

impl Default for AgentHostCapabilities {
    fn default() -> Self {
        Self {
            browser_use: true,
            computer_use: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct TodoEntry {
    content: String,
    status: String,
    priority: String,
}

fn default_tool_call_limit() -> u32 {
    DEFAULT_TOOL_CALL_LIMIT
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum TurnResponse {
    Completed {
        turn_id: String,
        message: Message,
        usage: Usage,
        iterations: u32,
        tool_calls: u32,
    },
    AwaitingApproval {
        turn_id: String,
        tool_call_id: String,
        approval_id: String,
    },
    AwaitingQuestion {
        turn_id: String,
        tool_call_id: String,
        request_id: String,
    },
    Queued {
        queued_id: String,
        active_turn_id: String,
        position: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QueuedMessageMode {
    Followup,
    Steering,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueuedMessageSnapshot {
    pub queued_id: String,
    pub position: usize,
    pub mode: QueuedMessageMode,
    pub input: String,
    pub image_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Continuation {
    session_id: String,
    #[serde(default)]
    session_started_at: String,
    project_id: String,
    project_root: String,
    #[serde(default)]
    agent_id: Option<String>,
    #[serde(default)]
    agent_version: Option<i64>,
    #[serde(default)]
    allowed_tools: Vec<String>,
    turn_id: String,
    submission_key: String,
    model: String,
    #[serde(default)]
    reasoning_effort: Option<String>,
    messages: Vec<Message>,
    iterations: u32,
    tool_calls: u32,
    #[serde(default = "default_tool_call_limit")]
    tool_call_limit: u32,
    usage: Usage,
    pending_call: Option<ToolCall>,
    /// The provider's approval pin for `pending_call`. Snapshots taken before
    /// the generic pin stored an MCP generation number under the old name.
    #[serde(default, alias = "pending_mcp_generation")]
    pending_approval_pin: Option<Value>,
    /// Calls already approved as part of the current approval batch.
    /// Snapshots taken before generic batches used the computer-specific names.
    #[serde(default, alias = "approved_computer_call_ids")]
    approved_batch_call_ids: Vec<String>,
    /// Set when a call in the current batch failed; the rest are skipped.
    #[serde(default, alias = "computer_batch_failed")]
    approval_batch_failed: bool,
    remaining_calls: Vec<ToolCall>,
    #[serde(default)]
    overflow_recovery_attempted: bool,
    #[serde(default)]
    last_tool_signature: Option<String>,
    #[serde(default)]
    repeated_tool_stalls: u32,
    #[serde(default)]
    active_call_id: Option<String>,
    #[serde(default)]
    question_answers: Option<Vec<Vec<String>>>,
    #[serde(default)]
    question_rejected: bool,
    #[serde(default)]
    todos: Vec<TodoEntry>,
    #[serde(default)]
    loaded_instruction_paths: Vec<String>,
    #[serde(default = "default_turn_timeout_ms")]
    turn_timeout_ms: u64,
    #[serde(default)]
    bash_timeout_ms: Option<u64>,
    #[serde(default)]
    full_control_override: Option<bool>,
    #[serde(default)]
    host_tool_allowlist: Vec<String>,
}

impl AgentRuntime {
    /// Agents the primary agent may delegate to, in catalog order.
    pub fn list_agents(&self) -> &'static [AgentDefinition] {
        suncode_agents::specialists()
    }
}

impl Continuation {
    /// The agent running this turn. Sessions without an agent run the primary
    /// agent. An id that no longer resolves gets no capabilities rather than
    /// the primary's; its tool scope still comes from the turn snapshot.
    fn agent(&self) -> &'static suncode_agents::AgentDefinition {
        suncode_agents::by_id(self.agent_id.as_deref()).unwrap_or(&suncode_agents::UNRESOLVED)
    }
}

#[derive(Debug, Clone)]
struct QueuedMessage {
    queued_id: String,
    idempotency_key: String,
    input: String,
    image_ids: Vec<String>,
    mode: QueuedMessageMode,
}

/// The process-wide host that runs every session's turns, for the primary and
/// specialist agents alike. Who an agent is lives in
/// [`suncode_agents::AgentDefinition`]; this type owns sessions, submission,
/// lifecycle, and the shared runtimes, and runs each turn through a
/// [`turn_loop::TurnLoop`].
#[derive(Clone)]
pub struct AgentRuntime {
    services: services::TurnServices,
    cancellations: Arc<Mutex<HashMap<String, CancellationToken>>>,
    active_turns: Arc<Mutex<HashMap<String, String>>>,
    session_locks: Arc<AsyncMutex<HashMap<String, Arc<AsyncMutex<()>>>>>,
    mcp: suncode_mcp::McpRuntime,
    lsp: suncode_lsp::LspRuntime,
    browser: suncode_browser::BrowserRuntime,
    preview: preview::PreviewManager,
    computer: suncode_computer::ComputerRuntime,
    shutting_down: Arc<std::sync::atomic::AtomicBool>,
}

/// Runs synchronous SQLite work from async code without stalling other tasks.
///
/// On a multi-threaded runtime the worker hands its queued tasks to another
/// thread first. Elsewhere (current-thread runtimes, plain threads) the work runs
/// inline, which is no worse than calling the store directly.
pub(super) fn db_blocking<T>(work: impl FnOnce() -> T) -> T {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread => {
            tokio::task::block_in_place(work)
        }
        _ => work(),
    }
}

fn default_turn_timeout_ms() -> u64 {
    RuntimeConfig::DEFAULT_TURN_TIMEOUT_MS
}

mod continuations;
mod lifecycle;
mod providers;
mod remote;
mod run;
mod services;
mod states;
mod submission;
mod support;
mod tools;
mod turn_loop;
use states::{ToolState, TurnState};
use support::*;
use turn_loop::TurnLoop;
#[cfg(test)]
mod tests;
