use crate::domain::{Message, ToolCall, Usage};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    TurnState,
    ToolState,
    ToolRequested,
    ToolResult,
    ToolOutput,
    MessageUser,
    MessageAssistant,
    MessageTool,
    AssistantDelta,
    TurnQueued,
    TurnCompleted,
    UsageUpdated,
    ContextCompacted,
    ProviderExchangeStarted,
    ProviderExchangeCompleted,
    ProviderExchangeFailed,
    ApprovalRequested,
    ApprovalResolved,
    QuestionAsked,
    QuestionReplied,
    QuestionRejected,
    TodoUpdated,
    CheckpointCaptured,
}

impl EventType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TurnState => "turn.state",
            Self::ToolState => "tool.state",
            Self::ToolRequested => "tool.requested",
            Self::ToolResult => "tool.result",
            Self::ToolOutput => "tool.output",
            Self::MessageUser => "message.user",
            Self::MessageAssistant => "message.assistant",
            Self::MessageTool => "message.tool",
            Self::AssistantDelta => "assistant.delta",
            Self::TurnQueued => "turn.queued",
            Self::TurnCompleted => "turn.completed",
            Self::UsageUpdated => "usage.updated",
            Self::ContextCompacted => "context.compacted",
            Self::ProviderExchangeStarted => "provider.exchange.started",
            Self::ProviderExchangeCompleted => "provider.exchange.completed",
            Self::ProviderExchangeFailed => "provider.exchange.failed",
            Self::ApprovalRequested => "approval.requested",
            Self::ApprovalResolved => "approval.resolved",
            Self::QuestionAsked => "question.asked",
            Self::QuestionReplied => "question.replied",
            Self::QuestionRejected => "question.rejected",
            Self::TodoUpdated => "todo.updated",
            Self::CheckpointCaptured => "checkpoint.captured",
        }
    }
}

macro_rules! payload {
    ($name:ident { $($fields:tt)* }) => {
        #[derive(Debug, Clone, Serialize)]
        pub struct $name { $($fields)* }
    };
}

payload!(TurnStatePayload { pub turn_id: String, pub state: String, pub model_id: Option<String>, pub submission_idempotency_key: Option<String>, pub reason: Option<String>, });
payload!(ToolStatePayload { pub turn_id: String, pub call_id: Option<String>, pub tool_call_id: String, pub name: String, pub state: String, pub reason: Option<String>, });
payload!(ToolRequestedPayload { pub turn_id: String, pub call_id: Option<String>, pub tool_call_id: String, pub name: String, pub arguments: Value, pub ordinal: usize, });
payload!(ToolResultPayload { pub turn_id: String, pub call_id: Option<String>, pub tool_call_id: String, pub result: Value, });
payload!(ToolOutputPayload { pub turn_id: String, pub call_id: Option<String>, pub tool_call_id: String, pub stream: String, pub chunk_base64: String, });
payload!(MessageUserPayload { pub message_id: String, pub turn_id: String, #[serde(skip_serializing_if = "Option::is_none")] pub queued_id: Option<String>, #[serde(skip_serializing_if = "Option::is_none")] pub queued_idempotency_key: Option<String>, pub message: Message, });
payload!(MessageAssistantPayload { pub message_id: String, pub turn_id: String, pub call_id: Option<String>, pub message: Message, pub usage: Usage, pub finish_reason: String, });
payload!(MessageToolPayload { pub turn_id: String, pub call_id: Option<String>, pub tool_call_id: String, pub message: Message, });
payload!(AssistantDeltaPayload { pub turn_id: String, pub text: String, });
payload!(TurnQueuedPayload { pub queued_id: String, pub active_turn_id: String, pub position: usize, });
payload!(TurnCompletedPayload { pub turn_id: String, pub usage: Usage, pub iterations: u32, pub tool_calls: u32, });
payload!(UsageUpdatedPayload { pub turn_id: String, pub usage: Usage, });
payload!(ContextSummaryPayload { pub objective: String, pub important_constraints: Vec<String>, pub completed_work: Vec<String>, pub active_work: Vec<String>, pub blockers: Vec<String>, pub next_action: String, });
payload!(ContextCompactedPayload { pub exchange_id: String, pub turn_id: String, pub provider: String, pub model_id: String, pub wire_model: String, pub iteration: u32, pub started_at: String, pub original_characters: usize, pub retained_characters: usize, pub original_tokens: usize, pub retained_tokens: usize, pub dropped_messages: usize, pub summary: Option<ContextSummaryPayload>, });
payload!(ProviderExchangeStartedPayload { pub exchange_id: String, pub turn_id: String, pub provider: String, pub model_id: String, pub wire_model: String, pub iteration: u32, pub input_messages: Vec<suncode_llm::Message>, });
payload!(ProviderErrorPayload { pub code: String, pub message: String, pub retryable: bool, });
payload!(ProviderExchangeFailedPayload { pub exchange_id: String, pub turn_id: String, pub error: ProviderErrorPayload, pub provider_request_id: Option<String>, });
payload!(ProviderExchangeCompletedPayload { pub exchange_id: String, pub turn_id: String, pub output_message: Message, pub tool_calls: Vec<ToolCall>, pub usage: Option<suncode_llm::Usage>, pub provider_request_id: Option<String>, pub provider_response_id: Option<String>, pub finish_reason: String, });
payload!(ApprovalRequestedPayload { pub turn_id: String, pub tool_call_id: String, pub approval_id: String, pub operation: String, pub arguments: Value, });
payload!(ApprovalResolvedPayload { pub approval_id: String, pub turn_id: String, pub decision: String, });
payload!(QuestionAskedPayload { pub request_id: String, pub turn_id: String, pub tool_call_id: String, pub questions: Value, });
payload!(QuestionAnsweredPayload { pub request_id: String, pub turn_id: String, pub tool_call_id: String, pub answers: Vec<Vec<String>>, });
payload!(TodoEventItem { pub content: String, pub status: String, pub priority: String, });
payload!(TodoUpdatedPayload { pub turn_id: String, pub call_id: Option<String>, pub tool_call_id: String, pub todos: Vec<TodoEventItem>, });
payload!(CheckpointCapturedPayload { pub turn_id: String, pub tool_call_id: String, pub manifest_id: String, pub checkpoint_id: String, pub path: Option<String>, pub ordinal: i64, });

#[derive(Debug, Clone)]
pub enum EventPayload {
    TurnState(TurnStatePayload),
    ToolState(ToolStatePayload),
    ToolRequested(ToolRequestedPayload),
    ToolResult(ToolResultPayload),
    ToolOutput(ToolOutputPayload),
    MessageUser(MessageUserPayload),
    MessageAssistant(MessageAssistantPayload),
    MessageTool(MessageToolPayload),
    AssistantDelta(AssistantDeltaPayload),
    TurnQueued(TurnQueuedPayload),
    TurnCompleted(TurnCompletedPayload),
    UsageUpdated(UsageUpdatedPayload),
    ContextCompacted(ContextCompactedPayload),
    ProviderExchangeStarted(ProviderExchangeStartedPayload),
    ProviderExchangeCompleted(ProviderExchangeCompletedPayload),
    ProviderExchangeFailed(ProviderExchangeFailedPayload),
    ApprovalRequested(ApprovalRequestedPayload),
    ApprovalResolved(ApprovalResolvedPayload),
    QuestionAsked(QuestionAskedPayload),
    QuestionReplied(QuestionAnsweredPayload),
    QuestionRejected(QuestionAnsweredPayload),
    TodoUpdated(TodoUpdatedPayload),
    CheckpointCaptured(CheckpointCapturedPayload),
}

impl EventPayload {
    pub const fn event_type(&self) -> EventType {
        match self {
            Self::TurnState(_) => EventType::TurnState,
            Self::ToolState(_) => EventType::ToolState,
            Self::ToolRequested(_) => EventType::ToolRequested,
            Self::ToolResult(_) => EventType::ToolResult,
            Self::ToolOutput(_) => EventType::ToolOutput,
            Self::MessageUser(_) => EventType::MessageUser,
            Self::MessageAssistant(_) => EventType::MessageAssistant,
            Self::MessageTool(_) => EventType::MessageTool,
            Self::AssistantDelta(_) => EventType::AssistantDelta,
            Self::TurnQueued(_) => EventType::TurnQueued,
            Self::TurnCompleted(_) => EventType::TurnCompleted,
            Self::UsageUpdated(_) => EventType::UsageUpdated,
            Self::ContextCompacted(_) => EventType::ContextCompacted,
            Self::ProviderExchangeStarted(_) => EventType::ProviderExchangeStarted,
            Self::ProviderExchangeCompleted(_) => EventType::ProviderExchangeCompleted,
            Self::ProviderExchangeFailed(_) => EventType::ProviderExchangeFailed,
            Self::ApprovalRequested(_) => EventType::ApprovalRequested,
            Self::ApprovalResolved(_) => EventType::ApprovalResolved,
            Self::QuestionAsked(_) => EventType::QuestionAsked,
            Self::QuestionReplied(_) => EventType::QuestionReplied,
            Self::QuestionRejected(_) => EventType::QuestionRejected,
            Self::TodoUpdated(_) => EventType::TodoUpdated,
            Self::CheckpointCaptured(_) => EventType::CheckpointCaptured,
        }
    }

    pub fn into_value(self) -> Value {
        macro_rules! serialize {
            ($value:expr) => {
                serde_json::to_value($value).expect("event payload is serializable")
            };
        }
        match self {
            Self::TurnState(v) => serialize!(v),
            Self::ToolState(v) => serialize!(v),
            Self::ToolRequested(v) => serialize!(v),
            Self::ToolResult(v) => serialize!(v),
            Self::ToolOutput(v) => serialize!(v),
            Self::MessageUser(v) => serialize!(v),
            Self::MessageAssistant(v) => serialize!(v),
            Self::MessageTool(v) => serialize!(v),
            Self::AssistantDelta(v) => serialize!(v),
            Self::TurnQueued(v) => serialize!(v),
            Self::TurnCompleted(v) => serialize!(v),
            Self::UsageUpdated(v) => serialize!(v),
            Self::ContextCompacted(v) => serialize!(v),
            Self::ProviderExchangeStarted(v) => serialize!(v),
            Self::ProviderExchangeCompleted(v) => serialize!(v),
            Self::ProviderExchangeFailed(v) => serialize!(v),
            Self::ApprovalRequested(v) => serialize!(v),
            Self::ApprovalResolved(v) => serialize!(v),
            Self::QuestionAsked(v) => serialize!(v),
            Self::QuestionReplied(v) => serialize!(v),
            Self::QuestionRejected(v) => serialize!(v),
            Self::TodoUpdated(v) => serialize!(v),
            Self::CheckpointCaptured(v) => serialize!(v),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn payload_selects_the_stable_event_name() {
        let event = EventPayload::TurnState(TurnStatePayload {
            turn_id: "turn-1".into(),
            state: "running".into(),
            model_id: Some("gpt-5.5".into()),
            submission_idempotency_key: Some("submission-1".into()),
            reason: None,
        });
        assert_eq!(event.event_type().as_str(), "turn.state");
        let payload = event.into_value();
        assert_eq!(payload["turn_id"], "turn-1");
        assert!(payload["reason"].is_null());
    }
}
