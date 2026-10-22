//! What a turn needs from the runtime: persistence, model providers, audited
//! operations, tool providers, events, and the host's settings.
//!
//! [`AgentRuntime`] owns one `TurnServices` alongside its session
//! bookkeeping and the runtimes it starts and stops. A [`TurnLoop`] sees only
//! these services and the [`ChildTurns`] it may delegate to, never the
//! runtime itself.

use super::providers::{EventOutput, OperationsDocumentReader, StorePathScopes, TokenCancellation};
use super::support::*;
use super::*;
use suncode_common::tool::{ToolContext, ToolInvocation, ToolOutput, ToolProvider};

#[derive(Clone)]
pub(super) struct TurnServices {
    pub(super) store: Store,
    pub(super) user_id: String,
    pub(super) providers: Arc<ModelProviderRegistry>,
    pub(super) operations: Arc<suncode_tool::Operations>,
    pub(super) events: SessionEventHub,
    pub(super) attention_events: AttentionEventHub,
    pub(super) queued_messages: Arc<Mutex<HashMap<String, VecDeque<QueuedMessage>>>>,
    pub(super) tool_providers: providers::ToolProviders,
    pub(super) non_interactive: bool,
    pub(super) host_capabilities: AgentHostCapabilities,
    pub(super) config: Config,
}

/// Starts a delegated child turn. Implemented by the runtime, which admits
/// the child session's turn like any other submission.
pub(super) trait ChildTurns: Send + Sync {
    fn submit_child<'a>(
        &'a self,
        session_id: &'a str,
        input: &'a str,
        model: &'a str,
        reasoning_effort: Option<&'a str>,
        parent_token: CancellationToken,
    ) -> futures_util::future::BoxFuture<'a, Result<TurnResponse, BusinessError>>;
}

impl ChildTurns for AgentRuntime {
    fn submit_child<'a>(
        &'a self,
        session_id: &'a str,
        input: &'a str,
        model: &'a str,
        reasoning_effort: Option<&'a str>,
        parent_token: CancellationToken,
    ) -> futures_util::future::BoxFuture<'a, Result<TurnResponse, BusinessError>> {
        Box::pin(AgentRuntime::submit_child(
            self,
            session_id,
            input,
            model,
            reasoning_effort,
            parent_token,
        ))
    }
}

impl TurnServices {
    pub(super) fn emit(&self, session_id: &str, event: EventPayload) -> Result<(), BusinessError> {
        let attention_payload = event.clone();
        let occurred_at = self.events.publish_projected(session_id, event, |event| {
            let event_type = event.event_type();
            let payload = event.clone().into_value();
            let projected = db_blocking(|| {
                self.store
                    .append_content(session_id, event_type.as_str(), &payload)
            })?;
            Ok::<String, BusinessError>(projected.occurred_at)
        })?;
        self.publish_attention(session_id, &occurred_at, &attention_payload);
        Ok(())
    }

    pub(super) fn emit_context_compacted(
        &self,
        context: &Continuation,
        result: &context::ContextBuildResult,
    ) -> Result<(), BusinessError> {
        let event = EventPayload::ContextCompacted(ContextCompactedPayload {
            exchange_id: suncode_common::id::new(suncode_common::id::IdKind::Exchange),
            turn_id: context.turn_id.clone(),
            provider: "SunCode".into(),
            model_id: "context-compaction".into(),
            wire_model: "internal".into(),
            iteration: context.iterations,
            started_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            original_characters: result.original_characters,
            retained_characters: result.retained_characters,
            original_tokens: result.original_tokens,
            retained_tokens: result.retained_tokens,
            dropped_messages: result.dropped_messages,
            summary: result
                .summary
                .as_ref()
                .map(|summary| ContextSummaryPayload {
                    objective: summary.objective.clone(),
                    important_constraints: summary.important_constraints.clone(),
                    completed_work: summary.completed_work.clone(),
                    active_work: summary.active_work.clone(),
                    blockers: summary.blockers.clone(),
                    next_action: summary.next_action.clone(),
                }),
        });
        self.events
            .publish_projected(&context.session_id, event, |event| {
                let mut payload = event.clone().into_value();
                payload["retained_messages"] =
                    serde_json::to_value(context::persistable_messages(&result.messages))?;
                let projected = self.store.append_content(
                    &context.session_id,
                    "context.compacted",
                    &payload,
                )?;
                Ok::<String, BusinessError>(projected.occurred_at)
            })?;
        Ok(())
    }

    pub(super) fn publish_attention(
        &self,
        session_id: &str,
        occurred_at: &str,
        event: &EventPayload,
    ) {
        let (kind, turn_id, correlation_id) = match event {
            EventPayload::TurnCompleted(payload) => (
                AttentionKind::PrimaryTurnCompleted,
                payload.turn_id.as_str(),
                payload.turn_id.as_str(),
            ),
            EventPayload::TurnState(payload) if payload.state == "failed" => (
                AttentionKind::PrimaryTurnFailed,
                payload.turn_id.as_str(),
                payload.turn_id.as_str(),
            ),
            EventPayload::ApprovalRequested(payload) => (
                AttentionKind::ApprovalRequested,
                payload.turn_id.as_str(),
                payload.approval_id.as_str(),
            ),
            EventPayload::QuestionAsked(payload) => (
                AttentionKind::QuestionAsked,
                payload.turn_id.as_str(),
                payload.request_id.as_str(),
            ),
            _ => return,
        };
        let session = match self.store.session_by_id(session_id) {
            Ok(Some(session)) => session,
            Ok(None) => return,
            Err(error) => {
                logging::write_business_error(
                    "attention",
                    "resolve_session",
                    &error,
                    "delivery=skipped",
                );
                return;
            }
        };
        if matches!(
            kind,
            AttentionKind::PrimaryTurnCompleted
                | AttentionKind::PrimaryTurnFailed
                | AttentionKind::QuestionAsked
        ) && session.kind != "primary"
        {
            return;
        }
        let Some(project_id) = session.project_id.clone() else {
            return;
        };
        let project = match self
            .store
            .project_by_id_for_user(&self.user_id, &project_id)
        {
            Ok(Some(project)) => project,
            Ok(None) => return,
            Err(error) => {
                logging::write_business_error(
                    "attention",
                    "resolve_project",
                    &error,
                    "delivery=skipped",
                );
                return;
            }
        };
        self.attention_events.publish(AgentAttentionEvent {
            kind,
            correlation_id: correlation_id.to_string(),
            project_id,
            project_display_name: project.display_name,
            session_id: session.session_id,
            session_title: session.title.unwrap_or_default(),
            session_kind: session.kind,
            parent_session_id: session.parent_session_id,
            turn_id: turn_id.to_string(),
            occurred_at: occurred_at.to_string(),
        });
    }

    pub(super) fn emit_live(&self, session_id: &str, event: EventPayload) {
        let event = AgentEvent {
            session_id: session_id.to_string(),
            occurred_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            payload: event,
        };
        self.events.publish(event);
    }

    pub(super) fn turn_state(
        &self,
        context: &Continuation,
        state: TurnState,
        reason: Option<&str>,
    ) -> Result<(), BusinessError> {
        self.emit(
            &context.session_id,
            EventPayload::TurnState(TurnStatePayload {
                turn_id: context.turn_id.clone(),
                state: state.as_str().to_owned(),
                model_id: Some(context.model.clone()),
                submission_idempotency_key: Some(context.submission_key.clone()),
                reason: reason.map(str::to_owned),
            }),
        )
    }

    pub(super) fn tool_state(
        &self,
        context: &Continuation,
        call: &ToolCall,
        state: ToolState,
        reason: Option<&str>,
    ) -> Result<(), BusinessError> {
        self.emit(
            &context.session_id,
            EventPayload::ToolState(ToolStatePayload {
                turn_id: context.turn_id.clone(),
                call_id: context.active_call_id.clone(),
                tool_call_id: call.call_id.clone(),
                name: call.name.clone(),
                state: state.as_str().to_owned(),
                reason: reason.map(str::to_owned),
            }),
        )
    }

    pub(super) fn fail_context<T>(
        &self,
        context: &Continuation,
        code: &str,
        message: &str,
    ) -> Result<T, BusinessError> {
        self.turn_state(context, states::failed_turn_state(code), Some(code))?;
        Err(BusinessError::new(code, message))
    }

    /// Runs `call` through `provider` with this turn's context.
    pub(super) async fn execute_provider(
        &self,
        provider: &dyn ToolProvider,
        context: &Continuation,
        call: &ToolCall,
        token: CancellationToken,
        approval_pin: Option<Value>,
    ) -> Result<suncode_common::tool::ToolResult, BusinessError> {
        let call_key = format!("{}:{}", context.turn_id, call.call_id);
        let project_root = std::path::Path::new(&context.project_root);
        let paths = StorePathScopes {
            store: &self.store,
            project_id: &context.project_id,
            project_root,
        };
        let documents = OperationsDocumentReader {
            services: self,
            paths: &paths,
            token: token.clone(),
        };
        let output: Arc<dyn ToolOutput> = Arc::new(EventOutput {
            services: self.clone(),
            session_id: context.session_id.clone(),
            turn_id: context.turn_id.clone(),
            call_id: context.active_call_id.clone(),
            tool_call_id: call.call_id.clone(),
        });
        let cancellation = TokenCancellation(token);
        provider
            .execute(
                ToolInvocation {
                    name: &tools::tool_name(call),
                    arguments: call.arguments.clone(),
                    approval_pin,
                },
                ToolContext {
                    project_id: &context.project_id,
                    project_root,
                    call_key: &call_key,
                    timeout_ceiling: context.bash_timeout_ms.map(Duration::from_millis),
                    paths: &paths,
                    documents: &documents,
                    output,
                    cancellation: &cancellation,
                },
            )
            .await
    }

    pub(super) async fn operation_in_project(
        &self,
        project_root: &str,
        method: &str,
        params: Value,
        token: CancellationToken,
    ) -> Result<Value, BusinessError> {
        let operations = self.operations.clone();
        let root = std::path::PathBuf::from(project_root);
        let method = method.to_string();
        let cancelled = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let cancelled_for_operation = cancelled.clone();
        let mut operation = Box::pin(tokio::task::spawn_blocking(move || {
            operations.execute_in_project_with_cancellation_and_output(
                &root,
                &method,
                params,
                Some(&cancelled_for_operation),
                None,
            )
        }));
        let join_result = tokio::select! {
            result = &mut operation => result,
            _ = token.cancelled() => {
                cancelled.store(true, std::sync::atomic::Ordering::Relaxed);
                (&mut operation).await
            }
        };
        join_result.map_err(|_| BusinessError::new("agent_unavailable", "operation task failed"))?
    }

    pub(super) fn dependency_context_message(
        &self,
        project_id: &str,
    ) -> Result<Option<suncode_llm::Message>, BusinessError> {
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

    pub(super) fn validate_message_images(
        &self,
        session_id: &str,
        model: &str,
        image_ids: &[String],
        require_vision: bool,
    ) -> Result<Vec<suncode_data::SessionImageRecord>, BusinessError> {
        if image_ids.len() > 3 {
            return Err(BusinessError::invalid(
                "a message can include at most three images",
            ));
        }
        if require_vision && !image_ids.is_empty() && !self.providers.supports_vision(model) {
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
