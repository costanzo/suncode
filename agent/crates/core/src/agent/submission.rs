//! Part of the `AgentRuntime` implementation; see `agent/mod.rs`.

use super::run::*;
use super::support::*;
use super::*;

impl AgentRuntime {
    pub fn new<P>(
        store: Store,
        providers: P,
        operations: Arc<suncode_tool::Operations>,
        events: SessionEventHub,
        non_interactive: bool,
    ) -> Self
    where
        P: Into<Arc<ModelProviderRegistry>>,
    {
        Self::new_with_mcp_configuration(
            store,
            providers,
            operations,
            events,
            non_interactive,
            std::env::temp_dir().join("suncode-mcp"),
        )
    }

    pub fn new_with_mcp_configuration<P>(
        store: Store,
        providers: P,
        operations: Arc<suncode_tool::Operations>,
        events: SessionEventHub,
        non_interactive: bool,
        application_data: PathBuf,
    ) -> Self
    where
        P: Into<Arc<ModelProviderRegistry>>,
    {
        Self::new_with_user_id_and_capabilities(
            store,
            providers,
            operations,
            events,
            non_interactive,
            application_data,
            "default".into(),
            AgentHostCapabilities::default(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_user_id_and_capabilities<P>(
        store: Store,
        providers: P,
        operations: Arc<suncode_tool::Operations>,
        events: SessionEventHub,
        non_interactive: bool,
        application_data: PathBuf,
        user_id: String,
        host_capabilities: AgentHostCapabilities,
    ) -> Self
    where
        P: Into<Arc<ModelProviderRegistry>>,
    {
        let config = Config::disabled();
        Self::new_with_user_id_and_capabilities_and_config(
            store,
            providers,
            operations,
            events,
            non_interactive,
            application_data,
            user_id,
            host_capabilities,
            config,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_user_id_and_capabilities_and_config<P>(
        store: Store,
        providers: P,
        operations: Arc<suncode_tool::Operations>,
        events: SessionEventHub,
        non_interactive: bool,
        application_data: PathBuf,
        user_id: String,
        host_capabilities: AgentHostCapabilities,
        config: Config,
    ) -> Self
    where
        P: Into<Arc<ModelProviderRegistry>>,
    {
        let mcp = suncode_mcp::McpRuntime::new(Arc::new(mcp::StoreServerSource {
            store: store.clone(),
            application_data: application_data.clone(),
        }));
        let lsp = suncode_lsp::LspRuntime::new(Arc::new(lsp::StoreServerSource {
            store: store.clone(),
        }));
        let browser = browser::new_runtime(
            &store,
            application_data.clone(),
            host_capabilities.browser_use,
        );
        let preview = preview::PreviewManager::new(operations.clone());
        let computer = suncode_computer::ComputerRuntime::new(
            computer::enabled_setting(&store),
            host_capabilities.computer_use,
            application_data.clone(),
        );
        // Registration order is the advertised tool order.
        let tool_providers = providers::ToolProviders::new(vec![
            Arc::new(suncode_tool::BuiltinTools::new(operations.clone())),
            Arc::new(lsp.clone()),
            Arc::new(mcp.clone()),
            Arc::new(browser.clone()),
            Arc::new(computer.clone()),
        ]);
        Self {
            services: services::TurnServices {
                store,
                user_id,
                providers: providers.into(),
                operations,
                events,
                attention_events: AttentionEventHub::new(64),
                queued_messages: Arc::new(Mutex::new(HashMap::new())),
                tool_providers,
                non_interactive,
                host_capabilities,
                config,
            },
            cancellations: Arc::new(Mutex::new(HashMap::new())),
            active_turns: Arc::new(Mutex::new(HashMap::new())),
            session_locks: Arc::new(AsyncMutex::new(HashMap::new())),
            mcp,
            lsp,
            browser,
            preview,
            computer,
            shutting_down: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn new_with_user_id<P>(
        store: Store,
        providers: P,
        operations: Arc<suncode_tool::Operations>,
        events: SessionEventHub,
        non_interactive: bool,
        application_data: PathBuf,
        user_id: String,
    ) -> Self
    where
        P: Into<Arc<ModelProviderRegistry>>,
    {
        Self::new_with_user_id_and_capabilities(
            store,
            providers,
            operations,
            events,
            non_interactive,
            application_data,
            user_id,
            AgentHostCapabilities::default(),
        )
    }

    #[cfg(test)]
    pub async fn submit(
        &self,
        session_id: &str,
        key: &str,
        input: &str,
        model: Option<&str>,
        reasoning_effort: Option<&str>,
    ) -> Result<TurnResponse, BusinessError> {
        self.submit_with_attachments(session_id, key, input, model, reasoning_effort, &[])
            .await
    }

    pub async fn submit_with_attachments(
        &self,
        session_id: &str,
        key: &str,
        input: &str,
        model: Option<&str>,
        reasoning_effort: Option<&str>,
        image_ids: &[String],
    ) -> Result<TurnResponse, BusinessError> {
        self.submit_session(
            session_id,
            key,
            input,
            model,
            reasoning_effort,
            image_ids,
            false,
            None,
            false,
            true,
            None,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) async fn submit_session(
        &self,
        session_id: &str,
        key: &str,
        input: &str,
        model: Option<&str>,
        reasoning_effort: Option<&str>,
        image_ids: &[String],
        allow_child: bool,
        inherited_token: Option<CancellationToken>,
        retry: bool,
        schedule_followups: bool,
        inherited_shell: Option<Option<suncode_common::shell::ResolvedShell>>,
    ) -> Result<TurnResponse, BusinessError> {
        self.ensure_running()?;
        let session_lock = self.session_lock(session_id).await;
        let model = match model {
            Some(model) => model.to_string(),
            None => {
                let session = self
                    .services
                    .store
                    .session_by_id(session_id)?
                    .ok_or_else(|| BusinessError::new("not_found", "session not found"))?;
                let project_id = session.project_id.ok_or_else(|| {
                    BusinessError::new("conflict", "session is not bound to a project")
                })?;
                self.services
                    .store
                    .project_settings(&project_id)?
                    .default_model
                    .unwrap_or_else(|| "deepseek-v4-flash".into())
            }
        };
        let Some(provider) = self.services.providers.route(&model) else {
            return Err(BusinessError::new(
                "model_unavailable",
                "model is not advertised",
            ));
        };
        let reasoning_effort = reasoning_effort.filter(|value| !value.trim().is_empty());
        let reasoning_effort = if self.services.providers.supports_reasoning_effort(&model) {
            let supported = self.services.providers.reasoning_efforts(&model);
            match reasoning_effort {
                Some(value) => {
                    if !supported.iter().any(|effort| effort == value) {
                        return Err(BusinessError::new(
                            "invalid_arguments",
                            format!("reasoning_effort is not supported by model `{model}`"),
                        ));
                    }
                    Some(value.to_owned())
                }
                None => supported.first().cloned(),
            }
        } else {
            if reasoning_effort.is_some() {
                return Err(BusinessError::new(
                    "invalid_arguments",
                    "selected model does not support reasoning effort",
                ));
            }
            None
        };
        let _guard = match session_lock.try_lock() {
            Ok(guard) => guard,
            Err(_) => {
                let has_active_turn = self
                    .active_turns
                    .lock()
                    .map_err(|_| BusinessError::new("agent_unavailable", "turn state unavailable"))?
                    .contains_key(session_id);
                if has_active_turn {
                    if retry {
                        return Err(BusinessError::new(
                            "conflict",
                            "a turn is already running in this session",
                        ));
                    }
                    return self.queue_message(session_id, key, input, image_ids);
                }
                session_lock.lock().await
            }
        };
        let images = self
            .services
            .validate_message_images(session_id, &model, image_ids, true)?;
        let user_message = message_with_image_refs(input, &images);
        let session = self
            .services
            .store
            .session_by_id(session_id)?
            .ok_or_else(|| BusinessError::new("not_found", "session not found"))?;
        if session.kind == "child" && !allow_child {
            return Err(BusinessError::new(
                "child_session_read_only",
                "child sessions can only be started by the parent agent",
            ));
        }
        let agent_definition = suncode_agents::by_id(session.agent_id.as_deref());
        if session.kind == "child" && agent_definition.is_none_or(|agent| agent.is_primary()) {
            return Err(BusinessError::new(
                "agent_unavailable",
                "child session agent is not available",
            ));
        }
        let session_started_at = session.created_at.clone();
        let project_id = session
            .project_id
            .ok_or_else(|| BusinessError::new("conflict", "session is not bound to a project"))?;
        let project = self
            .services
            .store
            .project_by_id_for_user(&self.services.user_id, &project_id)?
            .ok_or_else(|| BusinessError::new("not_found", "project not found"))?;
        let project_tool_call_limit = self
            .services
            .store
            .project_settings(&project_id)?
            .tool_call_limit
            .unwrap_or(DEFAULT_TOOL_CALL_LIMIT);
        let runtime = self.runtime_for(&project_id, session_id)?;
        let agent_definition = agent_definition.unwrap_or(&suncode_agents::PRIMARY);
        let tool_call_limit = agent_definition
            .tool_call_limit
            .map(|limit| {
                project_tool_call_limit
                    .min(runtime.tool_call_limit.unwrap_or(project_tool_call_limit))
                    .min(limit)
            })
            .unwrap_or_else(|| {
                project_tool_call_limit
                    .min(runtime.tool_call_limit.unwrap_or(project_tool_call_limit))
            });
        let shell = match inherited_shell {
            Some(shell) => shell,
            None => self.services.store.global_settings()?.agent_shell.resolved,
        };
        let admission = db_blocking(|| {
            self.services.store.begin_turn_with_images(
                session_id,
                key,
                input,
                &model,
                reasoning_effort.as_deref(),
                image_ids,
            )
        })?;
        if !admission.created {
            if admission.status == "completed" {
                let response = admission.response.ok_or_else(|| {
                    BusinessError::new("agent_unavailable", "completed turn has no response")
                })?;
                return serde_json::from_value(response).map_err(|_| {
                    BusinessError::new("agent_unavailable", "stored turn response is invalid")
                });
            }
            return Err(BusinessError::new(
                "idempotency_conflict",
                format!("turn submission is {}", admission.status),
            ));
        }
        db_blocking(|| self.services.store.mark_turn_started(session_id, key))?;
        let token = inherited_token.unwrap_or_default();
        self.cancellations
            .lock()
            .map_err(|_| BusinessError::new("agent_unavailable", "cancellation state unavailable"))?
            .insert(admission.turn_id.clone(), token.clone());
        self.active_turns
            .lock()
            .map_err(|_| BusinessError::new("agent_unavailable", "turn state unavailable"))?
            .insert(session_id.to_string(), admission.turn_id.clone());
        let continuation = Continuation {
            shell,
            session_id: session_id.into(),
            session_started_at,
            project_id,
            project_root: project.canonical_root,
            agent_id: session.agent_id.clone(),
            agent_version: session.agent_version,
            // Snapshotted with the turn; an empty list means the full catalog.
            allowed_tools: match agent_definition.tools {
                suncode_agents::ToolScope::Catalog => Vec::new(),
                suncode_agents::ToolScope::Only(tools) => {
                    tools.iter().map(|tool| (*tool).to_string()).collect()
                }
            },
            turn_id: admission.turn_id.clone(),
            submission_key: key.into(),
            model,
            reasoning_effort,
            messages: db_blocking(|| self.services.store.context_messages(session_id))?,
            iterations: 0,
            tool_calls: 0,
            tool_call_limit,
            usage: Usage::default(),
            pending_call: None,
            pending_approval_pin: None,
            approved_batch_call_ids: Vec::new(),
            approval_batch_failed: false,
            remaining_calls: Vec::new(),
            overflow_recovery_attempted: false,
            last_tool_signature: None,
            repeated_tool_stalls: 0,
            active_call_id: None,
            question_answers: None,
            question_rejected: false,
            todos: Vec::new(),
            loaded_instruction_paths: Vec::new(),
            turn_timeout_ms: runtime.turn_timeout_ms,
            bash_timeout_ms: runtime.bash_timeout_ms,
            full_control_override: runtime.full_control,
            host_tool_allowlist: runtime
                .tool_allowlist
                .unwrap_or_default()
                .into_iter()
                .collect(),
        };
        let start = if retry {
            TurnStart::Retry
        } else {
            TurnStart::Submit(user_message)
        };
        let mut turn_context = continuation.clone();
        let result = TurnLoop::new(self, &mut turn_context, token)
            .run(start, provider)
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
            logging::write_business_error(
                "turn",
                "submit",
                error,
                format!(
                    "session={} turn={}",
                    continuation.session_id, continuation.turn_id
                ),
            );
            if !states::suspends_turn(&error.code) {
                let _ = self.services.turn_state(
                    &continuation,
                    states::failed_turn_state(&error.code),
                    Some(&error.code),
                );
                db_blocking(|| {
                    self.services.store.fail_turn(
                        session_id,
                        key,
                        &json!({"code":error.code,"message":error.message,"details":error.details}),
                    )
                })?;
            }
        }
        let turn_ended = matches!(result, Ok(TurnResponse::Completed { .. }))
            || result
                .as_ref()
                .err()
                .is_some_and(|error| !states::suspends_turn(&error.code));
        if turn_ended {
            if let Ok(mut queues) = self.services.queued_messages.lock() {
                if let Some(queue) = queues.get_mut(session_id) {
                    for message in queue {
                        if message.mode == QueuedMessageMode::Steering {
                            message.mode = QueuedMessageMode::Followup;
                        }
                    }
                }
            }
        }
        drop(_guard);
        let has_followups = self
            .services
            .queued_messages
            .lock()
            .map(|queues| {
                queues.get(session_id).is_some_and(|queue| {
                    queue
                        .iter()
                        .any(|message| message.mode == QueuedMessageMode::Followup)
                })
            })
            .unwrap_or(false);
        if schedule_followups
            && has_followups
            && matches!(result, Ok(TurnResponse::Completed { .. }))
        {
            let runtime = self.clone();
            let session_id = session_id.to_string();
            let spawn_result = std::thread::Builder::new()
                .name("suncode-followup-queue".into())
                .spawn(move || {
                    let Ok(worker_runtime) = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                    else {
                        return;
                    };
                    worker_runtime.block_on(runtime.drain_followup_messages(session_id));
                });
            if let Err(error) = spawn_result {
                logging::write_business_error(
                    "turn",
                    "followup_worker",
                    &BusinessError::new("agent_unavailable", error.to_string()),
                    "failed to start queued follow-up worker",
                );
            }
        }
        result
    }

    pub(super) async fn submit_child(
        &self,
        session_id: &str,
        input: &str,
        model: &str,
        reasoning_effort: Option<&str>,
        parent_token: CancellationToken,
        shell: Option<suncode_common::shell::ResolvedShell>,
    ) -> Result<TurnResponse, BusinessError> {
        self.submit_session(
            session_id,
            &suncode_common::id::new(suncode_common::id::IdKind::Submission),
            input,
            Some(model),
            reasoning_effort,
            &[],
            true,
            Some(parent_token),
            false,
            false,
            Some(shell),
        )
        .await
    }

    /// Retry the session's latest turn when it failed. The retry is admitted as a new turn
    /// /// that continues from the persisted context, so the original user message is not repeated.
    pub async fn retry_last_turn(&self, session_id: &str) -> Result<TurnResponse, BusinessError> {
        let Some((input, model, image_ids)) =
            self.services.store.latest_failed_turn_input(session_id)?
        else {
            return Err(BusinessError::new("conflict", "no failed turn to retry"));
        };
        if model.is_empty() {
            return Err(BusinessError::new(
                "agent_unavailable",
                "failed turn has no model",
            ));
        }
        self.submit_session(
            session_id,
            &suncode_common::id::new(suncode_common::id::IdKind::Submission),
            &input,
            Some(&model),
            None,
            &image_ids,
            false,
            None,
            true,
            true,
            None,
        )
        .await
    }

    pub(super) fn clear_queued_messages(&self, session_id: &str) {
        self.services
            .queued_messages
            .lock()
            .ok()
            .map(|mut values| values.remove(session_id));
    }

    pub(super) fn queue_message(
        &self,
        session_id: &str,
        key: &str,
        input: &str,
        image_ids: &[String],
    ) -> Result<TurnResponse, BusinessError> {
        self.services
            .validate_message_images(session_id, "", image_ids, false)?;
        let active_turn_id = self
            .active_turns
            .lock()
            .map_err(|_| BusinessError::new("agent_unavailable", "turn state unavailable"))?
            .get(session_id)
            .cloned()
            .ok_or_else(|| BusinessError::new("conflict", "session is busy"))?;
        let mut queues = self
            .services
            .queued_messages
            .lock()
            .map_err(|_| BusinessError::new("agent_unavailable", "turn queue unavailable"))?;
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
        let queued_id = suncode_common::id::new(suncode_common::id::IdKind::QueuedMessage);
        queue.push_back(QueuedMessage {
            queued_id: queued_id.clone(),
            idempotency_key: key.to_string(),
            input: input.to_string(),
            image_ids: image_ids.to_vec(),
            mode: QueuedMessageMode::Followup,
        });
        let position = queue.len();
        drop(queues);
        self.services.emit(
            session_id,
            EventPayload::TurnQueued(TurnQueuedPayload {
                queued_id: queued_id.clone(),
                active_turn_id: active_turn_id.clone(),
                position,
            }),
        )?;
        Ok(TurnResponse::Queued {
            queued_id,
            active_turn_id,
            position,
        })
    }

    pub fn queued_messages(
        &self,
        session_id: &str,
    ) -> Result<Vec<QueuedMessageSnapshot>, BusinessError> {
        let queues = self
            .services
            .queued_messages
            .lock()
            .map_err(|_| BusinessError::new("agent_unavailable", "turn queue unavailable"))?;
        Ok(queues
            .get(session_id)
            .into_iter()
            .flatten()
            .enumerate()
            .map(|(index, message)| QueuedMessageSnapshot {
                queued_id: message.queued_id.clone(),
                position: index + 1,
                mode: message.mode.clone(),
                input: message.input.clone(),
                image_ids: message.image_ids.clone(),
            })
            .collect())
    }

    pub fn steer_queued_message(
        &self,
        session_id: &str,
        queued_id: &str,
    ) -> Result<QueuedMessageSnapshot, BusinessError> {
        if !self
            .active_turns
            .lock()
            .map_err(|_| BusinessError::new("agent_unavailable", "turn state unavailable"))?
            .contains_key(session_id)
        {
            return Err(BusinessError::new("conflict", "no active turn to steer"));
        }
        let mut queues = self
            .services
            .queued_messages
            .lock()
            .map_err(|_| BusinessError::new("agent_unavailable", "turn queue unavailable"))?;
        let queue = queues
            .get_mut(session_id)
            .ok_or_else(|| BusinessError::missing("queued_message"))?;
        let (index, message) = queue
            .iter_mut()
            .enumerate()
            .find(|(_, message)| message.queued_id == queued_id)
            .ok_or_else(|| BusinessError::missing("queued_message"))?;
        message.mode = QueuedMessageMode::Steering;
        Ok(QueuedMessageSnapshot {
            queued_id: message.queued_id.clone(),
            position: index + 1,
            mode: message.mode.clone(),
            input: message.input.clone(),
            image_ids: message.image_ids.clone(),
        })
    }

    pub fn take_queued_message(
        &self,
        session_id: &str,
        queued_id: &str,
    ) -> Result<QueuedMessageSnapshot, BusinessError> {
        let mut queues = self
            .services
            .queued_messages
            .lock()
            .map_err(|_| BusinessError::new("agent_unavailable", "turn queue unavailable"))?;
        let queue = queues
            .get_mut(session_id)
            .ok_or_else(|| BusinessError::missing("queued_message"))?;
        let index = queue
            .iter()
            .position(|message| message.queued_id == queued_id)
            .ok_or_else(|| BusinessError::missing("queued_message"))?;
        let message = queue
            .remove(index)
            .ok_or_else(|| BusinessError::missing("queued_message"))?;
        Ok(QueuedMessageSnapshot {
            queued_id: message.queued_id,
            position: index + 1,
            mode: message.mode,
            input: message.input,
            image_ids: message.image_ids,
        })
    }

    pub fn take_queued_message_replacing_with_followup(
        &self,
        session_id: &str,
        queued_id: &str,
        input: &str,
        image_ids: &[String],
    ) -> Result<QueuedMessageSnapshot, BusinessError> {
        self.services
            .validate_message_images(session_id, "", image_ids, false)?;
        let mut queues = self
            .services
            .queued_messages
            .lock()
            .map_err(|_| BusinessError::new("agent_unavailable", "turn queue unavailable"))?;
        let queue = queues
            .get_mut(session_id)
            .ok_or_else(|| BusinessError::missing("queued_message"))?;
        let index = queue
            .iter()
            .position(|message| message.queued_id == queued_id)
            .ok_or_else(|| BusinessError::missing("queued_message"))?;
        let message = queue
            .remove(index)
            .ok_or_else(|| BusinessError::missing("queued_message"))?;
        let snapshot = QueuedMessageSnapshot {
            queued_id: message.queued_id,
            position: index + 1,
            mode: message.mode,
            input: message.input,
            image_ids: message.image_ids,
        };
        let replacement_id = suncode_common::id::new(suncode_common::id::IdKind::QueuedMessage);
        queue.insert(
            index,
            QueuedMessage {
                queued_id: replacement_id.clone(),
                idempotency_key: format!("draft:{replacement_id}"),
                input: input.to_string(),
                image_ids: image_ids.to_vec(),
                mode: QueuedMessageMode::Followup,
            },
        );
        Ok(snapshot)
    }

    pub fn restore_queued_message(
        &self,
        session_id: &str,
        message: QueuedMessageSnapshot,
    ) -> Result<QueuedMessageSnapshot, BusinessError> {
        let mut queues = self
            .services
            .queued_messages
            .lock()
            .map_err(|_| BusinessError::new("agent_unavailable", "turn queue unavailable"))?;
        let queue = queues.entry(session_id.to_string()).or_default();
        if queue.iter().any(|item| item.queued_id == message.queued_id) {
            return Err(BusinessError::new(
                "conflict",
                "queued message already exists",
            ));
        }
        let index = message.position.saturating_sub(1).min(queue.len());
        queue.insert(
            index,
            QueuedMessage {
                queued_id: message.queued_id.clone(),
                idempotency_key: format!("restored:{}", message.queued_id),
                input: message.input.clone(),
                image_ids: message.image_ids.clone(),
                mode: message.mode.clone(),
            },
        );
        Ok(message)
    }

    async fn drain_followup_messages(&self, session_id: String) {
        loop {
            if self.ensure_running().is_err() {
                break;
            }
            let next = {
                let Ok(mut queues) = self.services.queued_messages.lock() else {
                    break;
                };
                let Some(queue) = queues.get_mut(&session_id) else {
                    break;
                };
                if queue
                    .front()
                    .is_some_and(|message| message.mode == QueuedMessageMode::Steering)
                {
                    if let Some(message) = queue.front_mut() {
                        message.mode = QueuedMessageMode::Followup;
                    }
                }
                let Some(message) = queue.pop_front() else {
                    break;
                };
                Some(message)
            };
            let Some(message) = next else {
                break;
            };
            match Box::pin(self.submit_session(
                &session_id,
                &message.idempotency_key,
                &message.input,
                None,
                None,
                &message.image_ids,
                false,
                None,
                false,
                false,
                None,
            ))
            .await
            {
                Ok(TurnResponse::Completed { .. }) => {}
                Ok(TurnResponse::Queued { queued_id, .. }) => {
                    if let Ok(mut queues) = self.services.queued_messages.lock() {
                        let queue = queues.entry(session_id.clone()).or_default();
                        if let Some(index) = queue
                            .iter()
                            .position(|queued| queued.queued_id == queued_id)
                        {
                            queue.remove(index);
                        }
                        queue.push_front(message);
                    }
                    break;
                }
                _ => {
                    if let Ok(mut queues) = self.services.queued_messages.lock() {
                        queues
                            .entry(session_id.clone())
                            .or_default()
                            .push_front(message);
                    }
                    break;
                }
            }
        }
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

    pub fn cancel_dormant_turn(
        &self,
        session_id: &str,
        turn_id: &str,
    ) -> Result<bool, BusinessError> {
        let Some(details) =
            db_blocking(|| self.services.store.cancel_dormant_turn(session_id, turn_id))?
        else {
            return Ok(false);
        };
        let tool_call_id = details.pending_tool_call_id.clone();
        let call_id = details.pending_call_id.clone();
        let tool_name = details.pending_tool_name.clone();
        self.services.emit(
            &details.session_id,
            EventPayload::TurnState(TurnStatePayload {
                turn_id: details.turn_id.clone(),
                state: "cancelled".into(),
                model_id: details.model_id.clone(),
                submission_idempotency_key: details.submission_idempotency_key.clone(),
                reason: Some("user_cancelled".into()),
            }),
        )?;
        match details.pending_kind.as_deref() {
            Some("approval") => {
                if let (Some(request_id), Some(tool_call_id), Some(tool_name)) = (
                    details.pending_request_id.as_deref(),
                    tool_call_id.as_deref(),
                    tool_name.as_deref(),
                ) {
                    self.services.emit(
                        &details.session_id,
                        EventPayload::ToolState(ToolStatePayload {
                            turn_id: details.turn_id.clone(),
                            call_id: call_id.clone(),
                            tool_call_id: tool_call_id.to_string(),
                            name: tool_name.to_string(),
                            state: "denied".into(),
                            reason: Some("user_cancelled".into()),
                        }),
                    )?;
                    self.services.emit(
                        &details.session_id,
                        EventPayload::ApprovalResolved(ApprovalResolvedPayload {
                            approval_id: request_id.to_string(),
                            turn_id: details.turn_id.clone(),
                            decision: "deny".into(),
                        }),
                    )?;
                }
            }
            Some("question") => {
                if let (Some(request_id), Some(tool_call_id)) = (
                    details.pending_request_id.as_deref(),
                    tool_call_id.as_deref(),
                ) {
                    self.services.emit(
                        &details.session_id,
                        EventPayload::QuestionRejected(QuestionAnsweredPayload {
                            request_id: request_id.to_string(),
                            turn_id: details.turn_id.clone(),
                            tool_call_id: tool_call_id.to_string(),
                            answers: Vec::new(),
                        }),
                    )?;
                }
            }
            _ => {}
        }
        self.clear_queued_messages(&details.session_id);
        self.active_turns
            .lock()
            .ok()
            .map(|mut values| values.remove(&details.session_id));
        self.cancellations
            .lock()
            .ok()
            .map(|mut values| values.remove(&details.turn_id));
        Ok(true)
    }

    pub(super) fn runtime_for(
        &self,
        project_id: &str,
        session_id: &str,
    ) -> Result<suncode_config::RuntimeConfig, BusinessError> {
        let persisted = self
            .services
            .store
            .persisted_settings(Some(project_id), Some(session_id))?;
        self.services
            .config
            .runtime(&persisted)
            .map_err(BusinessError::invalid)
    }
}
