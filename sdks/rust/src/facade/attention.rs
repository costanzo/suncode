use super::*;
use chrono::{DateTime, Duration, SecondsFormat, Utc};
use suncode_agent::{AgentAttentionEvent, AttentionKind};

impl AsyncAgentSdk {
    pub fn subscribe_attention_events(&self) -> SdkResult<AttentionEventStream> {
        Ok(AttentionEventStream::new(
            self.state.attention_events.subscribe(),
        ))
    }

    pub fn list_attention_candidates(
        &self,
        since: Option<&str>,
        limit: usize,
    ) -> SdkResult<AttentionCandidatesResult> {
        let since = match since {
            Some(value) => DateTime::parse_from_rfc3339(value.trim())
                .map_err(|_| BusinessError::invalid("since must be an RFC 3339 timestamp"))?
                .with_timezone(&Utc),
            None => Utc::now() - Duration::days(7),
        };
        let limit = limit.clamp(1, 512);
        let since = since.to_rfc3339_opts(SecondsFormat::Millis, true);
        let candidates = self
            .state
            .store
            .attention_candidates(&self.state.user_id, &since, limit)?
            .into_iter()
            .map(|candidate| {
                let kind = match candidate.kind.as_str() {
                    "primary_turn_completed" => AttentionKind::PrimaryTurnCompleted,
                    "primary_turn_failed" => AttentionKind::PrimaryTurnFailed,
                    "approval_requested" => AttentionKind::ApprovalRequested,
                    "question_asked" => AttentionKind::QuestionAsked,
                    _ => {
                        return Err(BusinessError::new(
                            "agent_unavailable",
                            "attention candidate kind is invalid",
                        ))
                    }
                };
                Ok(AgentAttentionEvent {
                    kind,
                    correlation_id: candidate.correlation_id,
                    project_id: candidate.project_id,
                    project_display_name: candidate.project_display_name,
                    session_id: candidate.session_id,
                    session_title: candidate.session_title,
                    session_kind: candidate.session_kind,
                    parent_session_id: candidate.parent_session_id,
                    turn_id: candidate.turn_id,
                    occurred_at: candidate.occurred_at,
                })
            })
            .collect::<SdkResult<Vec<_>>>()?;
        Ok(AttentionCandidatesResult { candidates })
    }
}
