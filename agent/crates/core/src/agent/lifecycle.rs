impl Agent {
    fn emit(
        &self,
        session_id: &str,
        event: EventPayload,
    ) -> Result<(), BusinessError> {
        let event_type = event.event_type();
        let payload = event.into_value();
        let event = self
            .store
            .append_content(session_id, event_type.as_str(), &payload)?;
        let _ = self.events.send(event);
        Ok(())
    }

    fn emit_live(&self, session_id: &str, event: EventPayload) {
        let event_type = event.event_type();
        let event = SessionEvent {
            session_id: session_id.to_string(),
            occurred_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            event_type: event_type.as_str().to_string(),
            payload: event.into_value(),
        };
        let _ = self.events.send(event);
    }
    fn turn_state(
        &self,
        context: &Continuation,
        state: &str,
        reason: Option<&str>,
    ) -> Result<(), BusinessError> {
        self.emit(&context.session_id, EventPayload::TurnState(TurnStatePayload { turn_id: context.turn_id.clone(), state: state.to_owned(), model_id: Some(context.model.clone()), submission_idempotency_key: Some(context.submission_key.clone()), reason: reason.map(str::to_owned) }))
    }
    fn tool_state(
        &self,
        context: &Continuation,
        call: &ToolCall,
        state: &str,
        reason: Option<&str>,
    ) -> Result<(), BusinessError> {
        self.emit(&context.session_id, EventPayload::ToolState(ToolStatePayload { turn_id: context.turn_id.clone(), call_id: context.active_call_id.clone(), tool_call_id: call.call_id.clone(), name: call.name.clone(), state: state.to_owned(), reason: reason.map(str::to_owned) }))
    }
    fn fail_context<T>(
        &self,
        context: &Continuation,
        code: &str,
        message: &str,
    ) -> Result<T, BusinessError> {
        self.turn_state(
            context,
            if code == "cancelled" {
                "cancelled"
            } else {
                "failed"
            },
            Some(code),
        )?;
        Err(BusinessError::new(code, message))
    }

    async fn session_lock(&self, session_id: &str) -> Arc<AsyncMutex<()>> {
        let mut locks = self.session_locks.lock().await;
        locks
            .entry(session_id.to_string())
            .or_insert_with(|| Arc::new(AsyncMutex::new(())))
            .clone()
    }

    pub async fn recover(&self) -> Result<(), BusinessError> {
        for event in self.store.recover_startup()? {
            let _ = self.events.send(event);
        }
        for suspended in self.store.resuming_turns()? {
            let mut continuation: Continuation = serde_json::from_value(suspended.snapshot)
                .map_err(|_| {
                    BusinessError::new("agent_unavailable", "approval continuation is invalid")
                })?;
            let token = CancellationToken::new();
            self.cancellations
                .lock()
                .map_err(|_| {
                    BusinessError::new("agent_unavailable", "cancellation state unavailable")
                })?
                .insert(continuation.turn_id.clone(), token.clone());
            self.active_turns
                .lock()
                .map_err(|_| BusinessError::new("agent_unavailable", "turn state unavailable"))?
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
