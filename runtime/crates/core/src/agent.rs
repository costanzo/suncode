use crate::{
    context,
    domain::{Message, SessionEvent, ToolCall, Usage},
    llm::ProviderError,
    persistence::{PersistenceError, Store},
    policy::{evaluate, tool_risk, Decision, Risk},
};
use base64::{engine::general_purpose::STANDARD, Engine};
use futures_util::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::{broadcast, mpsc, Mutex as AsyncMutex};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

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
        Self::new("runtime_unavailable", error.to_string())
    }
}
impl From<ProviderError> for AgentError {
    fn from(error: ProviderError) -> Self {
        Self::new(error.code, error.message).details(json!({"retryable":error.retryable}))
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
    Queued {
        queued_id: String,
        active_turn_id: String,
        position: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Continuation {
    session_id: String,
    project_id: String,
    project_root: String,
    turn_id: String,
    submission_key: String,
    model: String,
    messages: Vec<Message>,
    iterations: u32,
    tool_calls: u32,
    usage: Usage,
    pending_call: Option<ToolCall>,
    remaining_calls: Vec<ToolCall>,
    #[serde(default)]
    context_compacted: bool,
    #[serde(default)]
    last_tool_signature: Option<String>,
    #[serde(default)]
    repeated_tool_stalls: u32,
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
    providers: Arc<crate::model_provider::ModelProviderRegistry>,
    operations: Arc<suncode_operations::Operations>,
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
        operations: Arc<suncode_operations::Operations>,
        events: broadcast::Sender<SessionEvent>,
        non_interactive: bool,
    ) -> Self
    where
        P: Into<Arc<crate::model_provider::ModelProviderRegistry>>,
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
        let model = model.unwrap_or("deepseek-v4-flash");
        let Some(provider) = self.providers.route(model) else {
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
                    .map_err(|_| AgentError::new("runtime_unavailable", "turn state unavailable"))?
                    .contains_key(session_id);
                if has_active_turn {
                    return self.queue_message(session_id, key, input);
                }
                session_lock.lock().await
            }
        };
        let admission = self.store.begin_turn(session_id, key, input, model)?;
        if !admission.created {
            if admission.status == "completed" {
                let response = admission.response.ok_or_else(|| {
                    AgentError::new("runtime_unavailable", "completed turn has no response")
                })?;
                return serde_json::from_value(response).map_err(|_| {
                    AgentError::new("runtime_unavailable", "stored turn response is invalid")
                });
            }
            return Err(AgentError::new(
                "idempotency_conflict",
                format!("turn submission is {}", admission.status),
            ));
        }
        self.store.mark_turn_started(session_id, key)?;
        let session = self
            .store
            .session_by_id(session_id)?
            .ok_or_else(|| AgentError::new("not_found", "session not found"))?;
        let project_id = session
            .project_id
            .ok_or_else(|| AgentError::new("conflict", "session is not bound to a project"))?;
        let project = self
            .store
            .project_by_id(&project_id)?
            .ok_or_else(|| AgentError::new("not_found", "project not found"))?;
        let token = CancellationToken::new();
        self.cancellations
            .lock()
            .map_err(|_| AgentError::new("runtime_unavailable", "cancellation state unavailable"))?
            .insert(admission.turn_id.clone(), token.clone());
        self.active_turns
            .lock()
            .map_err(|_| AgentError::new("runtime_unavailable", "turn state unavailable"))?
            .insert(session_id.to_string(), admission.turn_id.clone());
        let continuation = Continuation {
            session_id: session_id.into(),
            project_id,
            project_root: project.canonical_root,
            turn_id: admission.turn_id.clone(),
            submission_key: key.into(),
            model: model.into(),
            messages: self.store.context_messages(session_id)?,
            iterations: 0,
            tool_calls: 0,
            usage: Usage::default(),
            pending_call: None,
            remaining_calls: Vec::new(),
            context_compacted: false,
            last_tool_signature: None,
            repeated_tool_stalls: 0,
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
            if error.code != "approval_required" {
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
                    &json!({"code":error.code,"message":error.message}),
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
            .map_err(|_| AgentError::new("runtime_unavailable", "turn state unavailable"))?
            .get(session_id)
            .cloned()
            .ok_or_else(|| AgentError::new("conflict", "session is busy"))?;
        let mut queues = self
            .queued_messages
            .lock()
            .map_err(|_| AgentError::new("runtime_unavailable", "turn queue unavailable"))?;
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
                AgentError::new("runtime_unavailable", "approval continuation is invalid")
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
            .map_err(|_| AgentError::new("runtime_unavailable", "cancellation state unavailable"))?
            .insert(continuation.turn_id.clone(), token.clone());
        self.active_turns
            .lock()
            .map_err(|_| AgentError::new("runtime_unavailable", "turn state unavailable"))?
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
                    &json!({"code":error.code,"message":error.message}),
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

    async fn continue_approved(
        &self,
        continuation: &mut Continuation,
        token: CancellationToken,
    ) -> Result<TurnResponse, AgentError> {
        if let Some(call) = continuation.pending_call.take() {
            self.execute_call(continuation, &call).await?;
        }
        let siblings = std::mem::take(&mut continuation.remaining_calls);
        self.resolve_calls(continuation, siblings).await?;
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
                AgentError::new("runtime_unavailable", "turn response could not be stored")
            })?,
        )?;
        Ok(response)
    }

    async fn run(
        &self,
        mut context: Continuation,
        user_input: Option<&str>,
        token: CancellationToken,
        provider: crate::model_provider::ModelRoute,
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
            let result = {
                let (delta_sender, mut delta_receiver) = mpsc::unbounded_channel();
                let provider = provider.provider.complete(
                    &context.messages,
                    provider.wire_model,
                    &token,
                    delta_sender,
                );
                tokio::pin!(provider);
                let result = loop {
                    tokio::select! {
                        value = &mut provider => break value?,
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
            if result.text.len() > 8 * 1024 * 1024 {
                return self.fail_context(
                    &context,
                    "output_budget_exceeded",
                    "Provider output exceeded its budget",
                );
            }
            if let Some(usage) = result.usage {
                context.usage.add(&usage);
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
                tool_calls: result.tool_calls.clone(),
                tool_call_id: None,
            };
            context.messages.push(assistant.clone());
            self.emit(&context.session_id, "message.assistant", json!({"message_id":Uuid::new_v4(),"turn_id":context.turn_id,"message":assistant,"usage":context.usage,"finish_reason":result.finish_reason}))?;
            if result.tool_calls.is_empty() {
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
                        AgentError::new("runtime_unavailable", "turn response could not be stored")
                    })?,
                )?;
                return Ok(response);
            }
            self.turn_state(&context, "resolving_calls", None)?;
            self.resolve_calls(&mut context, result.tool_calls).await?;
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
                .map_err(|_| AgentError::new("runtime_unavailable", "turn queue unavailable"))?;
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
    ) -> Result<(), AgentError> {
        let mut allowed_calls = Vec::new();
        for (index, call) in calls.iter().enumerate() {
            context.tool_calls += 1;
            if context.tool_calls > 32 {
                return self.fail_context::<()>(
                    context,
                    "tool_budget_exceeded",
                    "Turn exceeded its tool-call budget",
                );
            }
            self.emit(&context.session_id, "tool.requested", json!({"turn_id":context.turn_id,"tool_call_id":call.call_id,"name":call.name,"arguments":call.arguments}))?;
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
                self.tool_state(context, call, "failed", Some("malformed_tool_call"))?;
                return Err(AgentError::new(
                    "malformed_tool_call",
                    "Tool arguments must be an object",
                ));
            }
            self.tool_state(context, call, "policy_check", None)?;
            let decision = evaluate(tool_risk(&call.name), self.non_interactive);
            self.store.append_audit(Some(&context.project_id), Some(&context.session_id), Some(&context.turn_id), "capability.decision", &json!({"tool_call_id":call.call_id,"operation":call.name,"decision":format!("{decision:?}")}))?;
            match decision {
                Decision::Deny => {
                    self.execute_allowed_calls(context, std::mem::take(&mut allowed_calls))
                        .await?;
                    self.tool_state(context, call, "denied", Some("authorization_denied"))?;
                    return Err(AgentError::new(
                        "authorization_denied",
                        format!("Tool call denied: {}", call.name),
                    ));
                }
                Decision::ApprovalRequired => {
                    self.execute_allowed_calls(context, std::mem::take(&mut allowed_calls))
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
                            "runtime_unavailable",
                            "turn continuation could not be stored",
                        )
                    })?;
                    let approval =
                        self.store
                            .create_approval(crate::persistence::ApprovalInput {
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
        self.execute_allowed_calls(context, allowed_calls).await?;
        Ok(())
    }

    async fn execute_allowed_calls(
        &self,
        context: &mut Continuation,
        calls: Vec<ToolCall>,
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
                self.execute_call(context, &call).await?;
            }
            return Ok(());
        }

        let mut futures = Vec::with_capacity(calls.len());
        for call in calls {
            self.tool_state(context, &call, "authorized", None)?;
            self.tool_state(context, &call, "executing", None)?;
            let mut params = translate_arguments(&call.name, &call.arguments)?;
            params["idempotency_key"] = json!(format!("{}:{}", context.turn_id, call.call_id));
            let method = method_name(&call.name)
                .ok_or_else(|| AgentError::new("authorization_denied", "Unknown tool"))?
                .to_string();
            let project_root = context.project_root.clone();
            let agent = self.clone();
            futures.push(async move {
                let result = agent
                    .operation_in_project(&project_root, &method, params)
                    .await;
                (call, result)
            });
        }

        for (call, result) in join_all(futures).await {
            let result = match result {
                Ok(result) => result,
                Err(error) => {
                    self.tool_state(context, &call, "failed", Some("execution_failed"))?;
                    return Err(error);
                }
            };
            self.record_call_success(context, &call, result)?;
        }
        Ok(())
    }

    async fn execute_call(
        &self,
        context: &mut Continuation,
        call: &ToolCall,
    ) -> Result<(), AgentError> {
        self.tool_state(context, call, "authorized", None)?;
        self.tool_state(context, call, "executing", None)?;
        let mut params = translate_arguments(&call.name, &call.arguments)?;
        params["idempotency_key"] = json!(format!("{}:{}", context.turn_id, call.call_id));
        let method = method_name(&call.name)
            .ok_or_else(|| AgentError::new("authorization_denied", "Unknown tool"))?;
        let result = self
            .operation_in_project(&context.project_root, method, params)
            .await
            .inspect_err(|_error| {
                let _ = self.tool_state(context, call, "failed", Some("execution_failed"));
            })?;
        self.record_call_success(context, call, result)
    }

    fn record_call_success(
        &self,
        context: &mut Continuation,
        call: &ToolCall,
        result: Value,
    ) -> Result<(), AgentError> {
        self.tool_state(context, call, "succeeded", None)?;
        self.store.append_audit(
            Some(&context.project_id),
            Some(&context.session_id),
            Some(&context.turn_id),
            "operation.result",
            &json!({"tool_call_id":call.call_id,"operation":call.name,"outcome":"succeeded"}),
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
            serde_json::to_string(&normalize_result(&call.name, result))
                .unwrap_or_else(|_| "{}".into()),
        );
        tool.tool_call_id = Some(call.call_id.clone());
        context.messages.push(tool.clone());
        self.emit(
            &context.session_id,
            "message.tool",
            json!({"turn_id":context.turn_id,"tool_call_id":call.call_id,"message":tool}),
        )?;
        Ok(())
    }

    async fn operation_in_project(
        &self,
        project_root: &str,
        method: &str,
        params: Value,
    ) -> Result<Value, AgentError> {
        let operations = self.operations.clone();
        let root = std::path::PathBuf::from(project_root);
        let method = method.to_string();
        tokio::task::spawn_blocking(move || operations.execute_in_project(&root, &method, params))
            .await
            .map_err(|_| AgentError::new("runtime_unavailable", "operation task failed"))?
            .map_err(|error| {
                AgentError::new(
                    error
                        .get("code")
                        .and_then(Value::as_str)
                        .unwrap_or("operation_failed"),
                    error
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("operation failed"),
                )
            })
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
            content_sequence: 0,
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
        self.emit(&context.session_id,"tool.state",json!({"turn_id":context.turn_id,"tool_call_id":call.call_id,"name":call.name,"state":state,"reason":reason}))
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
                    AgentError::new("runtime_unavailable", "approval continuation is invalid")
                })?;
            let token = CancellationToken::new();
            self.cancellations
                .lock()
                .map_err(|_| {
                    AgentError::new("runtime_unavailable", "cancellation state unavailable")
                })?
                .insert(continuation.turn_id.clone(), token.clone());
            self.active_turns
                .lock()
                .map_err(|_| AgentError::new("runtime_unavailable", "turn state unavailable"))?
                .insert(
                    continuation.session_id.clone(),
                    continuation.turn_id.clone(),
                );
            let agent = self.clone();
            let approval_id = suspended.approval_id;
            tokio::spawn(async move {
                let session_lock = agent.session_lock(&continuation.session_id).await;
                let _guard = session_lock.lock().await;
                let result = agent.continue_approved(&mut continuation, token).await;
                let _ = agent.store.finish_suspended(
                    &approval_id,
                    if result.is_ok() {
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
        "apply_patch" => Some("tool/apply_patch"),
        "bash" => Some("tool/bash"),
        "project_inspect" | "project.inspect" => Some("project/inspect"),
        "fs_read" | "fs.read" => Some("fs/read"),
        "fs_metadata" | "fs.metadata" => Some("fs/metadata"),
        "search_glob" | "search.glob" => Some("search/glob"),
        "search_find" | "search.find" => Some("search/find"),
        "fs_write" | "fs.write" => Some("fs/write"),
        "fs_edit" | "fs.edit" => Some("fs/edit"),
        "fs_patch" | "fs.patch" => Some("fs/patch"),
        "fs_move" | "fs.move" => Some("fs/move"),
        "fs_delete" | "fs.delete" => Some("fs/delete"),
        "process_run" | "process.run" => Some("process/run"),
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
    if matches!(name, "write" | "fs_write" | "fs.write") {
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
        result["replacements"] = json!([{"old": old, "new": new, "replace_all": replace_all}]);
        if let Some(object) = result.as_object_mut() {
            object.remove("oldString");
            object.remove("newString");
            object.remove("replaceAll");
        }
    }
    if name == "apply_patch" {
        let patch = result
            .get("patchText")
            .and_then(Value::as_str)
            .ok_or_else(|| AgentError::new("invalid_arguments", "patchText is required"))?;
        result["patch"] = json!(patch);
        if let Some(object) = result.as_object_mut() {
            object.remove("patchText");
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
            .get("query")
            .or_else(|| result.get("pattern"))
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
            .map(str::to_string)
            .ok_or_else(|| AgentError::new("invalid_arguments", "command is required"))?;
        result["command"] = json!("/bin/sh");
        result["args"] = json!(["-lc", command]);
        if let Some(workdir) = result.get("workdir").cloned() {
            result["cwd"] = workdir;
        }
        if let Some(timeout) = result.get("timeout").cloned() {
            result["timeout_ms"] = timeout;
        }
        if let Some(object) = result.as_object_mut() {
            object.remove("workdir");
            object.remove("timeout");
        }
    }
    Ok(result)
}

fn scoped_glob(path: &str, pattern: &str) -> String {
    let base = path.trim_matches('/');
    if base.is_empty() || base == "." {
        return pattern.to_string();
    }
    format!("{base}/{}", pattern.trim_start_matches('/'))
}

fn normalize_result(name: &str, mut value: Value) -> Value {
    if matches!(name, "read" | "fs_read" | "fs.read") {
        if let Some(encoded) = value
            .get("data_base64")
            .and_then(Value::as_str)
            .map(str::to_string)
        {
            if let Ok(bytes) = STANDARD.decode(&encoded) {
                if let Ok(text) = String::from_utf8(bytes) {
                    value["content"] = json!(text);
                    value["precondition_base64"] = json!(encoded);
                }
            }
        }
        if let Some(object) = value.as_object_mut() {
            object.remove("data_base64");
        }
    }
    if name == "bash" {
        for (encoded_key, text_key) in [("stdout_base64", "stdout"), ("stderr_base64", "stderr")] {
            if let Some(encoded) = value
                .get(encoded_key)
                .and_then(Value::as_str)
                .map(str::to_string)
            {
                if let Ok(bytes) = STANDARD.decode(&encoded) {
                    if let Ok(text) = String::from_utf8(bytes) {
                        value[text_key] = json!(text);
                    }
                }
            }
            if let Some(object) = value.as_object_mut() {
                object.remove(encoded_key);
            }
        }
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::CredentialStore;
    use axum::{http::header, response::IntoResponse, routing::post, Json, Router};

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
        let user_text = messages
            .iter()
            .rev()
            .find(|message| message.get("role").and_then(Value::as_str) == Some("user"))
            .and_then(|message| message.get("content"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        if user_text.contains("slow") {
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
        let data = if last_role == Some("tool") {
            vec![
                json!({"choices":[{"delta":{"content":"done"},"finish_reason":"stop"}],"usage":{"prompt_tokens":8,"completion_tokens":1,"total_tokens":9}}),
            ]
        } else if user_text.contains("slow") || user_text.contains("follow up") {
            vec![
                json!({"choices":[{"delta":{"content":"queued done"},"finish_reason":"stop"}],"usage":{"prompt_tokens":4,"completion_tokens":2,"total_tokens":6}}),
            ]
        } else if user_text.contains("read two") {
            vec![json!({"choices":[{"delta":{"tool_calls":[
                    {"index":0,"id":"read-call-1","function":{"name":"read","arguments":"{\"path\":\"README.md\"}"}},
                    {"index":1,"id":"read-call-2","function":{"name":"read","arguments":"{\"path\":\"README.md\"}"}}
                ]},"finish_reason":"tool_calls"}]})]
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
        let session = store
            .create_session(&project.project_id, None, Some("deepseek-v4-flash"))
            .unwrap();
        let operations =
            Arc::new(suncode_operations::Operations::new(directory.join(".operations")).unwrap());
        let (events, _) = broadcast::channel(64);
        let registry = crate::model_provider::ModelProviderRegistry::with_deepseek_endpoint(
            format!("http://{address}"),
            CredentialStore::memory(Some("test-key"), None, None, None, None, None),
        );
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
        let (agent, store, _root, server, session_id) = fixture().await;
        let response = agent
            .submit(&session_id, "read-1", "read the file", None)
            .await
            .unwrap();
        assert!(matches!(
            response,
            TurnResponse::Completed { tool_calls: 1, .. }
        ));
        let events = store.events(&session_id, 0).unwrap();
        assert!(
            events
                .iter()
                .any(|event| event.event_type == "tool.state"
                    && event.payload["state"] == "succeeded")
        );
        assert!(!events
            .iter()
            .any(|event| event.event_type == "assistant.delta"));
        assert!(events.iter().any(|event| {
            event.event_type == "turn.state" && event.payload["state"] == "completed"
        }));
        assert_eq!(events.last().unwrap().event_type, "turn.completed");
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
        let events = store.events(&session_id, 0).unwrap();
        assert!(events.iter().any(|event| event.event_type == "turn.queued"));
        assert!(events.iter().any(|event| {
            event.event_type == "message.user" && event.payload.get("queued_id").is_some()
        }));
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
        let events = store.events(&session_id, 0).unwrap();
        let second_requested = events
            .iter()
            .position(|event| {
                event.event_type == "tool.state"
                    && event.payload["tool_call_id"] == "read-call-2"
                    && event.payload["state"] == "requested"
            })
            .unwrap();
        let first_executing = events
            .iter()
            .position(|event| {
                event.event_type == "tool.state"
                    && event.payload["tool_call_id"] == "read-call-1"
                    && event.payload["state"] == "executing"
            })
            .unwrap();
        assert!(second_requested < first_executing);
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
            if store.events(&session_id, 0).unwrap().iter().any(|event| {
                event.event_type == "turn.state" && event.payload["state"] == "completed"
            }) {
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
}
