//! Database row mappings for provider and model catalog tables.

use diesel::sql_types::{Integer, Nullable, Text};

#[derive(diesel::QueryableByName)]
pub(crate) struct ProviderRow {
    #[diesel(sql_type = Text)]
    pub provider_id: String,
    #[diesel(sql_type = Text)]
    pub display_name: String,
    #[diesel(sql_type = Text)]
    pub endpoint: String,
    #[diesel(sql_type = Text)]
    pub adapter_type: String,
    #[diesel(sql_type = Integer)]
    pub api_key_configured: i32,
    #[diesel(sql_type = Integer)]
    pub enabled: i32,
    #[diesel(sql_type = Integer)]
    pub sort_order: i32,
    #[diesel(sql_type = Text)]
    pub created_at: String,
    #[diesel(sql_type = Text)]
    pub updated_at: String,
}

#[derive(diesel::QueryableByName)]
pub(crate) struct ModelRow {
    #[diesel(sql_type = Text)]
    pub model_id: String,
    #[diesel(sql_type = Text)]
    pub provider_id: String,
    #[diesel(sql_type = Text)]
    pub display_name: String,
    #[diesel(sql_type = Text)]
    pub request_model: String,
    #[diesel(sql_type = Integer)]
    pub context_tokens: i32,
    #[diesel(sql_type = Integer)]
    pub auto_compact_tokens: i32,
    #[diesel(sql_type = Nullable<Integer>)]
    pub max_output_tokens: Option<i32>,
    #[diesel(sql_type = Integer)]
    pub supports_streaming: i32,
    #[diesel(sql_type = Integer)]
    pub supports_tool_use: i32,
    #[diesel(sql_type = Integer)]
    pub supports_vision: i32,
    #[diesel(sql_type = Integer)]
    pub supports_structured_output: i32,
    #[diesel(sql_type = Integer)]
    pub supports_cancellation: i32,
    #[diesel(sql_type = Integer)]
    pub supports_reasoning_effort: i32,
    #[diesel(sql_type = Integer)]
    pub enabled: i32,
    #[diesel(sql_type = Integer)]
    pub sort_order: i32,
    #[diesel(sql_type = Text)]
    pub created_at: String,
    #[diesel(sql_type = Text)]
    pub updated_at: String,
    #[diesel(sql_type = Text)]
    pub reasoning_efforts: String,
}
