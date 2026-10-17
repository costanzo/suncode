//! Model requests and compaction for [`TurnLoop`], plus the runtime's image
//! and message helpers; see `agent/turn_loop.rs`.

use super::support::*;
use super::*;

pub(super) fn provider_retry_allowed(error: &BusinessError) -> bool {
    error.retryable
        && matches!(error.code.as_str(), "transient" | "provider_protocol")
        && !is_context_overflow_error(error)
        && error.details["response_started"].as_bool() != Some(true)
}

pub(super) fn delegate_agent_definition() -> suncode_llm::ToolDefinition {
    suncode_llm::ToolDefinition {
        name: "delegate_agent".into(),
        description: "Delegate a bounded task to one fixed specialist agent. The specialist works in a linked child session and returns a result to the main agent.".into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "agent": {
                    "type": "string",
                    "enum": suncode_agents::specialists().iter().map(|agent| agent.name).collect::<Vec<_>>(),
                },
                "task": {"type": "string", "minLength": 1, "maxLength": 16000}
            },
            "required": ["agent", "task"],
            "additionalProperties": false
        }),
    }
}

pub(super) fn skill_guidance(
    project_root: &str,
    user_directory: Option<&std::path::Path>,
) -> Option<String> {
    let catalog = suncode_skills::SkillCatalog::discover(
        &suncode_skills::DiscoveryOptions::for_user(project_root, user_directory),
    );
    Some(suncode_skills::render_available_skills(
        catalog.visible().cloned(),
    ))
}

pub(super) enum TurnStart {
    /// A new submission whose user message is appended to the session.
    Submit(Message),
    /// A new turn that continues from the persisted context of the failed turn.
    Retry,
    /// An already admitted turn resuming after approval or a question.
    Resume,
}

impl AgentRuntime {
    pub(super) fn validate_message_images(
        &self,
        session_id: &str,
        model: &str,
        image_ids: &[String],
    ) -> Result<Vec<suncode_data::SessionImageRecord>, BusinessError> {
        if image_ids.len() > 3 {
            return Err(BusinessError::invalid(
                "a message can include at most three images",
            ));
        }
        if !image_ids.is_empty() && !self.providers.supports_vision(model) {
            return Err(BusinessError::new(
                "unsupported_capability",
                "selected model does not support image input",
            ));
        }
        let mut seen = std::collections::HashSet::new();
        let mut images = Vec::with_capacity(image_ids.len());
        for image_id in image_ids {
            if image_id.trim().is_empty() || !seen.insert(image_id.as_str()) {
                return Err(BusinessError::invalid(
                    "image IDs must be non-empty and unique",
                ));
            }
            let image = self
                .store
                .session_image_by_id(session_id, image_id)?
                .ok_or_else(|| {
                    BusinessError::new("not_found", "message image was not found in this session")
                })?;
            let size = fs::metadata(&image.storage_path)
                .map_err(|_| BusinessError::new("not_found", "message image file is unavailable"))?
                .len();
            if size == 0 || size > 20 * 1024 * 1024 {
                return Err(BusinessError::invalid(
                    "message image must be between 1 byte and 20 MiB",
                ));
            }
            images.push(image);
        }
        Ok(images)
    }

    pub(super) fn to_llm_message_with_images(
        &self,
        session_id: &str,
        message: &Message,
    ) -> Result<suncode_llm::Message, BusinessError> {
        let mut converted = message.clone();
        for part in &mut converted.content {
            if part.kind != "image_ref" {
                continue;
            }
            let image = self
                .store
                .session_image_by_id(session_id, &part.text)?
                .ok_or_else(|| BusinessError::new("not_found", "message image is unavailable"))?;
            let bytes = fs::read(&image.storage_path).map_err(|_| {
                BusinessError::new("not_found", "message image file is unavailable")
            })?;
            if bytes.is_empty() || bytes.len() > 20 * 1024 * 1024 {
                return Err(BusinessError::invalid(
                    "message image must be between 1 byte and 20 MiB",
                ));
            }
            part.kind = "image_url".into();
            part.text = format!(
                "data:{};base64,{}",
                image_mime_type(&image.storage_path)?,
                STANDARD.encode(bytes)
            );
        }
        Ok(converted)
    }
}

impl TurnLoop<'_> {
    /// Retries one immutable provider request, never the tools that produced it.
    pub(super) async fn complete_with_retries(
        &mut self,
        provider: &ModelRoute,
        started: Instant,
        llm_messages: &[suncode_llm::Message],
        tool_definitions: &[suncode_llm::ToolDefinition],
    ) -> Result<suncode_llm::Completion, BusinessError> {
        for attempt in 0..=2_u32 {
            if self.token.is_cancelled() {
                return Err(BusinessError::new("cancelled", "Turn was cancelled"));
            }
            if started.elapsed() >= Duration::from_millis(self.context.turn_timeout_ms) {
                return Err(BusinessError::new(
                    "turn_timeout",
                    "Turn exceeded its wall-clock budget",
                ));
            }
            let exchange_id = suncode_common::id::new(suncode_common::id::IdKind::Exchange);
            self.context.active_call_id = Some(exchange_id.clone());
            self.runtime.emit(
                &self.context.session_id,
                EventPayload::ProviderExchangeStarted(ProviderExchangeStartedPayload {
                    exchange_id: exchange_id.clone(),
                    turn_id: self.context.turn_id.clone(),
                    provider: provider.provider_id.clone(),
                    model_id: self.context.model.clone(),
                    wire_model: provider.wire_model.clone(),
                    iteration: self.context.iterations,
                }),
            )?;
            let result = {
                let mut response_started = false;
                let (delta_sender, mut delta_receiver) = mpsc::unbounded_channel();
                let (transfer_sender, mut transfer_receiver) = mpsc::unbounded_channel();
                let mut uploaded_bytes = 0_u64;
                let mut downloaded_bytes = 0_u64;
                let mut published_transfer = (0_u64, 0_u64);
                let mut transfer_tick = tokio::time::interval(Duration::from_millis(100));
                transfer_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                let provider_call = provider.provider.complete(
                    CompletionRequest {
                        messages: llm_messages,
                        wire_model: &provider.wire_model,
                        tools: tool_definitions,
                        client_toolsets: &[],
                        reasoning_effort: self.context.reasoning_effort.as_deref(),
                        max_output_tokens: self
                            .runtime
                            .providers
                            .limits(&self.context.model)
                            .and_then(|limits| limits.max_output_tokens),
                        transfer_progress: transfer_sender,
                    },
                    &self.token,
                    delta_sender,
                );
                tokio::pin!(provider_call);
                let turn_remaining = Duration::from_millis(self.context.turn_timeout_ms)
                    .saturating_sub(started.elapsed());
                let turn_deadline = tokio::time::sleep(turn_remaining);
                tokio::pin!(turn_deadline);
                let result = loop {
                    tokio::select! {
                        _ = self.token.cancelled() => break Err(BusinessError::new("cancelled", "Turn was cancelled")),
                        value = &mut provider_call => break value,
                        _ = &mut turn_deadline => break Err(BusinessError::new("turn_timeout", "Turn exceeded its wall-clock budget")),
                        Some(delta) = delta_receiver.recv() => {
                            response_started |= !delta.is_empty();
                            self.runtime.emit_live(&self.context.session_id, EventPayload::AssistantDelta(AssistantDeltaPayload { turn_id: self.context.turn_id.clone(), text: delta }));
                        }
                        Some(progress) = transfer_receiver.recv() => {
                            uploaded_bytes = uploaded_bytes.saturating_add(progress.uploaded_bytes);
                            downloaded_bytes = downloaded_bytes.saturating_add(progress.downloaded_bytes);
                        }
                        _ = transfer_tick.tick() => {
                            if published_transfer != (uploaded_bytes, downloaded_bytes) {
                                self.runtime.emit_live(&self.context.session_id, EventPayload::ProviderExchangeProgress(ProviderExchangeProgressPayload {
                                    exchange_id: exchange_id.clone(),
                                    turn_id: self.context.turn_id.clone(),
                                    provider: provider.provider_id.clone(),
                                    model_id: self.context.model.clone(),
                                    uploaded_bytes,
                                    downloaded_bytes,
                                }));
                                published_transfer = (uploaded_bytes, downloaded_bytes);
                            }
                        }
                    }
                };
                while let Ok(delta) = delta_receiver.try_recv() {
                    response_started |= !delta.is_empty();
                    self.runtime.emit_live(
                        &self.context.session_id,
                        EventPayload::AssistantDelta(AssistantDeltaPayload {
                            turn_id: self.context.turn_id.clone(),
                            text: delta,
                        }),
                    );
                }
                while let Ok(progress) = transfer_receiver.try_recv() {
                    uploaded_bytes = uploaded_bytes.saturating_add(progress.uploaded_bytes);
                    downloaded_bytes = downloaded_bytes.saturating_add(progress.downloaded_bytes);
                }
                if published_transfer != (uploaded_bytes, downloaded_bytes) {
                    self.runtime.emit_live(
                        &self.context.session_id,
                        EventPayload::ProviderExchangeProgress(ProviderExchangeProgressPayload {
                            exchange_id: exchange_id.clone(),
                            turn_id: self.context.turn_id.clone(),
                            provider: provider.provider_id.clone(),
                            model_id: self.context.model.clone(),
                            uploaded_bytes,
                            downloaded_bytes,
                        }),
                    );
                }
                result.map_err(|mut error| {
                    if response_started {
                        error.details["response_started"] = json!(true);
                    }
                    error
                })
            };
            let error = match result {
                Ok(completion) => return Ok(completion),
                Err(error) => error,
            };
            self.runtime.emit(
                &self.context.session_id,
                EventPayload::ProviderExchangeFailed(ProviderExchangeFailedPayload {
                    exchange_id: exchange_id.clone(),
                    turn_id: self.context.turn_id.clone(),
                    error: ProviderErrorPayload {
                        code: error.code.clone(),
                        message: error.message.clone(),
                        retryable: error.retryable,
                    },
                    provider_request_id: error.provider_request_id.clone(),
                }),
            )?;
            if attempt == 2 || !provider_retry_allowed(&error) {
                return Err(error);
            }
            let jitter = suncode_common::random::below(251);
            let delay_ms = (1000_u64 << attempt)
                .saturating_add(jitter)
                .max(error.details["retry_after_ms"].as_u64().unwrap_or(0));
            let remaining = Duration::from_millis(self.context.turn_timeout_ms)
                .saturating_sub(started.elapsed());
            if Duration::from_millis(delay_ms) >= remaining {
                return Err(BusinessError::new(
                    "turn_timeout",
                    "Provider retry cannot fit within the turn's remaining budget",
                ));
            }
            self.runtime.emit_live(
                &self.context.session_id,
                EventPayload::ProviderRetryScheduled(ProviderRetryScheduledPayload {
                    exchange_id,
                    turn_id: self.context.turn_id.clone(),
                    retry_attempt: attempt + 1,
                    max_retries: 2,
                    delay_ms,
                    error_code: error.code.clone(),
                }),
            );
            tokio::select! {
                biased;
                _ = self.token.cancelled() => return Err(BusinessError::new("cancelled", "Turn was cancelled")),
                _ = tokio::time::sleep(remaining) => return Err(BusinessError::new("turn_timeout", "Turn exceeded its wall-clock budget")),
                _ = tokio::time::sleep(Duration::from_millis(delay_ms)) => {}
            }
        }
        unreachable!("provider attempts are bounded")
    }

    pub(super) async fn run(
        &mut self,
        start: TurnStart,
        provider: ModelRoute,
    ) -> Result<TurnResponse, BusinessError> {
        let started = Instant::now();
        match start {
            TurnStart::Submit(message) => {
                self.runtime
                    .turn_state(self.context, TurnState::Admitted, None)?;
                self.context.messages.push(message.clone());
                self.runtime.emit(
                    &self.context.session_id,
                    EventPayload::MessageUser(MessageUserPayload {
                        message_id: suncode_common::id::new(suncode_common::id::IdKind::Message),
                        turn_id: self.context.turn_id.clone(),
                        queued_id: None,
                        queued_idempotency_key: None,
                        message,
                    }),
                )?;
            }
            TurnStart::Retry => self
                .runtime
                .turn_state(self.context, TurnState::Admitted, None)?,
            TurnStart::Resume => {}
        }
        self.runtime
            .turn_state(self.context, TurnState::Preparing, None)?;
        let base_system_messages = system_prompt::build_messages(system_prompt::PromptContext {
            model_id: &self.context.model,
            provider_id: &provider.provider_id,
            project_root: &self.context.project_root,
            session_started_at: &self.context.session_started_at,
            non_interactive: self.runtime.non_interactive,
            host_capabilities: self.runtime.host_capabilities,
            allowed_tools: &self.context.allowed_tools,
            agent: self.context.agent(),
            dependency_context: self
                .runtime
                .dependency_context_message(&self.context.project_id)?,
            skill_guidance: if self.context.agent().can_use_skills {
                skill_guidance(
                    &self.context.project_root,
                    Some(&self.runtime.config.user_directory),
                )
            } else {
                None
            },
        })?;
        while self.context.iterations < 1024 {
            if self.token.is_cancelled() {
                return self
                    .runtime
                    .fail_context(self.context, "cancelled", "Turn was cancelled");
            }
            if started.elapsed() > Duration::from_millis(self.context.turn_timeout_ms) {
                return self.runtime.fail_context(
                    self.context,
                    "turn_timeout",
                    "Turn exceeded its wall-clock budget",
                );
            }
            self.drain_queued_messages()?;
            self.context.iterations += 1;
            self.runtime
                .turn_state(self.context, TurnState::CallingModel, None)?;
            let agent = self.context.agent();
            let mut tool_definitions = suncode_tool::definitions::all();
            if !agent.can_ask_user {
                tool_definitions.retain(|definition| definition.name != "question");
            }
            if agent.can_delegate {
                tool_definitions.push(delegate_agent_definition());
            }
            if agent.can_use_external_tools {
                // The static set already lists the built-in tools; add only
                // tools that providers contribute beyond it.
                let catalog = suncode_common::tool::CatalogContext {
                    project_id: &self.context.project_id,
                    computer_use: self
                        .runtime
                        .providers
                        .supports_computer_use(&self.context.model),
                };
                let provider_tools = self
                    .runtime
                    .tool_providers
                    .catalog(&catalog)
                    .await
                    .into_iter()
                    .filter(|tool| {
                        !tool_definitions
                            .iter()
                            .any(|existing| existing.name == tool.name)
                    })
                    .collect::<Vec<_>>();
                logging::debug(
                    "agent.tools",
                    format!(
                        "provider_catalog session={} project={} count={}",
                        self.context.session_id,
                        self.context.project_id,
                        provider_tools.len()
                    ),
                );
                tool_definitions.extend(provider_tools);
            }
            // The agent's tool scope, snapshotted at admission; empty means
            // the full catalog.
            if !self.context.allowed_tools.is_empty() {
                tool_definitions.retain(|definition| {
                    self.context
                        .allowed_tools
                        .iter()
                        .any(|allowed| allowed == &definition.name)
                });
            }
            if !self.context.host_tool_allowlist.is_empty() {
                tool_definitions.retain(|definition| {
                    self.context
                        .host_tool_allowlist
                        .iter()
                        .any(|allowed| allowed == &definition.name)
                });
            }
            let fixed_request_tokens =
                serde_json::to_string(&(&base_system_messages, &tool_definitions))
                    .map(|value| value.len().div_ceil(4))
                    .unwrap_or(0)
                    .saturating_add(
                        self.runtime
                            .providers
                            .limits(&self.context.model)
                            .and_then(|limits| limits.max_output_tokens)
                            .and_then(|value| usize::try_from(value).ok())
                            .unwrap_or(0),
                    );
            let mut prompt = context::build_for_model_with_overhead(
                &self.context.messages,
                self.runtime
                    .providers
                    .limits(&self.context.model)
                    .and_then(|limits| limits.max_input_tokens),
                self.runtime
                    .providers
                    .limits(&self.context.model)
                    .and_then(|limits| limits.auto_compact_tokens),
                fixed_request_tokens,
            );
            if prompt.compacted && prompt.dropped_messages > 0 {
                self.generate_compaction_summary(&mut prompt, &provider)
                    .await?;
            }
            if prompt.retained_tokens > prompt.max_retained_tokens {
                return self.runtime.fail_context(
                    self.context,
                    "context_budget_exceeded",
                    "Compacted context still exceeds the selected model's request budget",
                );
            }
            if prompt.compacted {
                self.runtime.emit_context_compacted(self.context, &prompt)?;
            }
            if prompt.compacted {
                self.context.messages = prompt.messages;
            }
            let mut llm_messages = base_system_messages.clone();
            llm_messages.extend(
                self.context
                    .messages
                    .iter()
                    .map(|message| {
                        self.runtime
                            .to_llm_message_with_images(&self.context.session_id, message)
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            );
            let result = self
                .complete_with_retries(&provider, started, &llm_messages, &tool_definitions)
                .await;
            let result = match result {
                Ok(result) => result,
                Err(error) => {
                    if !self.context.overflow_recovery_attempted
                        && is_context_overflow_error(&error)
                        && error.details["response_started"].as_bool() != Some(true)
                    {
                        self.context.overflow_recovery_attempted = true;
                        let mut forced = context::force_compact(
                            &self.context.messages,
                            self.runtime
                                .providers
                                .limits(&self.context.model)
                                .and_then(|limits| limits.max_input_tokens)
                                .and_then(|value| usize::try_from(value).ok())
                                .unwrap_or(context::DEFAULT_CONTEXT_WINDOW_TOKENS),
                        );
                        if forced.compacted {
                            self.generate_compaction_summary(&mut forced, &provider)
                                .await?;
                            self.context.messages = forced.messages.clone();
                            self.runtime.emit_context_compacted(self.context, &forced)?;
                            continue;
                        }
                    }
                    return Err(error);
                }
            };
            let exchange_id = self
                .context
                .active_call_id
                .clone()
                .expect("successful provider attempt has an exchange ID");
            if result.text.len() > 8 * 1024 * 1024 {
                return self.runtime.fail_context(
                    self.context,
                    "output_budget_exceeded",
                    "Provider output exceeded its budget",
                );
            }
            let usage = result.usage.as_ref().map(Usage::without_breakdown);
            let tool_calls = result.tool_calls.clone();
            if let Some(usage) = &usage {
                self.context.usage.add(usage);
                self.runtime.emit(
                    &self.context.session_id,
                    EventPayload::UsageUpdated(UsageUpdatedPayload {
                        turn_id: self.context.turn_id.clone(),
                        usage: self.context.usage.clone(),
                    }),
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
            self.runtime.emit(
                &self.context.session_id,
                EventPayload::ProviderExchangeCompleted(ProviderExchangeCompletedPayload {
                    exchange_id: exchange_id.clone(),
                    turn_id: self.context.turn_id.clone(),
                    output_message: assistant.clone(),
                    tool_calls: tool_calls.clone(),
                    usage: result.usage.clone(),
                    provider_request_id: result.provider_request_id.clone(),
                    provider_response_id: result.provider_response_id.clone(),
                    finish_reason: result.finish_reason.clone(),
                }),
            )?;
            self.context.messages.push(assistant.clone());
            self.runtime.emit(
                &self.context.session_id,
                EventPayload::MessageAssistant(MessageAssistantPayload {
                    message_id: suncode_common::id::new(suncode_common::id::IdKind::Message),
                    turn_id: self.context.turn_id.clone(),
                    call_id: self.context.active_call_id.clone(),
                    message: assistant.clone(),
                    usage: self.context.usage.clone(),
                    finish_reason: result.finish_reason.clone(),
                }),
            )?;
            if tool_calls.is_empty() {
                if self.drain_queued_messages()? {
                    self.runtime
                        .turn_state(self.context, TurnState::Preparing, None)?;
                    continue;
                }
                self.runtime
                    .turn_state(self.context, TurnState::Completed, None)?;
                self.runtime.emit(
                    &self.context.session_id,
                    EventPayload::TurnCompleted(TurnCompletedPayload {
                        turn_id: self.context.turn_id.clone(),
                        usage: self.context.usage.clone(),
                        iterations: self.context.iterations,
                        tool_calls: self.context.tool_calls,
                    }),
                )?;
                let response = TurnResponse::Completed {
                    turn_id: self.context.turn_id.clone(),
                    message: assistant,
                    usage: self.context.usage.clone(),
                    iterations: self.context.iterations,
                    tool_calls: self.context.tool_calls,
                };
                db_blocking(|| {
                    self.runtime.store.complete_turn(
                        &self.context.session_id,
                        &self.context.submission_key,
                        &serde_json::to_value(&response).map_err(|_| {
                            BusinessError::new(
                                "agent_unavailable",
                                "turn response could not be stored",
                            )
                        })?,
                    )
                })?;
                return Ok(response);
            }
            self.runtime
                .turn_state(self.context, TurnState::ResolvingCalls, None)?;
            self.context.approval_batch_failed = false;
            self.resolve_calls(tool_calls).await?;
            self.drain_queued_messages()?;
            self.runtime
                .turn_state(self.context, TurnState::Preparing, None)?;
        }
        self.runtime.fail_context(
            self.context,
            "iteration_budget_exceeded",
            "Turn exceeded its iteration budget",
        )
    }

    pub(super) fn drain_queued_messages(&mut self) -> Result<bool, BusinessError> {
        let queued = {
            let mut queues =
                self.runtime.queued_messages.lock().map_err(|_| {
                    BusinessError::new("agent_unavailable", "turn queue unavailable")
                })?;
            queues.remove(&self.context.session_id).unwrap_or_default()
        };
        if queued.is_empty() {
            return Ok(false);
        }
        for item in queued {
            let images = self.runtime.validate_message_images(
                &self.context.session_id,
                &self.context.model,
                &item.image_ids,
            )?;
            let message = message_with_image_refs(&item.input, &images);
            self.context.messages.push(message.clone());
            self.runtime.emit(
                &self.context.session_id,
                EventPayload::MessageUser(MessageUserPayload {
                    message_id: suncode_common::id::new(suncode_common::id::IdKind::Message),
                    turn_id: self.context.turn_id.clone(),
                    queued_id: Some(item.queued_id),
                    queued_idempotency_key: Some(item.idempotency_key),
                    message,
                }),
            )?;
        }
        Ok(true)
    }

    pub(super) async fn generate_compaction_summary(
        &mut self,
        result: &mut context::ContextBuildResult,
        provider: &ModelRoute,
    ) -> Result<(), BusinessError> {
        if result.dropped_messages == 0 || self.token.is_cancelled() {
            return Ok(());
        }
        let dropped =
            &self.context.messages[..result.dropped_messages.min(self.context.messages.len())];
        let mut excerpts = Vec::new();
        let mut remaining = 48_000usize;
        for message in dropped.iter().rev() {
            if remaining == 0 {
                break;
            }
            let content = message.text_content();
            let calls = serde_json::to_string(&message.tool_calls).unwrap_or_default();
            let line = format!(
                "[{}] {} {}",
                message.role,
                content.chars().take(2_000).collect::<String>(),
                calls.chars().take(1_000).collect::<String>()
            );
            let excerpt = line.chars().take(remaining).collect::<String>();
            remaining = remaining.saturating_sub(excerpt.chars().count());
            excerpts.push(excerpt);
        }
        excerpts.reverse();
        if let Some(previous) = dropped.first().filter(|message| {
            message.role == "system" && message.text_content().contains("suncode_context_summary")
        }) {
            let text = previous.text_content();
            excerpts.insert(
                0,
                format!(
                    "[Previous summary] {}",
                    text.chars().take(4_000).collect::<String>()
                ),
            );
        }
        let system = suncode_llm::Message::text("system", suncode_agents::prompt::COMPACTION);
        let user = suncode_llm::Message::text("user", excerpts.join("\n\n"));
        let messages = [system, user];
        let (delta_sender, _delta_receiver) = mpsc::unbounded_channel();
        let (transfer_sender, _transfer_receiver) = mpsc::unbounded_channel();
        let request = CompletionRequest {
            messages: &messages,
            wire_model: &provider.wire_model,
            tools: &[],
            client_toolsets: &[],
            reasoning_effort: None,
            max_output_tokens: Some(2_048),
            transfer_progress: transfer_sender,
        };
        let exchange_id = suncode_common::id::new(suncode_common::id::IdKind::Exchange);
        self.runtime.emit(
            &self.context.session_id,
            EventPayload::ProviderExchangeStarted(ProviderExchangeStartedPayload {
                exchange_id: exchange_id.clone(),
                turn_id: self.context.turn_id.clone(),
                provider: provider.provider_id.clone(),
                model_id: self.context.model.clone(),
                wire_model: provider.wire_model.clone(),
                iteration: self.context.iterations,
            }),
        )?;
        let completion = tokio::time::timeout(
            Duration::from_secs(30),
            provider
                .provider
                .complete(request, &self.token, delta_sender),
        )
        .await;
        let completion = match completion {
            Ok(Ok(completion)) => completion,
            outcome => {
                let (code, message, retryable, provider_request_id) = match outcome {
                    Ok(Err(error)) => (
                        error.code,
                        error.message,
                        error.retryable,
                        error.provider_request_id,
                    ),
                    Err(_) => (
                        "compaction_timeout".into(),
                        "Summary generation timed out".into(),
                        false,
                        None,
                    ),
                    Ok(Ok(_)) => unreachable!(),
                };
                self.runtime.emit(
                    &self.context.session_id,
                    EventPayload::ProviderExchangeFailed(ProviderExchangeFailedPayload {
                        exchange_id,
                        turn_id: self.context.turn_id.clone(),
                        error: ProviderErrorPayload {
                            code,
                            message,
                            retryable,
                        },
                        provider_request_id,
                    }),
                )?;
                return Ok(());
            }
        };
        self.runtime.emit(
            &self.context.session_id,
            EventPayload::ProviderExchangeCompleted(ProviderExchangeCompletedPayload {
                exchange_id,
                turn_id: self.context.turn_id.clone(),
                output_message: Message::text("assistant", completion.text.clone()),
                tool_calls: Vec::new(),
                usage: completion.usage,
                provider_request_id: completion.provider_request_id,
                provider_response_id: completion.provider_response_id,
                finish_reason: completion.finish_reason.clone(),
            }),
        )?;
        if completion.finish_reason != "stop" && completion.finish_reason != "end_turn" {
            return Ok(());
        }
        let Ok(summary) = serde_json::from_str::<context::ContextSummary>(&completion.text) else {
            return Ok(());
        };
        if summary.objective.trim().is_empty() || summary.next_action.trim().is_empty() {
            return Ok(());
        }
        context::apply_generated_summary(result, summary);
        Ok(())
    }
}
