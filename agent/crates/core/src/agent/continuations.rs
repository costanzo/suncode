//! Part of the `AgentRuntime` implementation; see `agent/mod.rs`.

use super::support::*;
use super::*;

impl AgentRuntime {
    pub(super) fn persist_child_invocation_outcome(
        &self,
        continuation: &Continuation,
        outcome: &Result<TurnResponse, BusinessError>,
    ) -> Result<(), BusinessError> {
        let Some(agent_id) = continuation.agent_id.as_deref() else {
            return Ok(());
        };
        let definition = suncode_agents::by_id(Some(agent_id))
            .ok_or_else(|| BusinessError::unavailable("child session agent is not available"))?;
        let (state, result, error_code) = match outcome {
            Ok(TurnResponse::Completed {
                message,
                usage,
                iterations,
                tool_calls,
                ..
            }) => (
                "completed",
                json!({
                    "status":"completed",
                    "agentId":definition.id,
                    "agentName":definition.name,
                    "agentDisplayName":definition.display_name,
                    "childSessionId":continuation.session_id,
                    "result":message.text_content(),
                    "usage":usage,
                    "iterations":iterations,
                    "toolCalls":tool_calls
                }),
                None,
            ),
            Ok(TurnResponse::AwaitingApproval { approval_id, .. }) => (
                "awaiting_approval",
                json!({
                    "status":"awaiting_approval",
                    "agentId":definition.id,
                    "childSessionId":continuation.session_id,
                    "approvalId":approval_id
                }),
                None,
            ),
            Ok(TurnResponse::AwaitingQuestion { .. }) => (
                "failed",
                json!({
                    "status":"failed",
                    "agentId":definition.id,
                    "childSessionId":continuation.session_id,
                    "error":{"code":"agent_question_denied","message":"child agents cannot ask the user questions"}
                }),
                Some("agent_question_denied"),
            ),
            Ok(TurnResponse::Queued { .. }) => (
                "failed",
                json!({
                    "status":"failed",
                    "agentId":definition.id,
                    "childSessionId":continuation.session_id,
                    "error":{"code":"agent_busy","message":"child agent unexpectedly queued work"}
                }),
                Some("agent_busy"),
            ),
            Err(error) if error.code == states::error_codes::APPROVAL_REQUIRED => {
                let approval_id = error
                    .details
                    .get("approval_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        BusinessError::unavailable("child approval outcome is missing approval_id")
                    })?;
                (
                    "awaiting_approval",
                    json!({
                        "status":"awaiting_approval",
                        "agentId":definition.id,
                        "childSessionId":continuation.session_id,
                        "approvalId":approval_id
                    }),
                    None,
                )
            }
            Err(error) => {
                let state = if error.code == states::error_codes::CANCELLED {
                    "cancelled"
                } else {
                    "failed"
                };
                (
                    state,
                    json!({
                        "status":state,
                        "agentId":definition.id,
                        "childSessionId":continuation.session_id,
                        "error":{"code":error.code,"message":error.message,"details":error.details}
                    }),
                    Some(error.code.as_str()),
                )
            }
        };
        db_blocking(|| {
            self.services.store.update_subagent_invocation(
                &continuation.session_id,
                state,
                Some(&result),
                error_code,
            )
        })
    }

    pub async fn resolve_approval(
        &self,
        approval_id: &str,
        decision: &str,
    ) -> Result<bool, BusinessError> {
        self.ensure_running()?;
        let Some(suspended) =
            db_blocking(|| self.services.store.resolve_approval(approval_id, decision))?
        else {
            return Ok(false);
        };
        let mut continuation: Continuation =
            serde_json::from_value(suspended.snapshot).map_err(|_| {
                BusinessError::new("agent_unavailable", "approval continuation is invalid")
            })?;
        self.services.emit(
            &continuation.session_id,
            EventPayload::ApprovalResolved(ApprovalResolvedPayload {
                approval_id: approval_id.to_owned(),
                turn_id: continuation.turn_id.clone(),
                decision: decision.to_owned(),
            }),
        )?;
        if decision == "deny" {
            self.clear_queued_messages(&continuation.session_id);
            if let Some(call) = continuation.pending_call.take() {
                self.services.tool_state(
                    &continuation,
                    &call,
                    ToolState::Denied,
                    Some("user_denied"),
                )?;
            }
            self.services.turn_state(
                &continuation,
                TurnState::Failed,
                Some("authorization_denied"),
            )?;
            db_blocking(|| {
                self.services.store.fail_turn(
                    &continuation.session_id,
                    &continuation.submission_key,
                    &json!({"code":"authorization_denied","message":"Approval was denied"}),
                )
            })?;
            self.persist_child_invocation_outcome(
                &continuation,
                &Err(BusinessError::new(
                    "authorization_denied",
                    "Approval was denied",
                )),
            )?;
            db_blocking(|| self.services.store.finish_suspended(approval_id, "denied"))?;
            return Ok(true);
        }
        if continuation.agent_id.is_some() {
            db_blocking(|| {
                self.services.store.update_subagent_invocation(
                    &continuation.session_id,
                    "running",
                    None,
                    None,
                )
            })?;
        }
        let token = CancellationToken::new();
        self.cancellations
            .lock()
            .map_err(|_| BusinessError::new("agent_unavailable", "cancellation state unavailable"))?
            .insert(continuation.turn_id.clone(), token.clone());
        self.active_turns
            .lock()
            .map_err(|_| BusinessError::new("agent_unavailable", "turn state unavailable"))?
            .insert(
                continuation.session_id.clone(),
                continuation.turn_id.clone(),
            );
        let agent = self.clone();
        let approval_id = approval_id.to_string();
        tokio::spawn(async move {
            let session_lock = agent.session_lock(&continuation.session_id).await;
            let _guard = session_lock.lock().await;
            let result = TurnLoop::new(&agent, &mut continuation, token)
                .continue_approved()
                .await;
            let suspended_again = result
                .as_ref()
                .err()
                .is_some_and(|error| states::suspends_turn(&error.code));
            if let Err(error) = agent.persist_child_invocation_outcome(&continuation, &result) {
                logging::write_business_error(
                    "subagent",
                    "persist_approval_outcome",
                    &error,
                    format!(
                        "session={} turn={}",
                        continuation.session_id, continuation.turn_id
                    ),
                );
            }
            let status = if result.is_ok() || suspended_again {
                "completed"
            } else {
                "failed"
            };
            let _ = db_blocking(|| agent.services.store.finish_suspended(&approval_id, status));
            if let Err(error) = &result {
                logging::write_business_error(
                    "turn",
                    "continue_approval",
                    error,
                    format!(
                        "session={} turn={}",
                        continuation.session_id, continuation.turn_id
                    ),
                );
                if !suspended_again {
                    agent.clear_queued_messages(&continuation.session_id);
                    let _ = agent.services.turn_state(
                        &continuation,
                        states::failed_turn_state(&error.code),
                        Some(&error.code),
                    );
                    let _ = db_blocking(|| {
                        agent.services.store.fail_turn(
                        &continuation.session_id,
                        &continuation.submission_key,
                        &json!({"code":error.code,"message":error.message,"details":error.details}),
                    )
                    });
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

    pub async fn resolve_question(
        &self,
        request_id: &str,
        answers: Vec<Vec<String>>,
        rejected: bool,
    ) -> Result<bool, BusinessError> {
        self.ensure_running()?;
        let snapshot = self
            .services
            .store
            .question_snapshot(request_id)?
            .ok_or_else(|| {
                BusinessError::new("conflict", "question is missing or already resolved")
            })?;
        if !rejected {
            let continuation: Continuation = serde_json::from_value(snapshot).map_err(|_| {
                BusinessError::new("agent_unavailable", "question continuation is invalid")
            })?;
            let call = continuation.pending_call.as_ref().ok_or_else(|| {
                BusinessError::new("agent_unavailable", "question call is missing")
            })?;
            validate_question_answers(&call.arguments, &answers)?;
        }
        let Some(suspended) = self
            .services
            .store
            .resolve_question(request_id, &answers, rejected)?
        else {
            return Ok(false);
        };
        let mut continuation: Continuation =
            serde_json::from_value(suspended.snapshot).map_err(|_| {
                BusinessError::new("agent_unavailable", "question continuation is invalid")
            })?;
        let call = continuation
            .pending_call
            .as_ref()
            .ok_or_else(|| BusinessError::new("agent_unavailable", "question call is missing"))?;
        let event = if rejected {
            EventPayload::QuestionRejected(QuestionAnsweredPayload {
                request_id: request_id.to_owned(),
                turn_id: continuation.turn_id.clone(),
                tool_call_id: call.call_id.clone(),
                answers,
            })
        } else {
            EventPayload::QuestionReplied(QuestionAnsweredPayload {
                request_id: request_id.to_owned(),
                turn_id: continuation.turn_id.clone(),
                tool_call_id: call.call_id.clone(),
                answers,
            })
        };
        self.services.emit(&continuation.session_id, event)?;
        let token = CancellationToken::new();
        self.cancellations
            .lock()
            .map_err(|_| BusinessError::new("agent_unavailable", "cancellation state unavailable"))?
            .insert(continuation.turn_id.clone(), token.clone());
        self.active_turns
            .lock()
            .map_err(|_| BusinessError::new("agent_unavailable", "turn state unavailable"))?
            .insert(
                continuation.session_id.clone(),
                continuation.turn_id.clone(),
            );
        let agent = self.clone();
        let request_id = request_id.to_string();
        tokio::spawn(async move {
            let session_lock = agent.session_lock(&continuation.session_id).await;
            let _guard = session_lock.lock().await;
            let result = TurnLoop::new(&agent, &mut continuation, token)
                .continue_question()
                .await;
            let suspended_again = result
                .as_ref()
                .err()
                .is_some_and(|error| states::suspends_turn(&error.code));
            let status = if result.is_ok() || suspended_again {
                "completed"
            } else {
                "failed"
            };
            let _ = db_blocking(|| agent.services.store.finish_suspended(&request_id, status));
            if let Err(error) = &result {
                logging::write_business_error(
                    "turn",
                    "continue_question",
                    error,
                    format!(
                        "session={} turn={}",
                        continuation.session_id, continuation.turn_id
                    ),
                );
                if !suspended_again {
                    agent.clear_queued_messages(&continuation.session_id);
                    let _ = agent.services.turn_state(
                        &continuation,
                        states::failed_turn_state(&error.code),
                        Some(&error.code),
                    );
                    let _ = db_blocking(|| {
                        agent.services.store.fail_turn(
                        &continuation.session_id,
                        &continuation.submission_key,
                        &json!({"code":error.code,"message":error.message,"details":error.details}),
                    )
                    });
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
}
