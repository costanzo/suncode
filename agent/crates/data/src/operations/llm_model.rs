//! Operations for `llm_model`.

use crate::{
    domain::{LlmModelInput, LlmModelRecord},
    model::ModelRow,
    store::{lock, nonnegative, now, Store},
};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Integer, Nullable, Text};
use suncode_common::BusinessError;

impl Store {
    pub fn llm_models(&self, enabled_only: bool) -> Result<Vec<LlmModelRecord>, BusinessError> {
        let mut connection = lock(&self.connection)?;
        let sql = if enabled_only {
            "SELECT model_id,provider_id,display_name,request_model,context_tokens,auto_compact_tokens,max_output_tokens,supports_streaming,supports_tool_use,supports_vision,supports_structured_output,supports_cancellation,supports_reasoning_effort,enabled,sort_order,created_at,updated_at,reasoning_efforts FROM llm_model WHERE enabled=1 ORDER BY sort_order,model_id"
        } else {
            "SELECT model_id,provider_id,display_name,request_model,context_tokens,auto_compact_tokens,max_output_tokens,supports_streaming,supports_tool_use,supports_vision,supports_structured_output,supports_cancellation,supports_reasoning_effort,enabled,sort_order,created_at,updated_at,reasoning_efforts FROM llm_model ORDER BY sort_order,model_id"
        };
        sql_query(sql)
            .load::<ModelRow>(&mut *connection)
            .map_err(crate::database_error)?
            .into_iter()
            .map(to_record)
            .collect()
    }

    pub fn upsert_llm_model(&self, input: LlmModelInput<'_>) -> Result<(), BusinessError> {
        if input.model_id.trim().is_empty()
            || input.provider_id.trim().is_empty()
            || input.display_name.trim().is_empty()
            || input.request_model.trim().is_empty()
            || input.context_tokens < 16000
            || input.auto_compact_tokens < 1000
            || input.auto_compact_tokens >= input.context_tokens
            || input.max_output_tokens == Some(0)
            || input.sort_order < 0
        {
            return Err(BusinessError::invalid("model has invalid fields"));
        }
        let context = i32::try_from(input.context_tokens)
            .map_err(|_| BusinessError::invalid("context_tokens exceeds SQLite"))?;
        let compact = i32::try_from(input.auto_compact_tokens)
            .map_err(|_| BusinessError::invalid("auto_compact_tokens exceeds SQLite"))?;
        let max = input
            .max_output_tokens
            .map(|value| {
                i32::try_from(value)
                    .map_err(|_| BusinessError::invalid("max_output_tokens exceeds SQLite"))
            })
            .transpose()?;
        let mut connection = lock(&self.connection)?;
        let timestamp = now();
        sql_query("INSERT INTO llm_model(model_id,provider_id,display_name,request_model,context_tokens,auto_compact_tokens,max_output_tokens,supports_streaming,supports_tool_use,supports_vision,supports_structured_output,supports_cancellation,supports_reasoning_effort,enabled,sort_order,created_at,updated_at,reasoning_efforts) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?) ON CONFLICT(model_id) DO UPDATE SET provider_id=excluded.provider_id,display_name=excluded.display_name,request_model=excluded.request_model,context_tokens=excluded.context_tokens,auto_compact_tokens=excluded.auto_compact_tokens,max_output_tokens=excluded.max_output_tokens,supports_streaming=excluded.supports_streaming,supports_tool_use=excluded.supports_tool_use,supports_vision=excluded.supports_vision,supports_structured_output=excluded.supports_structured_output,supports_cancellation=excluded.supports_cancellation,supports_reasoning_effort=excluded.supports_reasoning_effort,enabled=excluded.enabled,sort_order=excluded.sort_order,reasoning_efforts=excluded.reasoning_efforts,updated_at=excluded.updated_at")
            .bind::<Text, _>(input.model_id.trim()).bind::<Text, _>(input.provider_id.trim()).bind::<Text, _>(input.display_name.trim()).bind::<Text, _>(input.request_model.trim())
            .bind::<Integer, _>(context).bind::<Integer, _>(compact).bind::<Nullable<Integer>, _>(max)
            .bind::<Integer, _>(input.supports_streaming as i32).bind::<Integer, _>(input.supports_tool_use as i32).bind::<Integer, _>(input.supports_vision as i32)
            .bind::<Integer, _>(input.supports_structured_output as i32).bind::<Integer, _>(input.supports_cancellation as i32).bind::<Integer, _>(input.supports_reasoning_effort as i32)
            .bind::<Integer, _>(input.enabled as i32).bind::<Integer, _>(input.sort_order as i32).bind::<Text, _>(&timestamp).bind::<Text, _>(&timestamp)
            .bind::<Text, _>(input.reasoning_efforts.trim()).execute(&mut *connection).map_err(crate::database_error)?;
        Ok(())
    }
}

fn to_record(row: ModelRow) -> Result<LlmModelRecord, BusinessError> {
    Ok(LlmModelRecord {
        model_id: row.model_id,
        provider_id: row.provider_id,
        display_name: row.display_name,
        request_model: row.request_model,
        context_tokens: nonnegative(row.context_tokens as i64)?,
        auto_compact_tokens: nonnegative(row.auto_compact_tokens as i64)?,
        max_output_tokens: row
            .max_output_tokens
            .map(|value| nonnegative(value as i64))
            .transpose()?,
        supports_streaming: row.supports_streaming != 0,
        supports_tool_use: row.supports_tool_use != 0,
        supports_vision: row.supports_vision != 0,
        supports_structured_output: row.supports_structured_output != 0,
        supports_cancellation: row.supports_cancellation != 0,
        supports_reasoning_effort: row.supports_reasoning_effort != 0,
        reasoning_efforts: row
            .reasoning_efforts
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .collect(),
        enabled: row.enabled != 0,
        sort_order: row.sort_order as i64,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}
