//! The agent loop for one turn: model requests, tool execution, compaction,
//! and resumption after an approval or a question.
//!
//! [`AgentRuntime`] admits a turn and owns session bookkeeping and the
//! runtimes it starts and stops. A `TurnLoop` borrows only the runtime's
//! [`TurnServices`] and the [`ChildTurns`] it may delegate to, with the turn's
//! state, and runs the turn to completion or suspension. Every agent, primary
//! or specialist, runs through this loop; what differs comes from its
//! `suncode_agents::AgentDefinition`.

use super::run::*;
use super::services::{ChildTurns, TurnServices};
use super::*;

pub(super) struct TurnLoop<'a> {
    pub(super) services: &'a TurnServices,
    pub(super) children: &'a dyn ChildTurns,
    pub(super) context: &'a mut Continuation,
    pub(super) token: CancellationToken,
}

impl<'a> TurnLoop<'a> {
    pub(super) fn new(
        runtime: &'a AgentRuntime,
        context: &'a mut Continuation,
        token: CancellationToken,
    ) -> Self {
        Self {
            services: &runtime.services,
            children: runtime,
            context,
            token,
        }
    }

    /// A loop for another run of the same turn, sharing these services.
    fn resumed<'b>(&'b self, context: &'b mut Continuation) -> TurnLoop<'b> {
        TurnLoop {
            services: self.services,
            children: self.children,
            context,
            token: self.token.clone(),
        }
    }
}

/// Resumption of a turn suspended on a question or an approval.
impl TurnLoop<'_> {
    pub(super) async fn continue_question(&mut self) -> Result<TurnResponse, BusinessError> {
        if let Some(call) = self.context.pending_call.take() {
            let answers = self.context.question_answers.take().unwrap_or_default();
            let result = json!({"answers": answers, "rejected": self.context.question_rejected});
            self.services
                .tool_state(self.context, &call, ToolState::Succeeded, None)?;
            self.services.emit(
                &self.context.session_id,
                EventPayload::ToolResult(ToolResultPayload {
                    turn_id: self.context.turn_id.clone(),
                    call_id: self.context.active_call_id.clone(),
                    tool_call_id: call.call_id.clone(),
                    result: result.clone(),
                }),
            )?;
            let mut tool = Message::text(
                "tool",
                serde_json::to_string(&result).unwrap_or_else(|_| "{}".into()),
            );
            tool.tool_call_id = Some(call.call_id.clone());
            self.context.messages.push(tool.clone());
            self.services.emit(
                &self.context.session_id,
                EventPayload::MessageTool(MessageToolPayload {
                    turn_id: self.context.turn_id.clone(),
                    call_id: self.context.active_call_id.clone(),
                    tool_call_id: call.call_id.clone(),
                    message: tool,
                }),
            )?;
        }
        let siblings = std::mem::take(&mut self.context.remaining_calls);
        self.resolve_calls(siblings).await?;
        let provider = self
            .services
            .providers
            .route(&self.context.model)
            .ok_or_else(|| BusinessError::new("model_unavailable", "model is not advertised"))?;
        // Run on a copy, as a fresh submission does, so the suspended
        // snapshot the caller holds is not advanced by the resumed run.
        let mut resumed = self.context.clone();
        let response = self
            .resumed(&mut resumed)
            .run(TurnStart::Resume, provider)
            .await?;
        db_blocking(|| {
            self.services.store.complete_turn(
                &self.context.session_id,
                &self.context.submission_key,
                &serde_json::to_value(&response).map_err(|_| {
                    BusinessError::new("agent_unavailable", "turn response could not be stored")
                })?,
            )
        })?;
        Ok(response)
    }

    pub(super) async fn continue_approved(&mut self) -> Result<TurnResponse, BusinessError> {
        if let Some(call) = self.context.pending_call.take() {
            let approval_pin = self.context.pending_approval_pin.take();
            self.execute_call(&call, approval_pin).await?;
            self.context
                .approved_batch_call_ids
                .retain(|call_id| call_id != &call.call_id);
        }
        let siblings = std::mem::take(&mut self.context.remaining_calls);
        self.resolve_calls(siblings).await?;
        let provider = self
            .services
            .providers
            .route(&self.context.model)
            .ok_or_else(|| BusinessError::new("model_unavailable", "model is not advertised"))?;
        // Run on a copy, as a fresh submission does, so the suspended
        // snapshot the caller holds is not advanced by the resumed run.
        let mut resumed = self.context.clone();
        let response = self
            .resumed(&mut resumed)
            .run(TurnStart::Resume, provider)
            .await?;
        db_blocking(|| {
            self.services.store.complete_turn(
                &self.context.session_id,
                &self.context.submission_key,
                &serde_json::to_value(&response).map_err(|_| {
                    BusinessError::new("agent_unavailable", "turn response could not be stored")
                })?,
            )
        })?;
        Ok(response)
    }
}
