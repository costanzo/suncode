//! Part of the `AgentRuntime` implementation; see `agent/mod.rs`.

use super::*;

impl AgentRuntime {
    pub(super) fn ensure_running(&self) -> Result<(), BusinessError> {
        if self
            .shutting_down
            .load(std::sync::atomic::Ordering::Acquire)
        {
            Err(BusinessError::new(
                "agent_shutting_down",
                "agent shutdown is in progress",
            ))
        } else {
            Ok(())
        }
    }

    pub async fn shutdown(&self) -> Result<(), BusinessError> {
        if self
            .shutting_down
            .swap(true, std::sync::atomic::Ordering::AcqRel)
        {
            return Ok(());
        }

        let (tokens, cancellation_state_available) = match self.cancellations.lock() {
            Ok(cancellations) => (cancellations.values().cloned().collect::<Vec<_>>(), true),
            Err(_) => (Vec::new(), false),
        };
        for token in tokens {
            token.cancel();
        }
        if let Ok(mut queued) = self.services.queued_messages.lock() {
            queued.clear();
        }

        let computer_result = self.computer.emergency_stop();
        tokio::join!(
            self.browser.shutdown(),
            self.mcp.shutdown(),
            self.lsp.shutdown()
        );
        self.preview.shutdown();

        let turns_stopped = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let active = self
                    .active_turns
                    .lock()
                    .map(|turns| turns.is_empty())
                    .unwrap_or(false);
                if active {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .is_ok();

        self.services.events.close_all();
        self.services.attention_events.close_all();

        computer_result?;
        if !cancellation_state_available {
            return Err(BusinessError::unavailable(
                "cancellation state was unavailable during shutdown",
            ));
        }
        if !turns_stopped {
            return Err(BusinessError::unavailable(
                "active turns did not stop before the shutdown deadline",
            ));
        }
        Ok(())
    }

    pub fn attention_event_hub(&self) -> AttentionEventHub {
        self.services.attention_events.clone()
    }

    pub(super) async fn session_lock(&self, session_id: &str) -> Arc<AsyncMutex<()>> {
        let mut locks = self.session_locks.lock().await;
        locks
            .entry(session_id.to_string())
            .or_insert_with(|| Arc::new(AsyncMutex::new(())))
            .clone()
    }

    pub async fn recover(&self) -> Result<(), BusinessError> {
        for event in db_blocking(|| self.services.store.recover_startup())? {
            let payload = EventPayload::TurnState(TurnStatePayload {
                turn_id: event.payload["turn_id"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                state: event.payload["state"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                model_id: event.payload["model_id"].as_str().map(str::to_string),
                submission_idempotency_key: event.payload["submission_idempotency_key"]
                    .as_str()
                    .map(str::to_string),
                reason: event.payload["reason"].as_str().map(str::to_string),
            });
            self.services.events.publish(AgentEvent {
                session_id: event.session_id,
                occurred_at: event.occurred_at,
                payload,
            });
        }
        for suspended in db_blocking(|| self.services.store.resuming_turns())? {
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
                    TurnLoop::new(&agent, &mut continuation, token)
                        .continue_question()
                        .await
                } else {
                    TurnLoop::new(&agent, &mut continuation, token)
                        .continue_approved()
                        .await
                };
                let suspended_again = result
                    .as_ref()
                    .err()
                    .is_some_and(|error| states::suspends_turn(&error.code));
                if let Err(error) = agent.persist_child_invocation_outcome(&continuation, &result) {
                    logging::write_business_error(
                        "subagent",
                        "persist_recovery_outcome",
                        &error,
                        format!(
                            "session={} turn={}",
                            continuation.session_id, continuation.turn_id
                        ),
                    );
                }
                let _ = db_blocking(|| {
                    agent.services.store.finish_suspended(
                        &approval_id,
                        if result.is_ok() || suspended_again {
                            "completed"
                        } else {
                            "failed"
                        },
                    )
                });
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
