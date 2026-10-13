//! Typed turn and tool-call lifecycle states.

/// Lifecycle state of a turn.
///
/// Mirrors the `state` CHECK constraint in the SQLite schema; event payloads keep
/// carrying the string form so the public event contract is unchanged.
// Some variants are written only by the data layer (recovery, reconciliation).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TurnState {
    Admitted,
    Queued,
    Preparing,
    CallingModel,
    ResolvingCalls,
    Compacting,
    Completed,
    Failed,
    Cancelled,
    Interrupted,
}

impl TurnState {
    #[cfg(test)]
    const ALL: &[Self] = &[
        Self::Admitted,
        Self::Queued,
        Self::Preparing,
        Self::CallingModel,
        Self::ResolvingCalls,
        Self::Compacting,
        Self::Completed,
        Self::Failed,
        Self::Cancelled,
        Self::Interrupted,
    ];

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Queued => "queued",
            Self::Preparing => "preparing",
            Self::CallingModel => "calling_model",
            Self::ResolvingCalls => "resolving_calls",
            Self::Compacting => "compacting",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Interrupted => "interrupted",
        }
    }
}

/// Lifecycle state of a single tool call.
///
/// Mirrors the `state` CHECK constraint in the SQLite schema; event payloads keep
/// carrying the string form so the public event contract is unchanged.
// Some variants are written only by the data layer (recovery, reconciliation).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ToolState {
    Requested,
    Validating,
    PolicyCheck,
    Denied,
    AwaitingApproval,
    AwaitingQuestion,
    Authorized,
    Executing,
    Succeeded,
    Failed,
    TimedOut,
    UnknownCompletion,
    Reconciling,
}

impl ToolState {
    #[cfg(test)]
    const ALL: &[Self] = &[
        Self::Requested,
        Self::Validating,
        Self::PolicyCheck,
        Self::Denied,
        Self::AwaitingApproval,
        Self::AwaitingQuestion,
        Self::Authorized,
        Self::Executing,
        Self::Succeeded,
        Self::Failed,
        Self::TimedOut,
        Self::UnknownCompletion,
        Self::Reconciling,
    ];

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Validating => "validating",
            Self::PolicyCheck => "policy_check",
            Self::Denied => "denied",
            Self::AwaitingApproval => "awaiting_approval",
            Self::AwaitingQuestion => "awaiting_question",
            Self::Authorized => "authorized",
            Self::Executing => "executing",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::TimedOut => "timed_out",
            Self::UnknownCompletion => "unknown_completion",
            Self::Reconciling => "reconciling",
        }
    }
}

/// Error codes the agent loop branches on. Codes are part of the client contract,
/// so the strings themselves must not change.
pub(super) mod error_codes {
    pub(in crate::agent) const CANCELLED: &str = "cancelled";
    pub(in crate::agent) const APPROVAL_REQUIRED: &str = "approval_required";
    pub(in crate::agent) const QUESTION_REQUIRED: &str = "question_required";
    pub(in crate::agent) const CONTEXT_OVERFLOW: &str = "context_overflow";
}

/// Whether an error only suspends the turn while it waits for the user.
pub(super) fn suspends_turn(error_code: &str) -> bool {
    error_code == error_codes::APPROVAL_REQUIRED || error_code == error_codes::QUESTION_REQUIRED
}

/// Terminal turn state for a failed run: `cancelled` when the error was a cancellation.
pub(super) fn failed_turn_state(error_code: &str) -> TurnState {
    if error_code == error_codes::CANCELLED {
        TurnState::Cancelled
    } else {
        TurnState::Failed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Values allowed by the `state` CHECK constraint of a schema script.
    fn schema_states(sql: &str) -> Vec<&str> {
        let start = sql.find("state TEXT NOT NULL CHECK(state IN (").unwrap();
        let end = start + sql[start..].find("))").unwrap();
        sql[start..end].split('\'').skip(1).step_by(2).collect()
    }

    #[test]
    fn states_match_the_sqlite_schema() {
        let turn = include_str!("../../../database/src/sqlite/schema/session_turn.sql");
        let tool = include_str!("../../../database/src/sqlite/schema/session_tool_use.sql");
        let turn_states: Vec<_> = TurnState::ALL.iter().map(|state| state.as_str()).collect();
        let tool_states: Vec<_> = ToolState::ALL.iter().map(|state| state.as_str()).collect();
        assert_eq!(turn_states, schema_states(turn));
        assert_eq!(tool_states, schema_states(tool));
    }
}
