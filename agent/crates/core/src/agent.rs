use crate::{
    context,
    domain::{Message, SessionEvent, ToolCall, Usage},
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
use suncode_db::{ApprovalInput, PersistenceError, Store};
use suncode_llm::{CompletionRequest, ModelProviderRegistry, ModelRoute, ProviderError};
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

#[derive(Debug)]
pub struct AgentError {
    pub code: String,
    pub message: String,
    pub details: Value,
}

impl AgentError {
    fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: json!({}),
        }
    }
    fn details(mut self, details: Value) -> Self {
        self.details = details;
        self
    }
}

impl From<PersistenceError> for AgentError {
    fn from(error: PersistenceError) -> Self {
        Self::new("agent_unavailable", error.to_string())
    }
}
impl From<ProviderError> for AgentError {
    fn from(error: ProviderError) -> Self {
        Self::new(&error.code, error.message).details(json!({
            "retryable": error.retryable,
            "provider_request_id": error.provider_request_id,
        }))
    }
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

impl Agent {
    pub fn new<P>(
        store: Store,
        providers: P,
        operations: Arc<suncode_tool::Operations>,
        events: broadcast::Sender<SessionEvent>,
        non_interactive: bool,
    ) -> Self
    where
        P: Into<Arc<ModelProviderRegistry>>,
    {
        Self {
            store,
            providers: providers.into(),
            operations,
            events,
            cancellations: Arc::new(Mutex::new(HashMap::new())),
            active_turns: Arc::new(Mutex::new(HashMap::new())),
            queued_messages: Arc::new(Mutex::new(HashMap::new())),
            non_interactive,
            session_locks: Arc::new(AsyncMutex::new(HashMap::new())),
        }
    }

    pub async fn submit(
        &self,
        session_id: &str,
        key: &str,
        input: &str,
        model: Option<&str>,
    ) -> Result<TurnResponse, AgentError> {
        let session_lock = self.session_lock(session_id).await;
        let model = match model {
            Some(model) => model.to_string(),
            None => {
                let session = self
                    .store
                    .session_by_id(session_id)?
                    .ok_or_else(|| AgentError::new("not_found", "session not found"))?;
                let project_id = session.project_id.ok_or_else(|| {
                    AgentError::new("conflict", "session is not bound to a project")
                })?;
                self.store
                    .project_default_model(&project_id)?
                    .unwrap_or_else(|| "deepseek-v4-flash".into())
            }
        };
        let Some(provider) = self.providers.route(&model) else {
            return Err(AgentError::new(
                "model_unavailable",
                "model is not advertised",
            ));
        };
        let _guard = match session_lock.try_lock() {
            Ok(guard) => guard,
            Err(_) => {
                let has_active_turn = self
                    .active_turns
                    .lock()
                    .map_err(|_| AgentError::new("agent_unavailable", "turn state unavailable"))?
                    .contains_key(session_id);
                if has_active_turn {
                    return self.queue_message(session_id, key, input);
                }
                session_lock.lock().await
            }
        };
        let session = self
            .store
            .session_by_id(session_id)?
            .ok_or_else(|| AgentError::new("not_found", "session not found"))?;
        let session_started_at = session.created_at.clone();
        let project_id = session
            .project_id
            .ok_or_else(|| AgentError::new("conflict", "session is not bound to a project"))?;
        let project = self
            .store
            .project_by_id(&project_id)?
            .ok_or_else(|| AgentError::new("not_found", "project not found"))?;
        let tool_call_limit = self
            .store
            .project_tool_call_limit(&project_id)?
            .unwrap_or(DEFAULT_TOOL_CALL_LIMIT);
        let admission = self.store.begin_turn(session_id, key, input, &model)?;
        if !admission.created {
            if admission.status == "completed" {
                let response = admission.response.ok_or_else(|| {
                    AgentError::new("agent_unavailable", "completed turn has no response")
                })?;
                return serde_json::from_value(response).map_err(|_| {
                    AgentError::new("agent_unavailable", "stored turn response is invalid")
                });
            }
            return Err(AgentError::new(
                "idempotency_conflict",
                format!("turn submission is {}", admission.status),
            ));
        }
        self.store.mark_turn_started(session_id, key)?;
        let token = CancellationToken::new();
        self.cancellations
            .lock()
            .map_err(|_| AgentError::new("agent_unavailable", "cancellation state unavailable"))?
            .insert(admission.turn_id.clone(), token.clone());
        self.active_turns
            .lock()
            .map_err(|_| AgentError::new("agent_unavailable", "turn state unavailable"))?
            .insert(session_id.to_string(), admission.turn_id.clone());
        let continuation = Continuation {
            session_id: session_id.into(),
            session_started_at,
            project_id,
            project_root: project.canonical_root,
            turn_id: admission.turn_id.clone(),
            submission_key: key.into(),
            model,
            messages: self.store.context_messages(session_id)?,
            iterations: 0,
            tool_calls: 0,
            tool_call_limit,
            usage: Usage::default(),
            pending_call: None,
            remaining_calls: Vec::new(),
            context_compacted: false,
            last_tool_signature: None,
            repeated_tool_stalls: 0,
            active_call_id: None,
            question_answers: None,
            question_rejected: false,
            todos: Vec::new(),
            loaded_instruction_paths: Vec::new(),
        };
        let result = self
            .run(continuation.clone(), Some(input), token, provider)
            .await;
        self.cancellations
            .lock()
            .ok()
            .map(|mut values| values.remove(&admission.turn_id));
        self.active_turns
            .lock()
            .ok()
            .map(|mut values| values.remove(session_id));
        if let Err(error) = &result {
            if !matches!(
                error.code.as_str(),
                "approval_required" | "question_required"
            ) {
                self.clear_queued_messages(session_id);
                let _ = self.turn_state(
                    &continuation,
                    if error.code == "cancelled" {
                        "cancelled"
                    } else {
                        "failed"
                    },
                    Some(&error.code),
                );
                self.store.fail_turn(
                    session_id,
                    key,
                    &json!({"code":error.code,"message":error.message,"details":error.details}),
                )?;
            }
        }
        result
    }

    fn clear_queued_messages(&self, session_id: &str) {
        self.queued_messages
            .lock()
            .ok()
            .map(|mut values| values.remove(session_id));
    }

    fn queue_message(
        &self,
        session_id: &str,
        key: &str,
        input: &str,
    ) -> Result<TurnResponse, AgentError> {
        let active_turn_id = self
            .active_turns
            .lock()
            .map_err(|_| AgentError::new("agent_unavailable", "turn state unavailable"))?
            .get(session_id)
            .cloned()
            .ok_or_else(|| AgentError::new("conflict", "session is busy"))?;
        let mut queues = self
            .queued_messages
            .lock()
            .map_err(|_| AgentError::new("agent_unavailable", "turn queue unavailable"))?;
        let queue = queues.entry(session_id.to_string()).or_default();
        if let Some((index, existing)) = queue
            .iter()
            .enumerate()
            .find(|(_, value)| value.idempotency_key == key)
        {
            return Ok(TurnResponse::Queued {
                queued_id: existing.queued_id.clone(),
                active_turn_id,
                position: index + 1,
            });
        }
        let queued_id = Uuid::new_v4().to_string();
        queue.push_back(QueuedMessage {
            queued_id: queued_id.clone(),
            idempotency_key: key.to_string(),
            input: input.to_string(),
        });
        let position = queue.len();
        drop(queues);
        self.emit(
            session_id,
            "turn.queued",
            json!({"queued_id": queued_id, "active_turn_id": active_turn_id, "position": position}),
        )?;
        Ok(TurnResponse::Queued {
            queued_id,
            active_turn_id,
            position,
        })
    }

    pub fn cancel(&self, turn_id: &str) -> bool {
        let Some(token) = self
            .cancellations
            .lock()
            .ok()
            .and_then(|values| values.get(turn_id).cloned())
        else {
            return false;
        };
        token.cancel();
        true
    }

    pub async fn resolve_approval(
        &self,
        approval_id: &str,
        decision: &str,
    ) -> Result<bool, AgentError> {
        let Some(suspended) = self.store.resolve_approval(approval_id, decision)? else {
            return Ok(false);
        };
        let mut continuation: Continuation =
            serde_json::from_value(suspended.snapshot).map_err(|_| {
                AgentError::new("agent_unavailable", "approval continuation is invalid")
            })?;
        self.emit(
            &continuation.session_id,
            "approval.resolved",
            json!({"approval_id":approval_id,"turn_id":continuation.turn_id,"decision":decision}),
        )?;
        if decision == "deny" {
            self.clear_queued_messages(&continuation.session_id);
            if let Some(call) = continuation.pending_call.take() {
                self.tool_state(&continuation, &call, "denied", Some("user_denied"))?;
            }
            self.turn_state(&continuation, "failed", Some("authorization_denied"))?;
            self.store.fail_turn(
                &continuation.session_id,
                &continuation.submission_key,
                &json!({"code":"authorization_denied","message":"Approval was denied"}),
            )?;
            self.store.finish_suspended(approval_id, "denied")?;
            return Ok(true);
        }
        let token = CancellationToken::new();
        self.cancellations
            .lock()
            .map_err(|_| AgentError::new("agent_unavailable", "cancellation state unavailable"))?
            .insert(continuation.turn_id.clone(), token.clone());
        self.active_turns
            .lock()
            .map_err(|_| AgentError::new("agent_unavailable", "turn state unavailable"))?
            .insert(
                continuation.session_id.clone(),
                continuation.turn_id.clone(),
            );
        let agent = self.clone();
        let approval_id = approval_id.to_string();
        tokio::spawn(async move {
            let session_lock = agent.session_lock(&continuation.session_id).await;
            let _guard = session_lock.lock().await;
            let result = agent.continue_approved(&mut continuation, token).await;
            let status = if result.is_ok() {
                "completed"
            } else {
                "failed"
            };
            let _ = agent.store.finish_suspended(&approval_id, status);
            if let Err(error) = &result {
                agent.clear_queued_messages(&continuation.session_id);
                let _ = agent.turn_state(
                    &continuation,
                    if error.code == "cancelled" {
                        "cancelled"
                    } else {
                        "failed"
                    },
                    Some(&error.code),
                );
                let _ = agent.store.fail_turn(
                    &continuation.session_id,
                    &continuation.submission_key,
                    &json!({"code":error.code,"message":error.message,"details":error.details}),
                );
            }
            agent
                .cancellations
                .lock()
                .ok()
                .map(|mut values| values.remove(&continuation.turn_id));
            agent
                .active_turns
                .lock()
                .ok()
                .map(|mut values| values.remove(&continuation.session_id));
        });
        Ok(true)
    }

    pub async fn resolve_question(
        &self,
        request_id: &str,
        answers: Vec<Vec<String>>,
        rejected: bool,
    ) -> Result<bool, AgentError> {
        let snapshot = self.store.question_snapshot(request_id)?.ok_or_else(|| {
            AgentError::new("conflict", "question is missing or already resolved")
        })?;
        if !rejected {
            let continuation: Continuation = serde_json::from_value(snapshot).map_err(|_| {
                AgentError::new("agent_unavailable", "question continuation is invalid")
            })?;
            let call = continuation
                .pending_call
                .as_ref()
                .ok_or_else(|| AgentError::new("agent_unavailable", "question call is missing"))?;
            validate_question_answers(&call.arguments, &answers)?;
        }
        let Some(suspended) = self
            .store
            .resolve_question(request_id, &answers, rejected)?
        else {
            return Ok(false);
        };
        let mut continuation: Continuation =
            serde_json::from_value(suspended.snapshot).map_err(|_| {
                AgentError::new("agent_unavailable", "question continuation is invalid")
            })?;
        let call = continuation
            .pending_call
            .as_ref()
            .ok_or_else(|| AgentError::new("agent_unavailable", "question call is missing"))?;
        let event = if rejected {
            "question.rejected"
        } else {
            "question.replied"
        };
        self.emit(
            &continuation.session_id,
            event,
            json!({"request_id":request_id,"turn_id":continuation.turn_id,"tool_call_id":call.call_id,"answers":answers}),
        )?;
        let token = CancellationToken::new();
        self.cancellations
            .lock()
            .map_err(|_| AgentError::new("agent_unavailable", "cancellation state unavailable"))?
            .insert(continuation.turn_id.clone(), token.clone());
        self.active_turns
            .lock()
            .map_err(|_| AgentError::new("agent_unavailable", "turn state unavailable"))?
            .insert(
                continuation.session_id.clone(),
                continuation.turn_id.clone(),
            );
        let agent = self.clone();
        let request_id = request_id.to_string();
        tokio::spawn(async move {
            let session_lock = agent.session_lock(&continuation.session_id).await;
            let _guard = session_lock.lock().await;
            let result = agent.continue_question(&mut continuation, token).await;
            let suspended_again = result.as_ref().err().is_some_and(|error| {
                matches!(
                    error.code.as_str(),
                    "approval_required" | "question_required"
                )
            });
            let status = if result.is_ok() || suspended_again {
                "completed"
            } else {
                "failed"
            };
            let _ = agent.store.finish_suspended(&request_id, status);
            if let Err(error) = &result {
                if !suspended_again {
                    agent.clear_queued_messages(&continuation.session_id);
                    let _ = agent.turn_state(
                        &continuation,
                        if error.code == "cancelled" {
                            "cancelled"
                        } else {
                            "failed"
                        },
                        Some(&error.code),
                    );
                    let _ = agent.store.fail_turn(
                        &continuation.session_id,
                        &continuation.submission_key,
                        &json!({"code":error.code,"message":error.message,"details":error.details}),
                    );
                }
            }
            agent
                .cancellations
                .lock()
                .ok()
                .map(|mut values| values.remove(&continuation.turn_id));
            agent
                .active_turns
                .lock()
                .ok()
                .map(|mut values| values.remove(&continuation.session_id));
        });
        Ok(true)
    }

    async fn continue_question(
        &self,
        continuation: &mut Continuation,
        token: CancellationToken,
    ) -> Result<TurnResponse, AgentError> {
        if let Some(call) = continuation.pending_call.take() {
            let answers = continuation.question_answers.take().unwrap_or_default();
            let result = json!({"answers": answers, "rejected": continuation.question_rejected});
            self.tool_state(continuation, &call, "succeeded", None)?;
            self.emit(&continuation.session_id, "tool.result", json!({"turn_id":continuation.turn_id,"call_id":continuation.active_call_id,"tool_call_id":call.call_id,"result":result}))?;
            let mut tool = Message::text(
                "tool",
                serde_json::to_string(&result).unwrap_or_else(|_| "{}".into()),
            );
            tool.tool_call_id = Some(call.call_id.clone());
            continuation.messages.push(tool.clone());
            self.emit(&continuation.session_id, "message.tool", json!({"turn_id":continuation.turn_id,"call_id":continuation.active_call_id,"tool_call_id":call.call_id,"message":tool}))?;
        }
        let siblings = std::mem::take(&mut continuation.remaining_calls);
        self.resolve_calls(continuation, siblings, token.clone())
            .await?;
        let provider = self
            .providers
            .route(&continuation.model)
            .ok_or_else(|| AgentError::new("model_unavailable", "model is not advertised"))?;
        let response = self
            .run(continuation.clone(), None, token, provider)
            .await?;
        self.store.complete_turn(
            &continuation.session_id,
            &continuation.submission_key,
            &serde_json::to_value(&response).map_err(|_| {
                AgentError::new("agent_unavailable", "turn response could not be stored")
            })?,
        )?;
        Ok(response)
    }

    async fn continue_approved(
        &self,
        continuation: &mut Continuation,
        token: CancellationToken,
    ) -> Result<TurnResponse, AgentError> {
        if let Some(call) = continuation.pending_call.take() {
            self.execute_call(continuation, &call, token.clone())
                .await?;
        }
        let siblings = std::mem::take(&mut continuation.remaining_calls);
        self.resolve_calls(continuation, siblings, token.clone())
            .await?;
        let provider = self
            .providers
            .route(&continuation.model)
            .ok_or_else(|| AgentError::new("model_unavailable", "model is not advertised"))?;
        let response = self
            .run(continuation.clone(), None, token, provider)
            .await?;
        self.store.complete_turn(
            &continuation.session_id,
            &continuation.submission_key,
            &serde_json::to_value(&response).map_err(|_| {
                AgentError::new("agent_unavailable", "turn response could not be stored")
            })?,
        )?;
        Ok(response)
    }

    async fn run(
        &self,
        mut context: Continuation,
        user_input: Option<&str>,
        token: CancellationToken,
        provider: ModelRoute,
    ) -> Result<TurnResponse, AgentError> {
        let started = Instant::now();
        if let Some(input) = user_input {
            self.turn_state(&context, "admitted", None)?;
            let message = Message::text("user", input);
            context.messages.push(message.clone());
            self.emit(
                &context.session_id,
                "message.user",
                json!({"message_id":Uuid::new_v4(),"turn_id":context.turn_id,"message":message}),
            )?;
        }
        self.turn_state(&context, "preparing", None)?;
        while context.iterations < 32 {
            if token.is_cancelled() {
                return self.fail_context(&context, "cancelled", "Turn was cancelled");
            }
            if started.elapsed() > Duration::from_secs(600) {
                return self.fail_context(
                    &context,
                    "turn_timeout",
                    "Turn exceeded its wall-clock budget",
                );
            }
            self.drain_queued_messages(&mut context)?;
            context.iterations += 1;
            self.turn_state(&context, "calling_model", None)?;
            let prompt = context::build_for_model(
                &context.messages,
                self.providers
                    .limits(&context.model)
                    .and_then(|limits| limits.max_input_tokens),
                self.providers
                    .limits(&context.model)
                    .and_then(|limits| limits.auto_compact_tokens),
            );
            if prompt.compacted && !context.context_compacted {
                context.context_compacted = true;
                self.emit(
                    &context.session_id,
                    "context.compacted",
                    json!({
                        "turn_id": context.turn_id,
                        "original_characters": prompt.original_characters,
                        "retained_characters": prompt.retained_characters,
                        "original_tokens": prompt.original_tokens,
                        "retained_tokens": prompt.retained_tokens,
                        "dropped_messages": prompt.dropped_messages,
                        "summary": prompt.summary,
                    }),
                )?;
            }
            if prompt.compacted {
                context.messages = prompt.messages;
            }
            let exchange_id = Uuid::new_v4().to_string();
            context.active_call_id = Some(exchange_id.clone());
            let mut llm_messages = vec![host_environment_message(&context.session_started_at)];
            if let Some(message) = project_instruction_message(&context.project_root) {
                llm_messages.push(message);
            }
            if let Some(message) = self.dependency_context_message(&context.project_id)? {
                llm_messages.push(message);
            }
            llm_messages.extend(
                context
                    .messages
                    .iter()
                    .map(to_llm_message)
                    .collect::<Vec<_>>(),
            );
            self.emit(
                &context.session_id,
                "provider.exchange.started",
                json!({
                    "exchange_id": exchange_id,
                    "turn_id": context.turn_id,
                    "provider": provider.provider_id,
                    "model_id": context.model,
                    "wire_model": provider.wire_model,
                    "iteration": context.iterations,
                    "input_messages": llm_messages,
                }),
            )?;
            let result = {
                let (delta_sender, mut delta_receiver) = mpsc::unbounded_channel();
                let tool_definitions = crate::tools::definitions();
                let provider_call = provider.provider.complete(
                    CompletionRequest {
                        messages: &llm_messages,
                        wire_model: &provider.wire_model,
                        tools: &tool_definitions,
                    },
                    &token,
                    delta_sender,
                );
                tokio::pin!(provider_call);
                let result = loop {
                    tokio::select! {
                        value = &mut provider_call => break value,
                        Some(delta) = delta_receiver.recv() => {
                            self.emit_live(&context.session_id, "assistant.delta", json!({"turn_id":context.turn_id,"text":delta}));
                        }
                    }
                };
                while let Ok(delta) = delta_receiver.try_recv() {
                    self.emit_live(
                        &context.session_id,
                        "assistant.delta",
                        json!({"turn_id":context.turn_id,"text":delta}),
                    );
                }
                result
            };
            let result = match result {
                Ok(result) => result,
                Err(error) => {
                    self.emit(
                        &context.session_id,
                        "provider.exchange.failed",
                        json!({
                            "exchange_id": exchange_id,
                            "turn_id": context.turn_id,
                            "error": {
                                "code": error.code,
                                "message": error.message.clone(),
                                "retryable": error.retryable,
                            },
                            "provider_request_id": error.provider_request_id,
                        }),
                    )?;
                    return Err(error.into());
                }
            };
            if result.text.len() > 8 * 1024 * 1024 {
                return self.fail_context(
                    &context,
                    "output_budget_exceeded",
                    "Provider output exceeded its budget",
                );
            }
            let usage = result.usage.as_ref().map(|usage| Usage {
                input_tokens: usage.input_tokens,
                output_tokens: usage.output_tokens,
                total_tokens: usage.total_tokens,
            });
            let cache_read_tokens = result
                .usage
                .as_ref()
                .and_then(|usage| usage.cache_read_tokens);
            let cache_miss_tokens = result
                .usage
                .as_ref()
                .and_then(|usage| usage.cache_miss_tokens);
            let cache_write_tokens = result
                .usage
                .as_ref()
                .and_then(|usage| usage.cache_write_tokens);
            let reasoning_tokens = result
                .usage
                .as_ref()
                .and_then(|usage| usage.reasoning_tokens);
            let tool_calls = result
                .tool_calls
                .iter()
                .map(|call| ToolCall {
                    call_id: call.call_id.clone(),
                    name: call.name.clone(),
                    arguments: call.arguments.clone(),
                })
                .collect::<Vec<_>>();
            if let Some(usage) = &usage {
                context.usage.add(usage);
                self.emit(
                    &context.session_id,
                    "usage.updated",
                    json!({"turn_id":context.turn_id,"usage":context.usage}),
                )?;
            }
            let assistant = Message {
                role: "assistant".into(),
                content: if result.text.is_empty() {
                    Vec::new()
                } else {
                    Message::text("assistant", result.text).content
                },
                tool_calls: tool_calls.clone(),
                tool_call_id: None,
            };
            self.emit(
                &context.session_id,
                "provider.exchange.completed",
                json!({
                    "exchange_id": exchange_id,
                    "turn_id": context.turn_id,
                    "output_message": assistant,
                    "tool_calls": tool_calls.clone(),
                    "usage": usage.as_ref().map(|usage| json!({
                        "input_tokens": usage.input_tokens,
                        "output_tokens": usage.output_tokens,
                        "total_tokens": usage.total_tokens,
                        "cache_read_tokens": cache_read_tokens,
                        "cache_miss_tokens": cache_miss_tokens,
                        "cache_write_tokens": cache_write_tokens,
                        "reasoning_tokens": reasoning_tokens,
                    })),
                    "provider_request_id": result.provider_request_id,
                    "provider_response_id": result.provider_response_id,
                    "finish_reason": result.finish_reason.clone(),
                }),
            )?;
            context.messages.push(assistant.clone());
            self.emit(&context.session_id, "message.assistant", json!({"message_id":Uuid::new_v4(),"turn_id":context.turn_id,"call_id":context.active_call_id,"message":assistant,"usage":context.usage,"finish_reason":result.finish_reason}))?;
            if tool_calls.is_empty() {
                if self.drain_queued_messages(&mut context)? {
                    self.turn_state(&context, "preparing", None)?;
                    continue;
                }
                self.turn_state(&context, "completed", None)?;
                self.emit(&context.session_id, "turn.completed", json!({"turn_id":context.turn_id,"usage":context.usage,"iterations":context.iterations,"tool_calls":context.tool_calls}))?;
                let response = TurnResponse::Completed {
                    turn_id: context.turn_id.clone(),
                    message: assistant,
                    usage: context.usage.clone(),
                    iterations: context.iterations,
                    tool_calls: context.tool_calls,
                };
                self.store.complete_turn(
                    &context.session_id,
                    &context.submission_key,
                    &serde_json::to_value(&response).map_err(|_| {
                        AgentError::new("agent_unavailable", "turn response could not be stored")
                    })?,
                )?;
                return Ok(response);
            }
            self.turn_state(&context, "resolving_calls", None)?;
            self.resolve_calls(&mut context, tool_calls, token.clone())
                .await?;
            self.drain_queued_messages(&mut context)?;
            self.turn_state(&context, "preparing", None)?;
        }
        self.fail_context(
            &context,
            "iteration_budget_exceeded",
            "Turn exceeded its iteration budget",
        )
    }

    fn drain_queued_messages(&self, context: &mut Continuation) -> Result<bool, AgentError> {
        let queued = {
            let mut queues = self
                .queued_messages
                .lock()
                .map_err(|_| AgentError::new("agent_unavailable", "turn queue unavailable"))?;
            queues.remove(&context.session_id).unwrap_or_default()
        };
        if queued.is_empty() {
            return Ok(false);
        }
        for item in queued {
            let message = Message::text("user", item.input);
            context.messages.push(message.clone());
            self.emit(
                &context.session_id,
                "message.user",
                json!({
                    "message_id": Uuid::new_v4(),
                    "turn_id": context.turn_id,
                    "queued_id": item.queued_id,
                    "queued_idempotency_key": item.idempotency_key,
                    "message": message
                }),
            )?;
        }
        Ok(true)
    }

    async fn resolve_calls(
        &self,
        context: &mut Continuation,
        calls: Vec<ToolCall>,
        token: CancellationToken,
    ) -> Result<(), AgentError> {
        let batch_size = u32::try_from(calls.len()).unwrap_or(u32::MAX);
        let next_tool_calls = context.tool_calls.saturating_add(batch_size);
        if next_tool_calls > context.tool_call_limit {
            context.tool_calls = next_tool_calls;
            for (index, call) in calls.iter().enumerate() {
                self.emit(
                    &context.session_id,
                    "tool.requested",
                    json!({
                        "turn_id": context.turn_id,
                        "call_id": context.active_call_id,
                        "tool_call_id": call.call_id,
                        "name": call.name,
                        "arguments": call.arguments,
                        "ordinal": index
                    }),
                )?;
                self.tool_state(context, call, "failed", Some("tool_budget_exceeded"))?;
            }
            self.turn_state(context, "failed", Some("tool_budget_exceeded"))?;
            return Err(AgentError::new(
                "tool_budget_exceeded",
                "Turn exceeded its tool-call budget",
            )
            .details(json!({
                "limit": context.tool_call_limit,
                "requested_total": context.tool_calls,
                "rejected_batch_size": batch_size
            })));
        }
        context.tool_calls = next_tool_calls;
        let mut allowed_calls = Vec::new();
        for (index, call) in calls.iter().enumerate() {
            self.emit(&context.session_id, "tool.requested", json!({"turn_id":context.turn_id,"call_id":context.active_call_id,"tool_call_id":call.call_id,"name":call.name,"arguments":call.arguments,"ordinal":index}))?;
            let signature = tool_signature(call);
            if context.last_tool_signature.as_deref() == Some(signature.as_str()) {
                context.repeated_tool_stalls += 1;
            } else {
                context.last_tool_signature = Some(signature);
                context.repeated_tool_stalls = 1;
            }
            if context.repeated_tool_stalls >= 4 {
                self.tool_state(context, call, "failed", Some("repeated_equivalent_call"))?;
                return self.fail_context(
                    context,
                    "tool_stall_detected",
                    "Repeated equivalent tool calls indicate a stalled turn",
                );
            }
            self.tool_state(context, call, "requested", None)?;
            self.tool_state(context, call, "validating", None)?;
            if !call.arguments.is_object() {
                let error =
                    AgentError::new("malformed_tool_call", "Tool arguments must be an object");
                if !self.record_recoverable_call_error(context, call, &error)? {
                    return Err(error);
                }
                continue;
            }
            if let Err(error) = self.validate_dependency_call(context, call) {
                if !self.record_recoverable_call_error(context, call, &error)? {
                    return Err(error);
                }
                continue;
            }
            if let Err(error) = validate_before_policy(&call.name, &call.arguments) {
                if !self.record_recoverable_call_error(context, call, &error)? {
                    return Err(error);
                }
                continue;
            }
            if call.name == "question" {
                self.execute_allowed_calls(
                    context,
                    std::mem::take(&mut allowed_calls),
                    token.clone(),
                )
                .await?;
                self.tool_state(
                    context,
                    call,
                    "awaiting_question",
                    Some("user_input_required"),
                )?;
                let request_id = format!("que_{}", Uuid::new_v4());
                context.pending_call = Some(call.clone());
                context.remaining_calls = calls[index + 1..].to_vec();
                let snapshot = serde_json::to_value(&*context).map_err(|_| {
                    AgentError::new("agent_unavailable", "turn continuation could not be stored")
                })?;
                self.store
                    .create_question(&request_id, &context.turn_id, &snapshot)?;
                self.emit(&context.session_id, "question.asked", json!({"request_id":request_id,"turn_id":context.turn_id,"tool_call_id":call.call_id,"questions":call.arguments["questions"]}))?;
                return Err(AgentError::new("question_required", "The user must answer the question tool").details(json!({"turn_id":context.turn_id,"tool_call_id":call.call_id,"request_id":request_id})));
            }
            if call.name == "todowrite" {
                self.execute_allowed_calls(
                    context,
                    std::mem::take(&mut allowed_calls),
                    token.clone(),
                )
                .await?;
                self.execute_todowrite(context, call)?;
                continue;
            }
            self.tool_state(context, call, "policy_check", None)?;
            let decision = evaluate(
                tool_risk(&call.name),
                self.non_interactive,
                self.store.session_full_control(&context.session_id)?,
            );
            self.store.append_audit(Some(&context.project_id), Some(&context.session_id), Some(&context.turn_id), "capability.decision", &json!({"tool_call_id":call.call_id,"operation":call.name,"decision":format!("{decision:?}")}))?;
            match decision {
                Decision::Deny => {
                    self.execute_allowed_calls(
                        context,
                        std::mem::take(&mut allowed_calls),
                        token.clone(),
                    )
                    .await?;
                    self.tool_state(context, call, "denied", Some("authorization_denied"))?;
                    return Err(AgentError::new(
                        "authorization_denied",
                        format!("Tool call denied: {}", call.name),
                    ));
                }
                Decision::ApprovalRequired => {
                    self.execute_allowed_calls(
                        context,
                        std::mem::take(&mut allowed_calls),
                        token.clone(),
                    )
                    .await?;
                    self.tool_state(
                        context,
                        call,
                        "awaiting_approval",
                        Some("risk_requires_approval"),
                    )?;
                    context.pending_call = Some(call.clone());
                    context.remaining_calls = calls[index + 1..].to_vec();
                    let snapshot = serde_json::to_value(&*context).map_err(|_| {
                        AgentError::new(
                            "agent_unavailable",
                            "turn continuation could not be stored",
                        )
                    })?;
                    let approval = self.store.create_approval(ApprovalInput {
                        project_id: Some(&context.project_id),
                        session_id: &context.session_id,
                        turn_id: &context.turn_id,
                        tool_call_id: &call.call_id,
                        operation: &call.name,
                        arguments: &call.arguments,
                        snapshot: &snapshot,
                    })?;
                    self.emit(&context.session_id,"approval.requested",json!({"turn_id":context.turn_id,"tool_call_id":call.call_id,"approval_id":approval.approval_id,"operation":call.name,"arguments":call.arguments}))?;
                    return Err(AgentError::new("approval_required",format!("Tool call requires approval: {}",call.name)).details(json!({"turn_id":context.turn_id,"tool_call_id":call.call_id,"approval_id":approval.approval_id})));
                }
                Decision::Allow => allowed_calls.push(call.clone()),
            }
        }
        self.execute_allowed_calls(context, allowed_calls, token)
            .await?;
        Ok(())
    }

    async fn execute_allowed_calls(
        &self,
        context: &mut Continuation,
        calls: Vec<ToolCall>,
        token: CancellationToken,
    ) -> Result<(), AgentError> {
        if calls.is_empty() {
            return Ok(());
        }
        let parallel_read_only = calls.len() > 1
            && calls
                .iter()
                .all(|call| tool_risk(&call.name) == Some(Risk::ReadOnly));
        if !parallel_read_only {
            for call in calls {
                self.execute_call(context, &call, token.clone()).await?;
            }
            return Ok(());
        }

        let mut futures = Vec::with_capacity(calls.len());
        for call in calls {
            self.tool_state(context, &call, "authorized", None)?;
            self.tool_state(context, &call, "executing", None)?;
            let (project_root, mut params) = match self.prepare_call(context, &call) {
                Ok(prepared) => prepared,
                Err(error) => {
                    if !self.record_recoverable_call_error(context, &call, &error)? {
                        return Err(error);
                    }
                    continue;
                }
            };
            params["idempotency_key"] = json!(format!("{}:{}", context.turn_id, call.call_id));
            let method = match method_name(&call.name) {
                Some(method) => method.to_string(),
                None => {
                    let error = AgentError::new("authorization_denied", "Unknown tool");
                    self.tool_state(context, &call, "failed", Some(&error.code))?;
                    return Err(error);
                }
            };
            let agent = self.clone();
            let token = token.clone();
            futures.push(async move {
                let result = agent
                    .operation_in_project(&project_root, &method, params, token)
                    .await;
                (call, result)
            });
        }

        for (call, result) in join_all(futures).await {
            let result = match result {
                Ok(result) => result,
                Err(error) => {
                    if !self.record_recoverable_call_error(context, &call, &error)? {
                        return Err(error);
                    }
                    continue;
                }
            };
            self.record_call_success(context, &call, result)?;
        }
        Ok(())
    }

    fn execute_todowrite(
        &self,
        context: &mut Continuation,
        call: &ToolCall,
    ) -> Result<(), AgentError> {
        let todos = parse_todos(&call.arguments)?;
        self.tool_state(context, call, "policy_check", None)?;
        self.tool_state(context, call, "authorized", None)?;
        self.tool_state(context, call, "executing", None)?;
        context.todos = todos.clone();
        self.emit(
            &context.session_id,
            "todo.updated",
            json!({"turn_id":context.turn_id,"call_id":context.active_call_id,"tool_call_id":call.call_id,"todos":todos}),
        )?;
        self.tool_state(context, call, "succeeded", None)?;
        let result = json!({"todos": context.todos});
        self.emit(
            &context.session_id,
            "tool.result",
            json!({"turn_id":context.turn_id,"call_id":context.active_call_id,"tool_call_id":call.call_id,"result":result}),
        )?;
        let mut tool = Message::text(
            "tool",
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".into()),
        );
        tool.tool_call_id = Some(call.call_id.clone());
        context.messages.push(tool.clone());
        self.emit(
            &context.session_id,
            "message.tool",
            json!({"turn_id":context.turn_id,"call_id":context.active_call_id,"tool_call_id":call.call_id,"message":tool}),
        )?;
        Ok(())
    }

    async fn execute_call(
        &self,
        context: &mut Continuation,
        call: &ToolCall,
        token: CancellationToken,
    ) -> Result<(), AgentError> {
        self.tool_state(context, call, "authorized", None)?;
        self.tool_state(context, call, "executing", None)?;
        let (project_root, mut params) = match self.prepare_call(context, call) {
            Ok(prepared) => prepared,
            Err(error) => {
                if !self.record_recoverable_call_error(context, call, &error)? {
                    return Err(error);
                }
                return Ok(());
            }
        };
        params["idempotency_key"] = json!(format!("{}:{}", context.turn_id, call.call_id));
        let method = match method_name(&call.name) {
            Some(method) => method,
            None => {
                let error = AgentError::new("authorization_denied", "Unknown tool");
                self.tool_state(context, call, "failed", Some(&error.code))?;
                return Err(error);
            }
        };
        let result = match self
            .operation_in_project(&project_root, method, params, token)
            .await
        {
            Ok(result) => result,
            Err(error) => {
                if !self.record_recoverable_call_error(context, call, &error)? {
                    return Err(error);
                }
                return Ok(());
            }
        };
        self.record_call_success(context, call, result)
    }

    fn record_recoverable_call_error(
        &self,
        context: &mut Continuation,
        call: &ToolCall,
        error: &AgentError,
    ) -> Result<bool, AgentError> {
        self.tool_state(context, call, "failed", Some(&error.code))?;
        if !matches!(
            error.code.as_str(),
            "invalid_arguments" | "malformed_tool_call"
        ) {
            return Ok(false);
        }
        let result = json!({
            "error": {
                "code": error.code,
                "message": error.message,
                "details": error.details,
            }
        });
        self.emit(
            &context.session_id,
            "tool.result",
            json!({
                "turn_id": context.turn_id,
                "call_id": context.active_call_id,
                "tool_call_id": call.call_id,
                "result": result,
            }),
        )?;
        self.store.append_audit(
            Some(&context.project_id),
            Some(&context.session_id),
            Some(&context.turn_id),
            "operation.result",
            &json!({
                "tool_call_id": call.call_id,
                "operation": call.name,
                "outcome": "failed",
                "error_code": error.code,
            }),
        )?;
        let mut tool = Message::text(
            "tool",
            serde_json::to_string(&result).unwrap_or_else(|_| "{\"error\":{}}".into()),
        );
        tool.tool_call_id = Some(call.call_id.clone());
        context.messages.push(tool.clone());
        self.emit(
            &context.session_id,
            "message.tool",
            json!({
                "turn_id": context.turn_id,
                "call_id": context.active_call_id,
                "tool_call_id": call.call_id,
                "message": tool,
            }),
        )?;
        Ok(true)
    }

    fn record_call_success(
        &self,
        context: &mut Continuation,
        call: &ToolCall,
        result: Value,
    ) -> Result<(), AgentError> {
        let dependency_id = call
            .arguments
            .get("path")
            .and_then(Value::as_str)
            .and_then(dependency_path)
            .map(|(dependency_id, _)| dependency_id);
        let mut normalized_result = normalize_result(&call.name, result.clone(), dependency_id);
        attach_nearby_instructions(context, call, &mut normalized_result);
        let process_failed = call.name == "bash"
            && normalized_result.get("success").and_then(Value::as_bool) == Some(false);
        self.tool_state(
            context,
            call,
            if process_failed {
                "failed"
            } else {
                "succeeded"
            },
            if process_failed {
                normalized_result.get("status").and_then(Value::as_str)
            } else {
                None
            },
        )?;
        self.emit(
            &context.session_id,
            "tool.result",
            json!({
                "turn_id": context.turn_id,
                "call_id": context.active_call_id,
                "tool_call_id": call.call_id,
                "result": normalized_result,
            }),
        )?;
        self.store.append_audit(
            Some(&context.project_id),
            Some(&context.session_id),
            Some(&context.turn_id),
            "operation.result",
            &json!({"tool_call_id":call.call_id,"operation":call.name,"outcome":if process_failed { "failed" } else { "succeeded" }, "status": normalized_result.get("status")}),
        )?;
        let checkpoint_ids = result
            .get("checkpoint_ids")
            .and_then(Value::as_array)
            .map(|values| values.iter().filter_map(Value::as_str).collect::<Vec<_>>())
            .unwrap_or_else(|| {
                result
                    .get("checkpoint_id")
                    .and_then(Value::as_str)
                    .map(|value| vec![value])
                    .unwrap_or_default()
            });
        if !checkpoint_ids.is_empty() {
            let manifest = self
                .store
                .ensure_manifest(&context.session_id, &context.turn_id)?;
            let existing = self.store.checkpoint_items(&manifest.manifest_id)?.len() as i64;
            for (index, id) in checkpoint_ids.iter().enumerate() {
                let path = if index == 0 {
                    result.get("path").or_else(|| result.get("from"))
                } else {
                    result.get("to")
                }
                .and_then(Value::as_str);
                self.emit(&context.session_id,"checkpoint.captured",json!({"turn_id":context.turn_id,"tool_call_id":call.call_id,"manifest_id":manifest.manifest_id,"checkpoint_id":id,"path":path,"ordinal":existing+index as i64}))?;
            }
        }
        let mut tool = Message::text(
            "tool",
            serde_json::to_string(&normalized_result).unwrap_or_else(|_| "{}".into()),
        );
        tool.tool_call_id = Some(call.call_id.clone());
        context.messages.push(tool.clone());
        self.emit(
            &context.session_id,
            "message.tool",
            json!({"turn_id":context.turn_id,"call_id":context.active_call_id,"tool_call_id":call.call_id,"message":tool}),
        )?;
        Ok(())
    }

    async fn operation_in_project(
        &self,
        project_root: &str,
        method: &str,
        params: Value,
        token: CancellationToken,
    ) -> Result<Value, AgentError> {
        let operations = self.operations.clone();
        let root = std::path::PathBuf::from(project_root);
        let method = method.to_string();
        let cancelled = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let cancelled_for_operation = cancelled.clone();
        let mut operation = Box::pin(tokio::task::spawn_blocking(move || {
            operations.execute_in_project_with_cancellation(
                &root,
                &method,
                params,
                Some(&cancelled_for_operation),
            )
        }));
        let join_result = tokio::select! {
            result = &mut operation => result,
            _ = token.cancelled() => {
                cancelled.store(true, std::sync::atomic::Ordering::Relaxed);
                (&mut operation).await
            }
        };
        join_result
            .map_err(|_| AgentError::new("agent_unavailable", "operation task failed"))?
            .map_err(|error| {
                let mut agent_error = AgentError::new(
                    error
                        .get("code")
                        .and_then(Value::as_str)
                        .unwrap_or("operation_failed"),
                    error
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("operation failed"),
                );
                if let Some(details) = error.get("details") {
                    agent_error = agent_error.details(details.clone());
                }
                agent_error
            })
    }

    fn prepare_call(
        &self,
        context: &Continuation,
        call: &ToolCall,
    ) -> Result<(String, Value), AgentError> {
        let mut arguments = call.arguments.clone();
        let Some(path) = arguments.get("path").and_then(Value::as_str) else {
            return Ok((
                context.project_root.clone(),
                translate_arguments(&call.name, &arguments)?,
            ));
        };
        if path.starts_with("dependency:") && dependency_path(path).is_none() {
            return Err(AgentError::new(
                "invalid_arguments",
                "dependency path must include a dependency ID",
            ));
        }
        let Some((dependency_id, relative_path)) = dependency_path(path) else {
            return Ok((
                context.project_root.clone(),
                translate_arguments(&call.name, &arguments)?,
            ));
        };
        if !dependency_tool_allowed(&call.name) {
            return Err(AgentError::new(
                "scope_denied",
                "dependencies are read-only and support only read, glob, and grep",
            ));
        }
        let dependency = self
            .store
            .project_dependency_by_id(&context.project_id, dependency_id)?
            .ok_or_else(|| AgentError::new("dependency_not_found", "dependency not found"))?;
        arguments["path"] = json!(relative_path);
        Ok((
            dependency.canonical_root,
            translate_arguments(&call.name, &arguments)?,
        ))
    }

    fn validate_dependency_call(
        &self,
        context: &Continuation,
        call: &ToolCall,
    ) -> Result<(), AgentError> {
        let Some(path) = call.arguments.get("path").and_then(Value::as_str) else {
            return Ok(());
        };
        if path.starts_with("dependency:") && dependency_path(path).is_none() {
            return Err(AgentError::new(
                "invalid_arguments",
                "dependency path must include a dependency ID",
            ));
        }
        let Some((dependency_id, _)) = dependency_path(path) else {
            return Ok(());
        };
        if !dependency_tool_allowed(&call.name) {
            return Err(AgentError::new(
                "scope_denied",
                "dependencies are read-only and support only read, glob, and grep",
            ));
        }
        if self
            .store
            .project_dependency_by_id(&context.project_id, dependency_id)?
            .is_none()
        {
            return Err(AgentError::new(
                "dependency_not_found",
                "dependency not found",
            ));
        }
        Ok(())
    }

    fn dependency_context_message(
        &self,
        project_id: &str,
    ) -> Result<Option<suncode_llm::Message>, AgentError> {
        let dependencies = self.store.project_dependencies(project_id)?;
        if dependencies.is_empty() {
            return Ok(None);
        }
        let roots = dependencies
            .iter()
            .map(|dependency| {
                format!(
                    "- {}: dependency:{}",
                    dependency.display_name, dependency.dependency_id
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        Ok(Some(suncode_llm::Message {
            role: "system".into(),
            content: vec![suncode_llm::ContentPart {
                kind: "text".into(),
                text: format!(
                    "Registered read-only source dependencies:\n{roots}\nUse dependency:<dependencyId>/<relativePath> with read, or dependency:<dependencyId> as the glob/grep path. Dependencies cannot be modified or used as a process working directory."
                ),
            }],
            tool_calls: Vec::new(),
            tool_call_id: None,
        }))
    }
    fn emit(&self, session_id: &str, event_type: &str, payload: Value) -> Result<(), AgentError> {
        let event = self
            .store
            .append_content(session_id, event_type, &payload)?;
        let _ = self.events.send(event);
        Ok(())
    }

    fn emit_live(&self, session_id: &str, event_type: &str, payload: Value) {
        let event = SessionEvent {
            session_id: session_id.to_string(),
            occurred_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            event_type: event_type.to_string(),
            payload,
        };
        let _ = self.events.send(event);
    }
    fn turn_state(
        &self,
        context: &Continuation,
        state: &str,
        reason: Option<&str>,
    ) -> Result<(), AgentError> {
        self.emit(&context.session_id,"turn.state",json!({"turn_id":context.turn_id,"state":state,"model_id":context.model,"submission_idempotency_key":context.submission_key,"reason":reason}))
    }
    fn tool_state(
        &self,
        context: &Continuation,
        call: &ToolCall,
        state: &str,
        reason: Option<&str>,
    ) -> Result<(), AgentError> {
        self.emit(&context.session_id,"tool.state",json!({"turn_id":context.turn_id,"call_id":context.active_call_id,"tool_call_id":call.call_id,"name":call.name,"state":state,"reason":reason}))
    }
    fn fail_context<T>(
        &self,
        context: &Continuation,
        code: &str,
        message: &str,
    ) -> Result<T, AgentError> {
        self.turn_state(
            context,
            if code == "cancelled" {
                "cancelled"
            } else {
                "failed"
            },
            Some(code),
        )?;
        Err(AgentError::new(code, message))
    }

    async fn session_lock(&self, session_id: &str) -> Arc<AsyncMutex<()>> {
        let mut locks = self.session_locks.lock().await;
        locks
            .entry(session_id.to_string())
            .or_insert_with(|| Arc::new(AsyncMutex::new(())))
            .clone()
    }

    pub async fn recover(&self) -> Result<(), AgentError> {
        for event in self.store.recover_startup()? {
            let _ = self.events.send(event);
        }
        for suspended in self.store.resuming_turns()? {
            let mut continuation: Continuation = serde_json::from_value(suspended.snapshot)
                .map_err(|_| {
                    AgentError::new("agent_unavailable", "approval continuation is invalid")
                })?;
            let token = CancellationToken::new();
            self.cancellations
                .lock()
                .map_err(|_| {
                    AgentError::new("agent_unavailable", "cancellation state unavailable")
                })?
                .insert(continuation.turn_id.clone(), token.clone());
            self.active_turns
                .lock()
                .map_err(|_| AgentError::new("agent_unavailable", "turn state unavailable"))?
                .insert(
                    continuation.session_id.clone(),
                    continuation.turn_id.clone(),
                );
            let agent = self.clone();
            let approval_id = suspended.approval_id;
            let is_question = continuation
                .pending_call
                .as_ref()
                .is_some_and(|call| call.name == "question");
            tokio::spawn(async move {
                let session_lock = agent.session_lock(&continuation.session_id).await;
                let _guard = session_lock.lock().await;
                let result = if is_question {
                    agent.continue_question(&mut continuation, token).await
                } else {
                    agent.continue_approved(&mut continuation, token).await
                };
                let suspended_again = result.as_ref().err().is_some_and(|error| {
                    matches!(
                        error.code.as_str(),
                        "approval_required" | "question_required"
                    )
                });
                let _ = agent.store.finish_suspended(
                    &approval_id,
                    if result.is_ok() || suspended_again {
                        "completed"
                    } else {
                        "failed"
                    },
                );
                agent
                    .active_turns
                    .lock()
                    .ok()
                    .map(|mut values| values.remove(&continuation.session_id));
            });
        }
        Ok(())
    }
}

fn method_name(name: &str) -> Option<&'static str> {
    match name {
        "read" => Some("tool/read"),
        "glob" => Some("tool/glob"),
        "grep" => Some("tool/grep"),
        "write" => Some("tool/write"),
        "edit" => Some("tool/edit"),
        "bash" => Some("tool/bash"),
        "webfetch" => Some("tool/webfetch"),
        _ => None,
    }
}

fn tool_signature(call: &ToolCall) -> String {
    format!(
        "{}:{}",
        call.name,
        serde_json::to_string(&call.arguments).unwrap_or_default()
    )
}

fn translate_arguments(name: &str, value: &Value) -> Result<Value, AgentError> {
    let mut result = value.clone();
    if name == "webfetch" {
        validate_webfetch_arguments(&result)?;
    }
    if name == "write" {
        let content = result
            .get("content")
            .and_then(Value::as_str)
            .ok_or_else(|| AgentError::new("invalid_arguments", "content is required"))?;
        result["content_base64"] = json!(STANDARD.encode(content));
        if let Some(object) = result.as_object_mut() {
            object.remove("content");
        }
    }
    if name == "edit" {
        let replacements = if let Some(edits) = result.get("edits").and_then(Value::as_array) {
            if edits.is_empty() {
                return Err(AgentError::new(
                    "invalid_arguments",
                    "edits must not be empty",
                ));
            }
            edits
                .iter()
                .map(|edit| {
                    let object = edit.as_object().ok_or_else(|| {
                        AgentError::new("invalid_arguments", "each edit must be an object")
                    })?;
                    let old = object
                        .get("oldText")
                        .and_then(Value::as_str)
                        .ok_or_else(|| {
                            AgentError::new("invalid_arguments", "oldText is required")
                        })?;
                    let new = object
                        .get("newText")
                        .and_then(Value::as_str)
                        .ok_or_else(|| {
                            AgentError::new("invalid_arguments", "newText is required")
                        })?;
                    Ok(json!({"old": old, "new": new, "replace_all": false}))
                })
                .collect::<Result<Vec<_>, AgentError>>()?
        } else {
            let old = result
                .get("oldString")
                .and_then(Value::as_str)
                .ok_or_else(|| AgentError::new("invalid_arguments", "oldString is required"))?;
            let new = result
                .get("newString")
                .and_then(Value::as_str)
                .ok_or_else(|| AgentError::new("invalid_arguments", "newString is required"))?;
            let replace_all = result
                .get("replaceAll")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            vec![json!({"old": old, "new": new, "replace_all": replace_all})]
        };
        result["replacements"] = json!(replacements);
        if let Some(object) = result.as_object_mut() {
            object.remove("oldString");
            object.remove("newString");
            object.remove("replaceAll");
            object.remove("edits");
        }
    }
    if name == "glob" {
        if let Some(path) = result.get("path").and_then(Value::as_str) {
            let pattern = result
                .get("pattern")
                .and_then(Value::as_str)
                .ok_or_else(|| AgentError::new("invalid_arguments", "pattern is required"))?;
            result["pattern"] = json!(scoped_glob(path, pattern));
        }
        if let Some(limit) = result.get("limit").cloned() {
            result["max_results"] = limit;
        }
        if let Some(object) = result.as_object_mut() {
            object.remove("limit");
            object.remove("path");
        }
    }
    if name == "grep" {
        let query = result
            .get("pattern")
            .or_else(|| result.get("query"))
            .and_then(Value::as_str)
            .ok_or_else(|| AgentError::new("invalid_arguments", "pattern is required"))?;
        result["query"] = json!(query);
        if let Some(include) = result.get("include").cloned() {
            result["pattern"] = include;
        } else {
            result["pattern"] = json!("**/*");
        }
        if let Some(path) = result.get("path").and_then(Value::as_str) {
            let pattern = result
                .get("pattern")
                .and_then(Value::as_str)
                .ok_or_else(|| AgentError::new("invalid_arguments", "pattern is required"))?;
            result["pattern"] = json!(scoped_glob(path, pattern));
        }
        if let Some(limit) = result.get("limit").cloned() {
            result["max_results"] = limit;
        }
        if let Some(object) = result.as_object_mut() {
            object.remove("include");
            object.remove("limit");
            object.remove("path");
        }
    }
    if name == "bash" {
        let command = result
            .get("command")
            .and_then(Value::as_str)
            .filter(|command| !command.trim().is_empty())
            .map(str::to_string)
            .ok_or_else(|| {
                AgentError::new(
                    "invalid_arguments",
                    "bash command must be a non-empty string",
                )
            })?;
        let (program, args) = shell_command(&command);
        result["program"] = json!(program);
        result["args"] = json!(args);
        if let Some(workdir) = result.get("workdir").or_else(|| result.get("cwd")).cloned() {
            result["cwd"] = workdir;
        }
        if let Some(timeout) = result.get("timeout").cloned() {
            result["timeout_ms"] = json!(timeout_millis(&timeout)?);
        }
        if let Some(object) = result.as_object_mut() {
            object.remove("command");
            object.remove("workdir");
            object.remove("timeout");
        }
    }
    Ok(result)
}

fn validate_before_policy(name: &str, value: &Value) -> Result<(), AgentError> {
    if name == "question" {
        return validate_question_arguments(value);
    }
    if name == "todowrite" {
        return validate_todowrite_arguments(value);
    }
    if name == "webfetch" {
        validate_webfetch_arguments(value)?;
    }
    Ok(())
}

fn validate_todowrite_arguments(value: &Value) -> Result<(), AgentError> {
    let todos = value
        .get("todos")
        .and_then(Value::as_array)
        .filter(|items| items.len() <= 100)
        .ok_or_else(|| {
            AgentError::new(
                "invalid_arguments",
                "todos must be an array of at most 100 items",
            )
        })?;
    let mut in_progress = 0;
    for todo in todos {
        let object = todo
            .as_object()
            .ok_or_else(|| AgentError::new("invalid_arguments", "each todo must be an object"))?;
        let content = object
            .get("content")
            .and_then(Value::as_str)
            .filter(|content| !content.trim().is_empty())
            .ok_or_else(|| AgentError::new("invalid_arguments", "todo content is required"))?;
        if content.chars().count() > 500 {
            return Err(AgentError::new(
                "invalid_arguments",
                "todo content must be at most 500 characters",
            ));
        }
        match object.get("status").and_then(Value::as_str) {
            Some("pending" | "completed" | "cancelled") => {}
            Some("in_progress") => in_progress += 1,
            _ => {
                return Err(AgentError::new(
                    "invalid_arguments",
                    "todo status must be pending, in_progress, completed, or cancelled",
                ))
            }
        }
        if !matches!(
            object.get("priority").and_then(Value::as_str),
            Some("high" | "medium" | "low")
        ) {
            return Err(AgentError::new(
                "invalid_arguments",
                "todo priority must be high, medium, or low",
            ));
        }
    }
    if in_progress > 1 {
        return Err(AgentError::new(
            "invalid_arguments",
            "only one todo may be in_progress",
        ));
    }
    Ok(())
}

fn parse_todos(value: &Value) -> Result<Vec<TodoEntry>, AgentError> {
    validate_todowrite_arguments(value)?;
    serde_json::from_value(
        value
            .get("todos")
            .cloned()
            .ok_or_else(|| AgentError::new("invalid_arguments", "todos is required"))?,
    )
    .map_err(|_| AgentError::new("invalid_arguments", "todos contain invalid values"))
}

fn validate_question_arguments(value: &Value) -> Result<(), AgentError> {
    let questions = value
        .get("questions")
        .and_then(Value::as_array)
        .filter(|items| !items.is_empty() && items.len() <= 8)
        .ok_or_else(|| {
            AgentError::new("invalid_arguments", "questions must contain 1 to 8 items")
        })?;
    for question in questions {
        let object = question.as_object().ok_or_else(|| {
            AgentError::new("invalid_arguments", "each question must be an object")
        })?;
        for field in ["question", "header"] {
            if object
                .get(field)
                .and_then(Value::as_str)
                .is_none_or(str::is_empty)
            {
                return Err(AgentError::new(
                    "invalid_arguments",
                    format!("{field} is required"),
                ));
            }
        }
        if object
            .get("header")
            .and_then(Value::as_str)
            .is_some_and(|value| value.chars().count() > 30)
        {
            return Err(AgentError::new(
                "invalid_arguments",
                "header must be at most 30 characters",
            ));
        }
        let options = object
            .get("options")
            .and_then(Value::as_array)
            .filter(|items| !items.is_empty() && items.len() <= 12)
            .ok_or_else(|| {
                AgentError::new("invalid_arguments", "options must contain 1 to 12 items")
            })?;
        let mut labels = std::collections::HashSet::new();
        for option in options {
            let option = option.as_object().ok_or_else(|| {
                AgentError::new("invalid_arguments", "each option must be an object")
            })?;
            let label = option
                .get("label")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| AgentError::new("invalid_arguments", "option label is required"))?;
            if !labels.insert(label) {
                return Err(AgentError::new(
                    "invalid_arguments",
                    "option labels must be unique",
                ));
            }
            if option
                .get("description")
                .and_then(Value::as_str)
                .is_none_or(str::is_empty)
            {
                return Err(AgentError::new(
                    "invalid_arguments",
                    "option description is required",
                ));
            }
        }
    }
    Ok(())
}

fn validate_question_answers(arguments: &Value, answers: &[Vec<String>]) -> Result<(), AgentError> {
    let questions = arguments
        .get("questions")
        .and_then(Value::as_array)
        .ok_or_else(|| AgentError::new("invalid_arguments", "questions are missing"))?;
    if answers.len() != questions.len() {
        return Err(AgentError::new(
            "invalid_arguments",
            "one answer list is required for each question",
        ));
    }
    for (question, values) in questions.iter().zip(answers) {
        let options = question
            .get("options")
            .and_then(Value::as_array)
            .ok_or_else(|| AgentError::new("invalid_arguments", "question options are missing"))?;
        let allowed = options
            .iter()
            .filter_map(|option| option.get("label").and_then(Value::as_str))
            .collect::<std::collections::HashSet<_>>();
        if question.get("multiple").and_then(Value::as_bool) != Some(true) && values.len() > 1 {
            return Err(AgentError::new(
                "invalid_arguments",
                "this question accepts only one answer",
            ));
        }
        let custom = question
            .get("custom")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        for value in values {
            if value.trim().is_empty() || (!custom && !allowed.contains(value.as_str())) {
                return Err(AgentError::new(
                    "invalid_arguments",
                    "an answer is not allowed for this question",
                ));
            }
        }
    }
    Ok(())
}

fn validate_webfetch_arguments(value: &Value) -> Result<(), AgentError> {
    let raw_url = value
        .get("url")
        .and_then(Value::as_str)
        .filter(|url| !url.trim().is_empty())
        .ok_or_else(|| AgentError::new("invalid_arguments", "url is required"))?;
    let url = url::Url::parse(raw_url).map_err(|_| {
        AgentError::new("invalid_arguments", "url must be a valid HTTP or HTTPS URL")
    })?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(AgentError::new(
            "invalid_arguments",
            "url must be a valid HTTP or HTTPS URL",
        ));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(AgentError::new(
            "invalid_arguments",
            "url must not contain embedded credentials",
        ));
    }
    if let Some(format) = value.get("format") {
        if !matches!(format.as_str(), Some("text" | "markdown" | "html")) {
            return Err(AgentError::new(
                "invalid_arguments",
                "format must be text, markdown, or html",
            ));
        }
    }
    if let Some(timeout) = value.get("timeout") {
        let seconds = timeout
            .as_f64()
            .filter(|value| value.is_finite())
            .ok_or_else(|| {
                AgentError::new("invalid_arguments", "timeout must be a number of seconds")
            })?;
        if seconds <= 0.0 || seconds > 120.0 {
            return Err(AgentError::new(
                "invalid_arguments",
                "timeout must be greater than zero and no more than 120 seconds",
            ));
        }
    }
    Ok(())
}

fn timeout_millis(value: &Value) -> Result<u64, AgentError> {
    let milliseconds = value.as_u64().ok_or_else(|| {
        AgentError::new(
            "invalid_arguments",
            "timeout must be a positive integer number of milliseconds",
        )
    })?;
    if milliseconds == 0 || milliseconds > 600_000 {
        return Err(AgentError::new(
            "invalid_arguments",
            "timeout must be greater than zero and no more than 600000 milliseconds",
        ));
    }
    Ok(milliseconds)
}

#[cfg(target_os = "windows")]
fn shell_command(script: &str) -> (&'static str, Vec<String>) {
    (
        "powershell.exe",
        vec![
            "-NoLogo".into(),
            "-NoProfile".into(),
            "-NonInteractive".into(),
            "-Command".into(),
            script.into(),
        ],
    )
}

#[cfg(not(target_os = "windows"))]
fn shell_command(script: &str) -> (&'static str, Vec<String>) {
    ("/bin/sh", vec!["-lc".into(), script.into()])
}

#[derive(Debug, Serialize)]
struct LoadedInstruction {
    path: String,
    content: String,
}

fn project_instruction_message(project_root: &str) -> Option<suncode_llm::Message> {
    let root = Path::new(project_root).canonicalize().ok()?;
    let content = read_instruction_file(&root, &root.join("AGENTS.md"))?;
    Some(suncode_llm::Message::text(
        "system",
        format!(
            "Repository instructions from AGENTS.md (scope: the entire opened project):\n{content}\nMore specific AGENTS.md files reported by the read tool override conflicting broader instructions for files in their directory tree."
        ),
    ))
}

fn attach_nearby_instructions(context: &mut Continuation, call: &ToolCall, result: &mut Value) {
    if call.name != "read" {
        return;
    }
    let Some(path) = call.arguments.get("path").and_then(Value::as_str) else {
        return;
    };
    if dependency_path(path).is_some() || !is_safe_relative_path(path) {
        return;
    }
    let instructions = nearby_instruction_files(
        &context.project_root,
        path,
        &context.loaded_instruction_paths,
    );
    if instructions.is_empty() {
        return;
    }
    let Some(object) = result.as_object_mut() else {
        return;
    };
    context.loaded_instruction_paths.extend(
        instructions
            .iter()
            .map(|instruction| instruction.path.clone()),
    );
    object.insert("repository_instructions".into(), json!(instructions));
}

fn nearby_instruction_files(
    project_root: &str,
    read_path: &str,
    loaded_paths: &[String],
) -> Vec<LoadedInstruction> {
    let Ok(root) = Path::new(project_root).canonicalize() else {
        return Vec::new();
    };
    let Ok(target) = root.join(read_path).canonicalize() else {
        return Vec::new();
    };
    if !target.starts_with(&root)
        || target.file_name().and_then(|name| name.to_str()) == Some("AGENTS.md")
    {
        return Vec::new();
    }
    let Some(mut current) = target.parent().map(Path::to_path_buf) else {
        return Vec::new();
    };
    let mut instructions = Vec::new();
    let mut total_bytes = 0usize;
    while current.starts_with(&root)
        && current != root
        && instructions.len() < MAX_NEARBY_INSTRUCTION_FILES
    {
        let candidate = current.join("AGENTS.md");
        let relative = candidate
            .strip_prefix(&root)
            .ok()
            .map(slash_path)
            .unwrap_or_default();
        if !relative.is_empty() && !loaded_paths.iter().any(|path| path == &relative) {
            if let Some(content) = read_instruction_file(&root, &candidate) {
                let bytes = content.len();
                if total_bytes + bytes > MAX_NEARBY_INSTRUCTION_BYTES {
                    break;
                }
                total_bytes += bytes;
                instructions.push(LoadedInstruction {
                    path: relative,
                    content: format!(
                        "Instructions from {}/AGENTS.md (scope: this directory tree):\n{}",
                        slash_path(current.strip_prefix(&root).unwrap_or(Path::new("."))),
                        content
                    ),
                });
            }
        }
        let Some(parent) = current.parent() else {
            break;
        };
        current = parent.to_path_buf();
    }
    instructions
}

fn read_instruction_file(root: &Path, candidate: &Path) -> Option<String> {
    let canonical = candidate.canonicalize().ok()?;
    if !canonical.starts_with(root) {
        return None;
    }
    let metadata = fs::metadata(&canonical).ok()?;
    if !metadata.is_file() || metadata.len() > MAX_INSTRUCTION_FILE_BYTES {
        return None;
    }
    let content = fs::read_to_string(canonical).ok()?;
    (!content.trim().is_empty()).then_some(content)
}

fn is_safe_relative_path(value: &str) -> bool {
    let path = PathBuf::from(value);
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

fn slash_path(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn host_environment_message(session_started_at: &str) -> suncode_llm::Message {
    let shell = if cfg!(target_os = "windows") {
        "Windows PowerShell"
    } else {
        "POSIX sh"
    };
    let path_style = if cfg!(target_os = "windows") {
        "Windows"
    } else {
        "POSIX"
    };
    let session_started_at = if session_started_at.is_empty() {
        "unavailable"
    } else {
        session_started_at
    };
    suncode_llm::Message {
        role: "system".into(),
        content: vec![suncode_llm::ContentPart {
            kind: "text".into(),
            text: format!(
                "SunCode host environment: OS={}, architecture={}, shell tool dialect={}, path style={}, session started at={}. Use the bash tool for terminal commands and write commands in the stated shell dialect. For file discovery and content search, use glob, grep, and read instead of running find, grep, or rg through bash.",
                std::env::consts::OS,
                std::env::consts::ARCH,
                shell,
                path_style,
                session_started_at
            ),
        }],
        tool_calls: Vec::new(),
        tool_call_id: None,
    }
}

fn scoped_glob(path: &str, pattern: &str) -> String {
    let base = path.trim_matches('/');
    if base.is_empty() || base == "." {
        return pattern.to_string();
    }
    format!("{base}/{}", pattern.trim_start_matches('/'))
}

fn dependency_path(path: &str) -> Option<(&str, &str)> {
    let value = path.strip_prefix("dependency:")?;
    let (dependency_id, relative_path) = value.split_once('/').unwrap_or((value, "."));
    if dependency_id.is_empty() {
        return None;
    }
    Some((dependency_id, relative_path))
}

fn dependency_tool_allowed(name: &str) -> bool {
    matches!(name, "read" | "glob" | "grep")
}

fn to_llm_message(message: &Message) -> suncode_llm::Message {
    suncode_llm::Message {
        role: message.role.clone(),
        content: message
            .content
            .iter()
            .map(|part| suncode_llm::ContentPart {
                kind: part.kind.clone(),
                text: part.text.clone(),
            })
            .collect(),
        tool_calls: message
            .tool_calls
            .iter()
            .map(|call| suncode_llm::ToolCall {
                call_id: call.call_id.clone(),
                name: call.name.clone(),
                arguments: call.arguments.clone(),
            })
            .collect(),
        tool_call_id: message.tool_call_id.clone(),
    }
}

fn normalize_result(name: &str, mut value: Value, dependency_id: Option<&str>) -> Value {
    if name == "read" {
        if let Some(encoded) = value
            .get("data_base64")
            .and_then(Value::as_str)
            .map(str::to_string)
        {
            if let Ok(bytes) = STANDARD.decode(&encoded) {
                if let Ok(text) = String::from_utf8(bytes) {
                    value["content"] = json!(text);
                    let complete = value.get("truncated").and_then(Value::as_bool) != Some(true)
                        && value.get("offset").and_then(Value::as_u64).unwrap_or(1) == 1
                        && value.get("limit").is_none();
                    if complete && value.get("precondition_base64").is_none() {
                        value["precondition_base64"] = json!(encoded);
                    }
                }
            }
        }
        if let Some(object) = value.as_object_mut() {
            object.remove("data_base64");
        }
    }
    if name == "bash" {
        for (encoded_key, text_key) in [("stdout_base64", "stdout"), ("stderr_base64", "stderr")] {
            let mut decoded_text = false;
            if let Some(encoded) = value
                .get(encoded_key)
                .and_then(Value::as_str)
                .map(str::to_string)
            {
                if let Ok(bytes) = STANDARD.decode(&encoded) {
                    if let Ok(text) = String::from_utf8(bytes) {
                        value[text_key] = json!(text);
                        decoded_text = true;
                    }
                }
            }
            if decoded_text {
                if let Some(object) = value.as_object_mut() {
                    object.remove(encoded_key);
                }
            } else if value.get(encoded_key).is_some() {
                value["binary_output"] = json!(true);
            }
        }
    }
    if let Some(dependency_id) = dependency_id {
        if name == "read" {
            prefix_result_path(&mut value, "path", dependency_id);
        }
        if name == "glob" {
            if let Some(paths) = value.get_mut("paths").and_then(Value::as_array_mut) {
                for path in paths {
                    if let Some(relative) = path.as_str() {
                        *path = json!(dependency_alias(dependency_id, relative));
                    }
                }
            }
        }
        if name == "grep" {
            if let Some(matches) = value.get_mut("matches").and_then(Value::as_array_mut) {
                for matched in matches {
                    prefix_result_path(matched, "path", dependency_id);
                }
            }
        }
    }
    value
}

fn prefix_result_path(value: &mut Value, key: &str, dependency_id: &str) {
    let Some(relative) = value.get(key).and_then(Value::as_str) else {
        return;
    };
    value[key] = json!(dependency_alias(dependency_id, relative));
}

fn dependency_alias(dependency_id: &str, relative: &str) -> String {
    let relative = relative.trim_start_matches('/');
    if relative.is_empty() || relative == "." {
        format!("dependency:{dependency_id}")
    } else {
        format!("dependency:{dependency_id}/{relative}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::CredentialStore;
    use axum::{http::header, response::IntoResponse, routing::post, Json, Router};
    use suncode_llm::{
        ModelCapabilities, ModelDescriptor, ModelLimits, ModelProviderRegistry,
        OpenAiCompatibleProvider,
    };

    #[test]
    fn bash_translation_uses_opencode_command_and_millisecond_timeout() {
        let translated = translate_arguments(
            "bash",
            &json!({"command":"echo hello","timeout":120_000,"workdir":"src"}),
        )
        .unwrap();
        assert_eq!(translated["timeout_ms"], 120_000);
        assert_eq!(translated["cwd"], "src");
        assert!(translated.get("command").is_none());
        assert!(translate_arguments("bash", &json!({"command":"echo hello","timeout":0})).is_err());
        assert!(
            translate_arguments("bash", &json!({"command":"echo hello","timeout":600_001}))
                .is_err()
        );
    }

    #[test]
    fn edit_translation_accepts_multiple_disjoint_edits() {
        let translated = translate_arguments(
            "edit",
            &json!({
                "path":"file.txt",
                "expected_base64":"YmVmb3Jl",
                "edits":[
                    {"oldText":"one","newText":"two"},
                    {"oldText":"three","newText":"four"}
                ]
            }),
        )
        .unwrap();
        assert_eq!(translated["replacements"].as_array().unwrap().len(), 2);
        assert!(translated.get("edits").is_none());
    }

    #[test]
    fn webfetch_arguments_are_validated_before_policy() {
        assert!(validate_before_policy(
            "webfetch",
            &json!({"url":"https://example.com","format":"markdown","timeout":30})
        )
        .is_ok());
        assert!(validate_before_policy("webfetch", &json!({"url":"file:///tmp/example"})).is_err());
        assert!(validate_before_policy(
            "webfetch",
            &json!({"url":"https://user:secret@example.com"})
        )
        .is_err());
        assert!(validate_before_policy(
            "webfetch",
            &json!({"url":"https://example.com","format":"pdf"})
        )
        .is_err());
        assert!(validate_before_policy(
            "webfetch",
            &json!({"url":"https://example.com","timeout":121})
        )
        .is_err());
    }

    #[test]
    fn question_arguments_and_answers_are_validated() {
        let arguments = json!({"questions":[{"question":"Choose","header":"Mode","options":[{"label":"Fast","description":"Quick"}],"custom":false}]});
        assert!(validate_before_policy("question", &arguments).is_ok());
        assert!(validate_question_answers(&arguments, &[vec!["Fast".into()]]).is_ok());
        assert!(validate_question_answers(&arguments, &[vec!["Other".into()]]).is_err());
        assert!(validate_question_answers(&arguments, &[]).is_err());
    }

    #[test]
    fn todo_write_arguments_require_one_active_task_at_most() {
        let valid = json!({"todos":[{"content":"Implement tool","status":"in_progress","priority":"high"},{"content":"Run tests","status":"pending","priority":"medium"}]});
        assert!(validate_before_policy("todowrite", &valid).is_ok());
        assert!(validate_before_policy("todowrite", &json!({"todos":[]})).is_ok());
        assert!(validate_before_policy("todowrite", &json!({"todos":[{"content":"one","status":"in_progress","priority":"low"},{"content":"two","status":"in_progress","priority":"low"}]})).is_err());
        assert!(validate_before_policy(
            "todowrite",
            &json!({"todos":[{"content":"one","status":"blocked","priority":"low"}]})
        )
        .is_err());
    }

    #[test]
    fn dependency_paths_are_parsed_without_exposing_absolute_roots() {
        assert_eq!(
            dependency_path("dependency:dependency-1/src/lib.rs"),
            Some(("dependency-1", "src/lib.rs"))
        );
        assert_eq!(
            dependency_path("dependency:dependency-1"),
            Some(("dependency-1", "."))
        );
        assert_eq!(dependency_path("src/lib.rs"), None);
        assert_eq!(dependency_path("dependency:"), None);
        assert!(dependency_tool_allowed("read"));
        assert!(!dependency_tool_allowed("write"));
    }

    #[test]
    fn dependency_results_preserve_the_stable_alias() {
        let read = normalize_result(
            "read",
            json!({"path":"src/lib.rs","data_base64":STANDARD.encode("hello")}),
            Some("dependency-1"),
        );
        assert_eq!(read["path"], "dependency:dependency-1/src/lib.rs");

        let glob = normalize_result(
            "glob",
            json!({"paths":["src/lib.rs","README.md"]}),
            Some("dependency-1"),
        );
        assert_eq!(
            glob["paths"],
            json!([
                "dependency:dependency-1/src/lib.rs",
                "dependency:dependency-1/README.md"
            ])
        );

        let grep = normalize_result(
            "grep",
            json!({"matches":[{"path":"src/lib.rs","line":1}]}),
            Some("dependency-1"),
        );
        assert_eq!(
            grep["matches"][0]["path"],
            "dependency:dependency-1/src/lib.rs"
        );
    }

    #[test]
    fn legacy_bash_translation_uses_the_platform_shell() {
        let translated = translate_arguments("bash", &json!({"command":"echo hello"})).unwrap();
        #[cfg(target_os = "windows")]
        {
            assert_eq!(translated["program"], "powershell.exe");
            assert_eq!(
                translated["args"],
                json!([
                    "-NoLogo",
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    "echo hello"
                ])
            );
        }
        #[cfg(not(target_os = "windows"))]
        {
            assert_eq!(translated["program"], "/bin/sh");
            assert_eq!(translated["args"], json!(["-lc", "echo hello"]));
        }
        assert!(translated.get("command").is_none());
    }

    #[test]
    fn host_context_identifies_platform_and_stable_session_time() {
        let session_started_at = "2026-08-23T01:02:03.000Z";
        let message = host_environment_message(session_started_at);
        let text = message.content[0].text.as_str();
        assert!(text.contains(std::env::consts::OS));
        assert!(text.contains(std::env::consts::ARCH));
        assert!(text.contains(&format!("session started at={session_started_at}")));
        assert!(text.contains(
            "use glob, grep, and read instead of running find, grep, or rg through bash"
        ));
    }

    #[test]
    fn project_agents_file_is_loaded_as_repository_instructions() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("AGENTS.md"),
            "# Project rules\nRun focused tests.",
        )
        .unwrap();

        let message = project_instruction_message(directory.path().to_str().unwrap()).unwrap();

        assert_eq!(message.role, "system");
        let text = message.text_content();
        assert!(text.contains("Repository instructions from AGENTS.md"));
        assert!(text.contains("Run focused tests."));
        assert!(!text.contains(directory.path().to_str().unwrap()));
    }

    #[test]
    fn nearby_agents_files_are_loaded_nearest_first_and_deduplicated() {
        let directory = tempfile::tempdir().unwrap();
        fs::create_dir_all(directory.path().join("src/nested")).unwrap();
        fs::write(directory.path().join("AGENTS.md"), "root").unwrap();
        fs::write(directory.path().join("src/AGENTS.md"), "src").unwrap();
        fs::write(directory.path().join("src/nested/AGENTS.md"), "nested").unwrap();
        fs::write(directory.path().join("src/nested/file.rs"), "fn main() {}").unwrap();

        let instructions = nearby_instruction_files(
            directory.path().to_str().unwrap(),
            "src/nested/file.rs",
            &[],
        );

        assert_eq!(instructions.len(), 2);
        assert_eq!(instructions[0].path, "src/nested/AGENTS.md");
        assert!(instructions[0].content.contains("nested"));
        assert_eq!(instructions[1].path, "src/AGENTS.md");
        assert!(instructions[1].content.contains("src"));
        assert!(nearby_instruction_files(
            directory.path().to_str().unwrap(),
            "src/nested/file.rs",
            &["src/nested/AGENTS.md".into(), "src/AGENTS.md".into()],
        )
        .is_empty());
        assert!(nearby_instruction_files(
            directory.path().to_str().unwrap(),
            "src/nested/AGENTS.md",
            &[],
        )
        .is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn instruction_symlinks_cannot_escape_the_project() {
        use std::os::unix::fs::symlink;

        let project = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        fs::create_dir_all(project.path().join("src")).unwrap();
        fs::write(project.path().join("src/file.rs"), "fn main() {}").unwrap();
        fs::write(outside.path().join("AGENTS.md"), "outside rules").unwrap();
        symlink(
            outside.path().join("AGENTS.md"),
            project.path().join("src/AGENTS.md"),
        )
        .unwrap();

        assert!(
            nearby_instruction_files(project.path().to_str().unwrap(), "src/file.rs", &[],)
                .is_empty()
        );
    }

    async fn mock_deepseek(Json(body): Json<Value>) -> impl IntoResponse {
        let messages = body
            .get("messages")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let last_role = messages
            .last()
            .and_then(|message| message.get("role"))
            .and_then(Value::as_str);
        let has_tool_error = messages.iter().any(|message| {
            message.get("role").and_then(Value::as_str) == Some("tool")
                && message
                    .get("content")
                    .and_then(Value::as_str)
                    .map(|content| content.contains("invalid_arguments"))
                    .unwrap_or(false)
        });
        let user_text = messages
            .iter()
            .rev()
            .find(|message| message.get("role").and_then(Value::as_str) == Some("user"))
            .and_then(|message| message.get("content"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        let dependency_alias = messages
            .iter()
            .filter(|message| message.get("role").and_then(Value::as_str) == Some("system"))
            .filter_map(|message| message.get("content").and_then(Value::as_str))
            .find_map(|content| {
                content
                    .split_whitespace()
                    .find(|value| value.starts_with("dependency:") && !value.contains('<'))
                    .map(|value| {
                        value.trim_end_matches(|character: char| character.is_ascii_punctuation())
                    })
            });
        if user_text.contains("slow") {
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
        let data = if user_text.contains("invalid arguments") && !has_tool_error {
            vec![
                json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"invalid-read-call","function":{"name":"read","arguments":"{\"path\":123}"}}]},"finish_reason":"tool_calls"}]}),
            ]
        } else if last_role == Some("tool") {
            vec![
                json!({"choices":[{"delta":{"content":"done"},"finish_reason":"stop"}],"usage":{"prompt_tokens":8,"completion_tokens":1,"total_tokens":9,"prompt_cache_hit_tokens":6,"prompt_cache_miss_tokens":2,"completion_tokens_details":{"reasoning_tokens":1}}}),
            ]
        } else if user_text.contains("slow") || user_text.contains("follow up") {
            vec![
                json!({"choices":[{"delta":{"content":"queued done"},"finish_reason":"stop"}],"usage":{"prompt_tokens":4,"completion_tokens":2,"total_tokens":6}}),
            ]
        } else if user_text.contains("dependency read") {
            let path = format!("{}/lib.rs", dependency_alias.unwrap());
            vec![
                json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"dependency-read","function":{"name":"read","arguments":serde_json::to_string(&json!({"path":path})).unwrap()}}]},"finish_reason":"tool_calls"}]}),
            ]
        } else if user_text.contains("dependency write") {
            let path = format!("{}/lib.rs", dependency_alias.unwrap());
            vec![
                json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"dependency-write","function":{"name":"write","arguments":serde_json::to_string(&json!({"path":path,"content":"changed"})).unwrap()}}]},"finish_reason":"tool_calls"}]}),
            ]
        } else if user_text.contains("read nested") {
            vec![
                json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"nested-read","function":{"name":"read","arguments":"{\"path\":\"src/nested/file.rs\"}"}}]},"finish_reason":"tool_calls"}]}),
            ]
        } else if user_text.contains("read two") {
            vec![json!({"choices":[{"delta":{"tool_calls":[
                    {"index":0,"id":"read-call-1","function":{"name":"read","arguments":"{\"path\":\"README.md\"}"}},
                    {"index":1,"id":"read-call-2","function":{"name":"read","arguments":"{\"path\":\"README.md\"}"}}
                ]},"finish_reason":"tool_calls"}]})]
        } else if user_text.contains("write again") {
            vec![
                json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"write-again-call","function":{"name":"write","arguments":"{\"path\":\"README.md\",\"content\":\"updated again\",\"expected_base64\":\"dXBkYXRlZA==\"}"}}]},"finish_reason":"tool_calls"}]}),
            ]
        } else if user_text.contains("write") {
            vec![
                json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"write-call","function":{"name":"write","arguments":"{\"path\":\"README.md\",\"content\":\"updated\",\"expected_base64\":\"aGVsbG8=\"}"}}]},"finish_reason":"tool_calls"}]}),
            ]
        } else {
            vec![
                json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"read-call","function":{"name":"read","arguments":"{\"path\":\"README.md\"}"}}]},"finish_reason":"tool_calls"}]}),
            ]
        };
        let body = data
            .into_iter()
            .map(|value| format!("data: {value}\n\n"))
            .collect::<String>()
            + "data: [DONE]\n\n";
        ([(header::CONTENT_TYPE, "text/event-stream")], body).into_response()
    }

    async fn fixture() -> (
        Agent,
        Store,
        std::path::PathBuf,
        tokio::task::JoinHandle<()>,
        String,
    ) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(
                listener,
                Router::new().route("/chat/completions", post(mock_deepseek)),
            )
            .await
            .unwrap();
        });
        let directory = tempfile::tempdir().unwrap().keep();
        std::fs::write(directory.join("README.md"), "hello").unwrap();
        let store = Store::open_memory().unwrap();
        let root = directory.canonicalize().unwrap();
        let project = store.project(root.to_str().unwrap(), "Fixture").unwrap();
        let dependency_root = directory.join("dependency-source");
        std::fs::create_dir_all(&dependency_root).unwrap();
        std::fs::write(dependency_root.join("lib.rs"), "pub fn shared() {}\n").unwrap();
        store
            .add_project_dependency(
                &project.project_id,
                dependency_root.to_str().unwrap(),
                "Shared source",
            )
            .unwrap();
        let session = store
            .create_session(&project.project_id, None, Some("deepseek-v4-flash"))
            .unwrap();
        let operations =
            Arc::new(suncode_tool::Operations::new(directory.join(".operations")).unwrap());
        let (events, _) = broadcast::channel(64);
        let credentials = Arc::new(CredentialStore::memory(
            Some("test-key"),
            None,
            None,
            None,
            None,
            None,
        ));
        let provider = Arc::new(OpenAiCompatibleProvider::new(
            "deepseek",
            "DeepSeek",
            format!("http://{address}"),
            credentials,
        ));
        let mut registry = ModelProviderRegistry::new();
        let models = ["deepseek-v4-flash", "deepseek-v4-pro"]
            .into_iter()
            .enumerate()
            .map(|(_index, model_id)| ModelDescriptor {
                provider: "deepseek".into(),
                provider_label: "DeepSeek".into(),
                id: model_id.into(),
                wire_model: model_id.into(),
                api_base: format!("http://{address}"),
                capabilities: ModelCapabilities {
                    streaming: true,
                    tool_use: true,
                    vision: false,
                    structured_output: false,
                    cancellation: true,
                },
                limits: ModelLimits {
                    max_input_tokens: Some(64_000),
                    auto_compact_tokens: Some(47_616),
                    max_output_tokens: None,
                },
                availability: "configured".into(),
            })
            .collect();
        registry.register("deepseek", provider, models).unwrap();
        (
            Agent::new(store.clone(), Arc::new(registry), operations, events, false),
            store,
            root,
            server,
            session.session_id,
        )
    }

    #[tokio::test]
    async fn read_tool_round_trip_completes() {
        let (agent, store, root, server, session_id) = fixture().await;
        fs::write(root.join("AGENTS.md"), "Always run focused tests.").unwrap();
        let response = agent
            .submit(&session_id, "read-1", "read the file", None)
            .await
            .unwrap();
        assert!(matches!(
            response,
            TurnResponse::Completed { tool_calls: 1, .. }
        ));
        let messages = store.messages(&session_id).unwrap();
        assert!(messages.iter().all(|message| message.role != "tool"));
        assert_eq!(messages.last().unwrap().role, "assistant");
        let context = store.context_messages(&session_id).unwrap();
        assert!(context.iter().any(|message| message.role == "tool"));
        let exchanges = store.provider_exchanges(&session_id).unwrap();
        assert_eq!(exchanges.len(), 2);
        assert_eq!(
            exchanges[0].input_messages[0],
            exchanges[1].input_messages[0]
        );
        assert!(exchanges.iter().all(|exchange| exchange
            .input_messages
            .as_array()
            .unwrap()
            .iter()
            .any(|message| message["role"] == "system"
                && message["content"][0]["text"]
                    .as_str()
                    .is_some_and(|text| text.contains("Always run focused tests.")))));
        let usage = exchanges[0].usage.as_ref().unwrap();
        assert_eq!(usage["cache_read_tokens"], 6);
        assert_eq!(usage["cache_miss_tokens"], 2);
        assert_eq!(usage["cache_write_tokens"], serde_json::Value::Null);
        assert_eq!(usage["reasoning_tokens"], 1);
        server.abort();
    }

    #[tokio::test]
    async fn read_tool_attaches_nearby_agents_instructions_to_its_result() {
        let (agent, store, root, server, session_id) = fixture().await;
        fs::create_dir_all(root.join("src/nested")).unwrap();
        fs::write(
            root.join("src/AGENTS.md"),
            "Use the src module conventions.",
        )
        .unwrap();
        fs::write(root.join("src/nested/file.rs"), "pub fn nested() {}\n").unwrap();

        agent
            .submit(&session_id, "nested-read-1", "read nested", None)
            .await
            .unwrap();

        let turns = store.session_conversation_turns(&session_id).unwrap();
        let result = turns[0].tool_uses[0].result.as_ref().unwrap();
        assert_eq!(
            result["repository_instructions"][0]["path"],
            "src/AGENTS.md"
        );
        assert!(result["repository_instructions"][0]["content"]
            .as_str()
            .unwrap()
            .contains("Use the src module conventions."));
        server.abort();
    }

    #[tokio::test]
    async fn invalid_tool_arguments_are_returned_to_model_for_recovery() {
        let (agent, store, _root, server, session_id) = fixture().await;
        let response = agent
            .submit(
                &session_id,
                "invalid-arguments-1",
                "invalid arguments",
                None,
            )
            .await
            .unwrap();
        assert!(matches!(
            response,
            TurnResponse::Completed {
                iterations: 2,
                tool_calls: 1,
                ..
            }
        ));
        let context = store.context_messages(&session_id).unwrap();
        let tool = context
            .iter()
            .find(|message| message.role == "tool")
            .expect("invalid tool result should be retained in context");
        assert_eq!(tool.tool_call_id.as_deref(), Some("invalid-read-call"));
        assert!(tool.text_content().contains("invalid_arguments"));
        assert!(tool.text_content().contains("path is required"));
        server.abort();
    }

    #[tokio::test]
    async fn dependency_read_is_routed_and_write_is_rejected_before_approval() {
        let (agent, store, root, server, session_id) = fixture().await;
        let response = agent
            .submit(&session_id, "dependency-read-1", "dependency read", None)
            .await
            .unwrap();
        assert!(matches!(
            response,
            TurnResponse::Completed { tool_calls: 1, .. }
        ));
        let result = store
            .context_messages(&session_id)
            .unwrap()
            .into_iter()
            .find(|message| message.role == "tool")
            .unwrap()
            .text_content();
        assert!(result.contains("dependency:"));
        assert!(!result.contains(root.to_str().unwrap()));

        let error = agent
            .submit(&session_id, "dependency-write-1", "dependency write", None)
            .await
            .unwrap_err();
        assert_eq!(error.code, "scope_denied");
        assert_eq!(
            std::fs::read_to_string(root.join("dependency-source/lib.rs")).unwrap(),
            "pub fn shared() {}\n"
        );
        server.abort();
    }

    #[tokio::test]
    async fn queued_submit_is_injected_before_completion() {
        let (agent, store, _root, server, session_id) = fixture().await;
        let running = {
            let agent = agent.clone();
            let session_id = session_id.clone();
            tokio::spawn(async move {
                agent
                    .submit(&session_id, "slow-1", "slow initial request", None)
                    .await
            })
        };
        tokio::time::sleep(Duration::from_millis(30)).await;

        let queued = agent
            .submit(&session_id, "queued-1", "follow up while running", None)
            .await
            .unwrap();
        assert!(matches!(queued, TurnResponse::Queued { position: 1, .. }));
        let response = running.await.unwrap().unwrap();
        assert!(matches!(
            response,
            TurnResponse::Completed { iterations: 2, .. }
        ));
        assert!(store
            .messages(&session_id)
            .unwrap()
            .iter()
            .filter(|message| message.role == "user")
            .any(|message| message.text_content() == "follow up while running"));
        server.abort();
    }

    #[tokio::test]
    async fn read_only_tool_batch_is_preflighted_before_execution() {
        let (agent, store, _root, server, session_id) = fixture().await;
        let response = agent
            .submit(&session_id, "read-two-1", "read two files", None)
            .await
            .unwrap();
        assert!(matches!(
            response,
            TurnResponse::Completed { tool_calls: 2, .. }
        ));
        assert!(store
            .messages(&session_id)
            .unwrap()
            .iter()
            .all(|message| message.role != "tool"));
        assert_eq!(
            store
                .context_messages(&session_id)
                .unwrap()
                .iter()
                .filter(|message| message.role == "tool")
                .count(),
            2
        );
        server.abort();
    }

    #[tokio::test]
    async fn over_budget_batch_is_rejected_before_any_call_executes() {
        let (agent, store, _root, server, session_id) = fixture().await;
        let project_id = store
            .session_by_id(&session_id)
            .unwrap()
            .unwrap()
            .project_id
            .unwrap();
        store
            .set_setting("project", &project_id, "tool_call_limit", &json!(1))
            .unwrap();

        let error = agent
            .submit(&session_id, "over-budget-1", "read two files", None)
            .await
            .unwrap_err();
        assert_eq!(error.code, "tool_budget_exceeded");
        assert_eq!(error.details["limit"], 1);
        assert_eq!(error.details["rejected_batch_size"], 2);

        let turns = store.session_conversation_turns(&session_id).unwrap();
        let turn = turns.last().unwrap();
        assert_eq!(turn.state, "failed");
        assert_eq!(turn.tool_uses.len(), 2);
        assert!(turn.tool_uses.iter().all(|tool| {
            tool.state == "failed"
                && tool.error_code.as_deref() == Some("tool_budget_exceeded")
                && tool.result.is_none()
        }));
        assert!(store
            .context_messages(&session_id)
            .unwrap()
            .iter()
            .all(|message| message.role != "tool"));
        server.abort();
    }

    #[tokio::test]
    async fn write_waits_for_approval_and_captures_checkpoint() {
        let (agent, store, root, server, session_id) = fixture().await;
        let error = agent
            .submit(&session_id, "write-1", "write the file", None)
            .await
            .unwrap_err();
        assert_eq!(error.code, "approval_required");
        assert_eq!(
            std::fs::read_to_string(root.join("README.md")).unwrap(),
            "hello"
        );
        let approval_id = error.details["approval_id"].as_str().unwrap().to_string();
        assert!(agent
            .resolve_approval(&approval_id, "allow_once")
            .await
            .unwrap());
        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        loop {
            if std::fs::read_to_string(root.join("README.md")).unwrap() == "updated" {
                break;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "approval continuation did not complete"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert_eq!(
            std::fs::read_to_string(root.join("README.md")).unwrap(),
            "updated"
        );
        let manifests = store.manifests(&session_id).unwrap();
        assert_eq!(manifests.len(), 1);
        assert_eq!(
            store
                .checkpoint_items(&manifests[0].manifest_id)
                .unwrap()
                .len(),
            1
        );
        server.abort();
    }

    #[tokio::test]
    async fn allow_session_skips_later_approvals_for_the_same_session() {
        let (agent, store, root, server, session_id) = fixture().await;
        let error = agent
            .submit(&session_id, "write-session-1", "write the file", None)
            .await
            .unwrap_err();
        assert_eq!(error.code, "approval_required");
        let approval_id = error.details["approval_id"].as_str().unwrap();
        assert!(agent
            .resolve_approval(approval_id, "allow_session")
            .await
            .unwrap());

        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        loop {
            let completed = store
                .session_trace_turns(&session_id)
                .unwrap()
                .first()
                .map(|turn| turn.state == "completed")
                .unwrap_or(false);
            if completed {
                break;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "session approval continuation did not complete"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert!(store.session_full_control(&session_id).unwrap());

        let response = agent
            .submit(&session_id, "write-session-2", "write again", None)
            .await
            .unwrap();
        assert!(matches!(
            response,
            TurnResponse::Completed { tool_calls: 1, .. }
        ));
        assert_eq!(
            std::fs::read_to_string(root.join("README.md")).unwrap(),
            "updated again"
        );
        server.abort();
    }
}
