use crate::{
    context,
    domain::{Message, SessionEvent, ToolCall, Usage},
    logging,
    policy::{evaluate, tool_risk, Decision, Risk},
};
use base64::{engine::general_purpose::STANDARD, Engine};
use futures_util::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{HashMap, VecDeque},
    fs,
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use suncode_common::BusinessError;
use suncode_data::{ApprovalInput, Store};
use suncode_llm::{CompletionRequest, ModelProviderRegistry, ModelRoute};
use tokio::sync::{broadcast, mpsc, Mutex as AsyncMutex};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

const DEFAULT_TOOL_CALL_LIMIT: u32 = 64;
const MAX_INSTRUCTION_FILE_BYTES: u64 = 32 * 1024;
const MAX_NEARBY_INSTRUCTION_FILES: usize = 16;
const MAX_NEARBY_INSTRUCTION_BYTES: usize = 64 * 1024;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Continuation {
    session_id: String,
    #[serde(default)]
    session_started_at: String,
    project_id: String,
    project_root: String,
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
    remaining_calls: Vec<ToolCall>,
    #[serde(default)]
    context_compacted: bool,
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
}

#[derive(Debug, Clone)]
struct QueuedMessage {
    queued_id: String,
    idempotency_key: String,
    input: String,
    image_ids: Vec<String>,
}

#[derive(Clone)]
pub struct Agent {
    store: Store,
    providers: Arc<ModelProviderRegistry>,
    operations: Arc<suncode_tool::Operations>,
    events: broadcast::Sender<SessionEvent>,
    cancellations: Arc<Mutex<HashMap<String, CancellationToken>>>,
    active_turns: Arc<Mutex<HashMap<String, String>>>,
    queued_messages: Arc<Mutex<HashMap<String, VecDeque<QueuedMessage>>>>,
    non_interactive: bool,
    session_locks: Arc<AsyncMutex<HashMap<String, Arc<AsyncMutex<()>>>>>,
}

include!("submission.rs");
include!("continuations.rs");
include!("run.rs");
include!("tools.rs");
include!("lifecycle.rs");
include!("support.rs");
#[cfg(test)]
include!("tests.rs");
