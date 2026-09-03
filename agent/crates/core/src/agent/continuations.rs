impl Agent {
    pub async fn resolve_approval(
        &self,
        approval_id: &str,
        decision: &str,
    ) -> Result<bool, BusinessError> {
        let Some(suspended) = self.store.resolve_approval(approval_id, decision)? else {
            return Ok(false);
        };
        let mut continuation: Continuation =
            serde_json::from_value(suspended.snapshot).map_err(|_| {
                BusinessError::new("agent_unavailable", "approval continuation is invalid")
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
            let result = agent.continue_approved(&mut continuation, token).await;
            let status = if result.is_ok() {
                "completed"
            } else {
                "failed"
            };
            let _ = agent.store.finish_suspended(&approval_id, status);
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
    ) -> Result<bool, BusinessError> {
        let snapshot = self.store.question_snapshot(request_id)?.ok_or_else(|| {
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
    ) -> Result<TurnResponse, BusinessError> {
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
            .ok_or_else(|| BusinessError::new("model_unavailable", "model is not advertised"))?;
        let response = self
            .run(continuation.clone(), None, token, provider)
            .await?;
        self.store.complete_turn(
            &continuation.session_id,
            &continuation.submission_key,
            &serde_json::to_value(&response).map_err(|_| {
                BusinessError::new("agent_unavailable", "turn response could not be stored")
            })?,
        )?;
        Ok(response)
    }

    async fn continue_approved(
        &self,
        continuation: &mut Continuation,
        token: CancellationToken,
    ) -> Result<TurnResponse, BusinessError> {
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
            .ok_or_else(|| BusinessError::new("model_unavailable", "model is not advertised"))?;
        let response = self
            .run(continuation.clone(), None, token, provider)
            .await?;
        self.store.complete_turn(
            &continuation.session_id,
            &continuation.submission_key,
            &serde_json::to_value(&response).map_err(|_| {
                BusinessError::new("agent_unavailable", "turn response could not be stored")
            })?,
        )?;
        Ok(response)
    }

}
