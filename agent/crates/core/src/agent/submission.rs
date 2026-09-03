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
        if reasoning_effort.is_some_and(|value| !matches!(value, "low" | "medium" | "high")) {
            return Err(BusinessError::new(
                "invalid_arguments",
                "reasoning_effort must be low, medium, or high",
            ));
        }
        let session_lock = self.session_lock(session_id).await;
        let model = match model {
            Some(model) => model.to_string(),
            None => {
                let session = self
                    .store
                    .session_by_id(session_id)?
                    .ok_or_else(|| BusinessError::new("not_found", "session not found"))?;
                let project_id = session.project_id.ok_or_else(|| {
                    BusinessError::new("conflict", "session is not bound to a project")
                })?;
                self.store
                    .project_default_model(&project_id)?
                    .unwrap_or_else(|| "deepseek-v4-flash".into())
            }
        };
        let Some(provider) = self.providers.route(&model) else {
            return Err(BusinessError::new(
                "model_unavailable",
                "model is not advertised",
            ));
        };
        if reasoning_effort.is_some() && !self.providers.supports_reasoning_effort(&model) {
            return Err(BusinessError::new(
                "invalid_arguments",
                "selected model does not support reasoning effort",
            ));
        }
        let images = self.validate_message_images(session_id, &model, image_ids)?;
        let user_message = message_with_image_refs(input, &images);
        let _guard = match session_lock.try_lock() {
            Ok(guard) => guard,
            Err(_) => {
                let has_active_turn = self
                    .active_turns
                    .lock()
                    .map_err(|_| BusinessError::new("agent_unavailable", "turn state unavailable"))?
                    .contains_key(session_id);
                if has_active_turn {
                    if !image_ids.is_empty() {
                        return Err(BusinessError::new(
                            "conflict",
                            "image attachments cannot be queued while a turn is active",
                        ));
                    }
                    return self.queue_message(session_id, key, input, image_ids);
                }
                session_lock.lock().await
            }
        };
        let session = self
            .store
            .session_by_id(session_id)?
            .ok_or_else(|| BusinessError::new("not_found", "session not found"))?;
        let session_started_at = session.created_at.clone();
        let project_id = session
            .project_id
            .ok_or_else(|| BusinessError::new("conflict", "session is not bound to a project"))?;
        let project = self
            .store
            .project_by_id(&project_id)?
            .ok_or_else(|| BusinessError::new("not_found", "project not found"))?;
        let tool_call_limit = self
            .store
            .project_tool_call_limit(&project_id)?
            .unwrap_or(DEFAULT_TOOL_CALL_LIMIT);
        let admission = self
            .store
            .begin_turn_with_images(session_id, key, input, &model, image_ids)?;
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
        self.store.mark_turn_started(session_id, key)?;
        let token = CancellationToken::new();
        self.cancellations
            .lock()
            .map_err(|_| BusinessError::new("agent_unavailable", "cancellation state unavailable"))?
            .insert(admission.turn_id.clone(), token.clone());
        self.active_turns
            .lock()
            .map_err(|_| BusinessError::new("agent_unavailable", "turn state unavailable"))?
            .insert(session_id.to_string(), admission.turn_id.clone());
        let continuation = Continuation {
            session_id: session_id.into(),
            session_started_at,
            project_id,
            project_root: project.canonical_root,
            turn_id: admission.turn_id.clone(),
            submission_key: key.into(),
            model,
            reasoning_effort: reasoning_effort.map(str::to_owned),
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
            .run(continuation.clone(), Some(user_message), token, provider)
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

    /// Retry the most recently failed turn in a session using its persisted input.
    /// A new idempotency key is generated so the retry is admitted as a distinct turn.
    pub async fn retry_last_turn(&self, session_id: &str) -> Result<TurnResponse, BusinessError> {
        let Some((input, model, image_ids)) = self.store.latest_failed_turn_input(session_id)? else {
            return Err(BusinessError::new("conflict", "no failed turn to retry"));
        };
        if model.is_empty() {
            return Err(BusinessError::new("agent_unavailable", "failed turn has no model"));
        }
        self.submit_with_attachments(
            session_id,
            &Uuid::new_v4().to_string(),
            &input,
            Some(&model),
            None,
            &image_ids,
        )
        .await
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
        image_ids: &[String],
    ) -> Result<TurnResponse, BusinessError> {
        let active_turn_id = self
            .active_turns
            .lock()
            .map_err(|_| BusinessError::new("agent_unavailable", "turn state unavailable"))?
            .get(session_id)
            .cloned()
            .ok_or_else(|| BusinessError::new("conflict", "session is busy"))?;
        let mut queues = self
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
        let queued_id = Uuid::new_v4().to_string();
        queue.push_back(QueuedMessage {
            queued_id: queued_id.clone(),
            idempotency_key: key.to_string(),
            input: input.to_string(),
            image_ids: image_ids.to_vec(),
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
}
