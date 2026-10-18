//! Operations for `llm_model_provider`, including provider credentials.

use crate::{
    domain::{LlmModelProviderInput, LlmModelProviderRecord},
    model::ProviderRow,
    store::{lock, now, Store},
};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Integer, Nullable, Text};
use suncode_common::BusinessError;

impl Store {
    pub fn llm_model_providers(
        &self,
        enabled_only: bool,
    ) -> Result<Vec<LlmModelProviderRecord>, BusinessError> {
        let mut connection = lock(&self.connection)?;
        let sql = if enabled_only {
            "SELECT provider_id,display_name,endpoint,default_endpoint,adapter_type,(api_key IS NOT NULL OR secret_ref IS NOT NULL) AS api_key_configured,credential_mode,api_key_mask,enabled,sort_order,created_at,updated_at FROM llm_model_provider WHERE enabled=1 ORDER BY sort_order,provider_id"
        } else {
            "SELECT provider_id,display_name,endpoint,default_endpoint,adapter_type,(api_key IS NOT NULL OR secret_ref IS NOT NULL) AS api_key_configured,credential_mode,api_key_mask,enabled,sort_order,created_at,updated_at FROM llm_model_provider ORDER BY sort_order,provider_id"
        };
        sql_query(sql)
            .load::<ProviderRow>(&mut *connection)
            .map_err(crate::database_error)?
            .into_iter()
            .map(to_record)
            .collect()
    }

    pub fn upsert_llm_model_provider(
        &self,
        input: LlmModelProviderInput<'_>,
    ) -> Result<(), BusinessError> {
        let provider = input.provider_id.trim();
        let name = input.display_name.trim();
        let endpoint = input.endpoint.trim().trim_end_matches('/');
        let default_endpoint = input.default_endpoint.trim().trim_end_matches('/');
        let adapter_type = input.adapter_type.trim();
        if provider.is_empty()
            || name.is_empty()
            || endpoint.is_empty()
            || default_endpoint.is_empty()
            || !matches!(adapter_type, "openai" | "anthropic" | "chat_completions")
            || input.sort_order < 0
        {
            return Err(BusinessError::invalid("model provider has invalid fields"));
        }
        let mut connection = lock(&self.connection)?;
        let timestamp = now();
        sql_query("INSERT INTO llm_model_provider(provider_id,display_name,endpoint,default_endpoint,adapter_type,api_key,enabled,sort_order,created_at,updated_at) VALUES (?,?,?,?,?,?,?,?,?,?) ON CONFLICT(provider_id) DO UPDATE SET display_name=excluded.display_name,endpoint=excluded.endpoint,default_endpoint=excluded.default_endpoint,adapter_type=excluded.adapter_type,enabled=excluded.enabled,sort_order=excluded.sort_order,updated_at=excluded.updated_at")
            .bind::<Text, _>(provider).bind::<Text, _>(name).bind::<Text, _>(endpoint).bind::<Text, _>(default_endpoint)
            .bind::<Text, _>(adapter_type).bind::<Nullable<Text>, _>(None::<String>).bind::<Integer, _>(input.enabled as i32)
            .bind::<Integer, _>(input.sort_order as i32).bind::<Text, _>(&timestamp).bind::<Text, _>(&timestamp)
            .execute(&mut *connection).map_err(crate::database_error)?;
        Ok(())
    }
}

fn to_record(row: ProviderRow) -> Result<LlmModelProviderRecord, BusinessError> {
    Ok(LlmModelProviderRecord {
        provider_id: row.provider_id,
        display_name: row.display_name,
        endpoint: row.endpoint,
        default_endpoint: row.default_endpoint,
        adapter_type: row.adapter_type,
        api_key_configured: row.api_key_configured != 0,
        credential_mode: crate::CredentialStorageMode::parse(&row.credential_mode)?,
        api_key_mask: row.api_key_mask,
        enabled: row.enabled != 0,
        sort_order: row.sort_order as i64,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}
