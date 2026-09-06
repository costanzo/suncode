//! Diesel row projections shared by the table operation modules.

use diesel::sql_types::{Integer, Nullable, Text};
use diesel::QueryableByName;

#[derive(QueryableByName)]
pub(crate) struct StringRow {
    #[diesel(sql_type = Text)]
    pub(crate) value: String,
}
#[derive(QueryableByName)]
pub(crate) struct JournalRow {
    #[diesel(sql_type = Text)]
    pub(crate) journal_mode: String,
}
#[derive(QueryableByName)]
pub(crate) struct TurnRow {
    #[diesel(sql_type = Text)]
    pub(crate) turn_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) session_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) state: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) model_id: Option<String>,
    #[diesel(sql_type = Text)]
    pub(crate) created_at: String,
    #[diesel(sql_type = Text)]
    pub(crate) updated_at: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) started_at: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) completed_at: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) error_code: Option<String>,
    #[diesel(sql_type = Integer)]
    pub(crate) input_tokens: i32,
    #[diesel(sql_type = Integer)]
    pub(crate) output_tokens: i32,
    #[diesel(sql_type = Integer)]
    pub(crate) total_tokens: i32,
}
#[derive(QueryableByName)]
pub(crate) struct SubmissionRow {
    #[diesel(sql_type = Text)]
    pub(crate) state: String,
    #[diesel(sql_type = Text)]
    pub(crate) turn_id: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) input_json: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) model_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) response_json: Option<String>,
}
#[derive(QueryableByName)]
pub(crate) struct RetryInputRow {
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) input_json: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) model_id: Option<String>,
}
#[derive(QueryableByName)]
pub(crate) struct ApprovalRow {
    #[diesel(sql_type = Text)]
    pub(crate) approval_id: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) project_id: Option<String>,
    #[diesel(sql_type = Text)]
    pub(crate) session_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) turn_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) tool_call_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) operation: String,
    #[diesel(sql_type = Text)]
    pub(crate) arguments_json: String,
    #[diesel(sql_type = Text)]
    pub(crate) status: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) decision: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) decision_source: Option<String>,
    #[diesel(sql_type = Text)]
    pub(crate) created_at: String,
    #[diesel(sql_type = Text)]
    pub(crate) updated_at: String,
}
#[derive(QueryableByName)]
pub(crate) struct ManifestRow {
    #[diesel(sql_type = Text)]
    pub(crate) manifest_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) session_id: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) turn_id: Option<String>,
    #[diesel(sql_type = Text)]
    pub(crate) status: String,
    #[diesel(sql_type = Text)]
    pub(crate) created_at: String,
    #[diesel(sql_type = Text)]
    pub(crate) updated_at: String,
    #[diesel(sql_type = Text)]
    pub(crate) expires_at: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) restored_at: Option<String>,
}
#[derive(QueryableByName)]
pub(crate) struct CheckpointRow {
    #[diesel(sql_type = Text)]
    pub(crate) checkpoint_id: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) manifest_id: Option<String>,
    #[diesel(sql_type = Text)]
    pub(crate) session_id: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) turn_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) tool_call_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) relative_path: Option<String>,
    #[diesel(sql_type = Text)]
    pub(crate) status: String,
    #[diesel(sql_type = Text)]
    pub(crate) created_at: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) restored_at: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) invalidated_at: Option<String>,
    #[diesel(sql_type = Nullable<Integer>)]
    pub(crate) ordinal: Option<i32>,
}
#[derive(QueryableByName)]
pub(crate) struct ExchangeRow {
    #[diesel(sql_type = Text)]
    pub(crate) call_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) session_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) turn_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) provider: String,
    #[diesel(sql_type = Text)]
    pub(crate) model_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) wire_model: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) provider_request_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) provider_response_id: Option<String>,
    #[diesel(sql_type = Text)]
    pub(crate) state: String,
    #[diesel(sql_type = Integer)]
    pub(crate) iteration: i32,
    #[diesel(sql_type = Text)]
    pub(crate) started_at: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) completed_at: Option<String>,
    #[diesel(sql_type = Text)]
    pub(crate) input_messages_json: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) output_message_json: Option<String>,
    #[diesel(sql_type = Text)]
    pub(crate) tool_calls_json: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) usage_json: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) finish_reason: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) error_json: Option<String>,
}
#[derive(QueryableByName)]
pub(crate) struct RecoveryRow {
    #[diesel(sql_type = Text)]
    pub(crate) approval_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) session_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) turn_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) snapshot_json: String,
    #[diesel(sql_type = Text)]
    pub(crate) status: String,
}
#[derive(QueryableByName)]
pub(crate) struct MessageRow {
    #[diesel(sql_type = Text)]
    pub(crate) message_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) session_id: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) turn_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) session_call_id: Option<String>,
    #[diesel(sql_type = Text)]
    pub(crate) role: String,
    #[diesel(sql_type = Text)]
    pub(crate) message_json: String,
    #[diesel(sql_type = Text)]
    pub(crate) created_at: String,
}
#[derive(QueryableByName)]
pub(crate) struct ToolRow {
    #[diesel(sql_type = Text)]
    pub(crate) turn_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) tool_call_id: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) session_call_id: Option<String>,
    #[diesel(sql_type = Text)]
    pub(crate) name: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) request_json: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) result_json: Option<String>,
    #[diesel(sql_type = Text)]
    pub(crate) state: String,
    #[diesel(sql_type = Nullable<Integer>)]
    pub(crate) ordinal: Option<i32>,
    #[diesel(sql_type = Text)]
    pub(crate) created_at: String,
    #[diesel(sql_type = Text)]
    pub(crate) updated_at: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) completed_at: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) error_code: Option<String>,
}
#[derive(QueryableByName)]
pub(crate) struct TodoRow {
    #[diesel(sql_type = Text)]
    pub(crate) turn_id: String,
    #[diesel(sql_type = Integer)]
    pub(crate) ordinal: i32,
    #[diesel(sql_type = Text)]
    pub(crate) content: String,
    #[diesel(sql_type = Text)]
    pub(crate) status: String,
    #[diesel(sql_type = Text)]
    pub(crate) priority: String,
    #[diesel(sql_type = Text)]
    pub(crate) created_at: String,
    #[diesel(sql_type = Text)]
    pub(crate) updated_at: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) completed_at: Option<String>,
}
