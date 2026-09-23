use super::*;
use suncode_agent::{agent::TurnResponse, domain::ApprovalRecord};

impl AsyncAgentSdk {
    pub async fn submit_turn(
        &self,
        session_id: &str,
        input: &str,
        idempotency_key: &str,
        model: Option<&str>,
        reasoning_effort: Option<&str>,
    ) -> SdkResult<TurnResponse> {
        self.session_for_user(session_id)?;
        self.submit_turn_with_attachments(
            session_id,
            input,
            idempotency_key,
            model,
            reasoning_effort,
            &[],
        )
        .await
    }

    pub async fn submit_turn_with_attachments(
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
            .state
            .agent
            .submit_with_attachments(
                session_id,
                idempotency_key,
                input,
                model,
                reasoning_effort,
                image_ids,
            )
            .await
        {
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

    pub fn cancel_turn(&self, session_id: &str, turn_id: &str) -> SdkResult<CancellationOutcome> {
        self.session_for_user(session_id)?;
        if self.state.agent.cancel(turn_id) {
            return Ok(CancellationOutcome {
                turn_id: turn_id.to_string(),
                status: "cancellation_requested",
            });
        }
        let cancelled = self.state.agent.cancel_dormant_turn(session_id, turn_id)?;
        Ok(CancellationOutcome {
            turn_id: turn_id.to_string(),
            status: if cancelled {
                "cancelled"
            } else {
                "not_running"
            }
        })
    }

    pub async fn retry_last_turn(&self, session_id: &str) -> SdkResult<TurnResponse> {
        let session = self.session_for_user(session_id)?;
        if session.kind != "primary" {
            return Err(BusinessError::new(
                "child_session_read_only",
                "child sessions cannot be retried directly",
            ));
        }
        self.state.agent.retry_last_turn(session_id).await
    }

    pub fn get_approval(&self, approval_id: &str) -> SdkResult<ApprovalRecord> {
        let approval = self
            .state
            .store
            .approval(approval_id)?
            .ok_or_else(|| BusinessError::missing("approval"))?;
        self.session_for_user(&approval.session_id)?;
        Ok(approval)
    }

    pub fn pending_approval(&self, session_id: &str) -> SdkResult<Option<ApprovalRecord>> {
        self.session_for_user(session_id)?;
        self.state.store.pending_approval(session_id)
    }

    pub async fn resolve_approval(
        &self,
        approval_id: &str,
        decision: &str,
    ) -> SdkResult<ApprovalOutcome> {
        if !["deny", "allow_once", "allow_session"].contains(&decision) {
            return Err(BusinessError::invalid("invalid approval decision"));
        }
        let approval = self
            .state
            .store
            .approval(approval_id)?
            .ok_or_else(|| BusinessError::missing("approval"))?;
        self.session_for_user(&approval.session_id)?;
        let resolved = self
            .state
            .agent
            .resolve_approval(approval_id, decision)
            .await?;
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

    pub async fn reply_question(
        &self,
        request_id: &str,
        answers: &Value,
    ) -> SdkResult<QuestionOutcome> {
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
        let resolved = self
            .state
            .agent
            .resolve_question(request_id, answers, false)
            .await?;
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

    pub async fn reject_question(&self, request_id: &str) -> SdkResult<QuestionOutcome> {
        let resolved = self
            .state
            .agent
            .resolve_question(request_id, Vec::new(), true)
            .await?;
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

    pub fn subscribe_session_events(&self, session_id: &str) -> SdkResult<SessionEventStream> {
        logging::debug("subscribe", format!("begin session={session_id}"));
        self.session_for_user(session_id)?;
        let stream = SessionEventStream::new(
            session_id.to_string(),
            self.state.events.subscribe(session_id.to_string()),
        );
        logging::info("subscribe", format!("ready session={session_id}"));
        Ok(stream)
    }
}
