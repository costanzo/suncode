use super::*;
use std::os::raw::c_void;
use suncode_agent::{agent::TurnResponse, domain::ApprovalRecord};

impl AgentSdk {
    pub fn submit_turn(
        &self,
        session_id: &str,
        input: &str,
        idempotency_key: &str,
        model: Option<&str>,
        reasoning_effort: Option<&str>,
    ) -> SdkResult<TurnResponse> {
        self.submit_turn_with_attachments(
            session_id,
            input,
            idempotency_key,
            model,
            reasoning_effort,
            &[],
        )
    }

    pub fn submit_turn_with_attachments(
        &self,
        session_id: &str,
        input: &str,
        idempotency_key: &str,
        model: Option<&str>,
        reasoning_effort: Option<&str>,
        image_ids: &[String],
    ) -> SdkResult<TurnResponse> {
        if input.is_empty() {
            return Err(BusinessError::invalid("input is required"));
        }
        if idempotency_key.is_empty() {
            return Err(BusinessError::invalid("idempotency_key is required"));
        }
        match self
            .runtime
            .block_on(self.state.agent.submit_with_attachments(
                session_id,
                idempotency_key,
                input,
                model,
                reasoning_effort,
                image_ids,
            )) {
            Ok(response) => Ok(response),
            Err(error) if error.code == "approval_required" => Ok(TurnResponse::AwaitingApproval {
                turn_id: detail_string(&error, "turn_id")?,
                tool_call_id: detail_string(&error, "tool_call_id")?,
                approval_id: detail_string(&error, "approval_id")?,
            }),
            Err(error) if error.code == "question_required" => Ok(TurnResponse::AwaitingQuestion {
                turn_id: detail_string(&error, "turn_id")?,
                tool_call_id: detail_string(&error, "tool_call_id")?,
                request_id: detail_string(&error, "request_id")?,
            }),
            Err(error) => Err(error),
        }
    }

    pub fn cancel_turn(&self, _session_id: &str, turn_id: &str) -> SdkResult<CancellationOutcome> {
        if !self.state.agent.cancel(turn_id) {
            return Err(BusinessError::new("conflict", "turn is not running"));
        }
        Ok(CancellationOutcome {
            turn_id: turn_id.to_string(),
            status: "cancellation_requested",
        })
    }

    pub fn retry_last_turn(&self, session_id: &str) -> SdkResult<TurnResponse> {
        self.runtime
            .block_on(self.state.agent.retry_last_turn(session_id))
    }

    pub fn get_approval(&self, approval_id: &str) -> SdkResult<ApprovalRecord> {
        self.state
            .store
            .approval(approval_id)?
            .ok_or_else(|| BusinessError::missing("approval"))
    }

    pub fn resolve_approval(
        &self,
        approval_id: &str,
        decision: &str,
    ) -> SdkResult<ApprovalOutcome> {
        if !["deny", "allow_once", "allow_session"].contains(&decision) {
            return Err(BusinessError::invalid("invalid approval decision"));
        }
        let resolved = self
            .runtime
            .block_on(self.state.agent.resolve_approval(approval_id, decision))?;
        if !resolved {
            return Err(BusinessError::new(
                "conflict",
                "approval is missing or already resolved",
            ));
        }
        Ok(ApprovalOutcome {
            approval_id: approval_id.to_string(),
            decision: decision.to_string(),
        })
    }

    pub fn reply_question(&self, request_id: &str, answers: &Value) -> SdkResult<QuestionOutcome> {
        let answers = answers
            .as_array()
            .ok_or_else(|| BusinessError::invalid("answers must be an array"))?;
        let answers = answers
            .iter()
            .map(|answer| {
                answer
                    .as_array()
                    .ok_or_else(|| BusinessError::invalid("each answer must be an array"))
                    .and_then(|values| {
                        values
                            .iter()
                            .map(|value| {
                                value.as_str().map(str::to_string).ok_or_else(|| {
                                    BusinessError::invalid("answers must contain strings")
                                })
                            })
                            .collect()
                    })
            })
            .collect::<SdkResult<Vec<Vec<String>>>>()?;
        let resolved = self.runtime.block_on(
            self.state
                .agent
                .resolve_question(request_id, answers, false),
        )?;
        if !resolved {
            return Err(BusinessError::new(
                "conflict",
                "question is missing or already resolved",
            ));
        }
        Ok(QuestionOutcome {
            request_id: request_id.to_string(),
            status: "replied".into(),
        })
    }

    pub fn reject_question(&self, request_id: &str) -> SdkResult<QuestionOutcome> {
        let resolved = self.runtime.block_on(self.state.agent.resolve_question(
            request_id,
            Vec::new(),
            true,
        ))?;
        if !resolved {
            return Err(BusinessError::new(
                "conflict",
                "question is missing or already resolved",
            ));
        }
        Ok(QuestionOutcome {
            request_id: request_id.to_string(),
            status: "rejected".into(),
        })
    }

    pub fn subscribe_session_events(
        &self,
        session_id: String,
        _after: i64,
        callback: SunCodeEventCallback,
        user_data: *mut c_void,
    ) -> SdkResult<AgentSubscription> {
        logging::write(
            Level::Debug,
            "subscribe",
            format!("begin session={session_id} after={_after}"),
        );
        if self.state.store.session_by_id(&session_id)?.is_none() {
            return Err(BusinessError::missing("session"));
        }

        // Events are live-only. Hosts recover durable state by reading a fresh snapshot.
        let mut receiver = self.state.events.subscribe();
        let cancellation = CancellationToken::new();
        let cancellation_for_thread = cancellation.clone();
        let handle = self.runtime.handle().clone();
        let user_data = user_data as usize;
        let log_session_id = session_id.clone();
        let subscribed_session_id = session_id.clone();
        let join = std::thread::spawn(move || loop {
            let next = handle.block_on(async {
                tokio::select! {
                    _ = cancellation_for_thread.cancelled() => None,
                    value = receiver.recv() => Some(value),
                }
            });
            match next {
                None => {
                    logging::write(
                        Level::Debug,
                        "subscribe",
                        format!("thread_exit session={log_session_id} reason=cancelled"),
                    );
                    break;
                }
                Some(Ok(event)) if event.session_id == subscribed_session_id => {
                    subscriptions::emit_sdk_event(callback, user_data, &event);
                }
                Some(Ok(_)) => {}
                Some(Err(broadcast::error::RecvError::Lagged(_))) => {
                    logging::write(
                        Level::Warn,
                        "subscribe",
                        format!("lagged session={log_session_id}"),
                    );
                    let event = SessionEvent {
                        session_id: subscribed_session_id.clone(),
                        occurred_at: chrono::Utc::now()
                            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                        event_type: "resync.required".into(),
                        payload: json!({"reason":"subscriber_lagged"}),
                    };
                    subscriptions::emit_sdk_event(callback, user_data, &event);
                }
                Some(Err(broadcast::error::RecvError::Closed)) => {
                    logging::write(
                        Level::Error,
                        "subscribe",
                        format!("thread_exit session={log_session_id} reason=channel_closed unexpected=true"),
                    );
                    break;
                }
            }
        });
        logging::write(
            Level::Info,
            "subscribe",
            format!("ready session={session_id}"),
        );
        Ok(AgentSubscription {
            session_id,
            cancellation,
            join: Mutex::new(Some(join)),
        })
    }
}
