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

#[derive(QueryableByName)]
struct OptionalStringRow {
    #[diesel(sql_type = Nullable<Text>)]
    value: Option<String>,
}

impl Store {
    pub fn llm_model_providers(
        &self,
        enabled_only: bool,
    ) -> Result<Vec<LlmModelProviderRecord>, BusinessError> {
        let mut connection = lock(&self.connection)?;
        let sql = if enabled_only {
            "SELECT provider_id,display_name,endpoint,default_endpoint,adapter_type,(api_key IS NOT NULL AND length(api_key)>0) AS api_key_configured,enabled,sort_order,created_at,updated_at FROM llm_model_provider WHERE enabled=1 ORDER BY sort_order,provider_id"
        } else {
            "SELECT provider_id,display_name,endpoint,default_endpoint,adapter_type,(api_key IS NOT NULL AND length(api_key)>0) AS api_key_configured,enabled,sort_order,created_at,updated_at FROM llm_model_provider ORDER BY sort_order,provider_id"
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
        if provider.is_empty()
            || name.is_empty()
            || endpoint.is_empty()
            || default_endpoint.is_empty()
            || input.adapter_type.trim() != "openai"
            || input.sort_order < 0
        {
            return Err(BusinessError::invalid("model provider has invalid fields"));
        }
        let mut connection = lock(&self.connection)?;
        let timestamp = now();
        sql_query("INSERT INTO llm_model_provider(provider_id,display_name,endpoint,default_endpoint,adapter_type,api_key,enabled,sort_order,created_at,updated_at) VALUES (?,?,?,?,?,?,?,?,?,?) ON CONFLICT(provider_id) DO UPDATE SET display_name=excluded.display_name,endpoint=excluded.endpoint,default_endpoint=excluded.default_endpoint,adapter_type=excluded.adapter_type,enabled=excluded.enabled,sort_order=excluded.sort_order,updated_at=excluded.updated_at")
            .bind::<Text, _>(provider).bind::<Text, _>(name).bind::<Text, _>(endpoint).bind::<Text, _>(default_endpoint)
            .bind::<Text, _>("openai").bind::<Nullable<Text>, _>(None::<String>).bind::<Integer, _>(input.enabled as i32)
            .bind::<Integer, _>(input.sort_order as i32).bind::<Text, _>(&timestamp).bind::<Text, _>(&timestamp)
            .execute(&mut *connection).map_err(crate::database_error)?;
        Ok(())
    }

    pub fn llm_provider_api_key(&self, provider_id: &str) -> Result<Option<String>, BusinessError> {
        if provider_id.trim().is_empty() {
            return Err(BusinessError::invalid("model provider id is required"));
        }
        let mut connection = lock(&self.connection)?;
        Ok(sql_query(
            "SELECT api_key AS value FROM llm_model_provider WHERE provider_id=? AND enabled=1",
        )
        .bind::<Text, _>(provider_id.trim())
        .get_result::<OptionalStringRow>(&mut *connection)
        .optional()
        .map_err(crate::database_error)?
        .and_then(|row| row.value))
    }

    pub fn set_llm_provider_api_key(
        &self,
        provider_id: &str,
        value: &str,
    ) -> Result<(), BusinessError> {
        if provider_id.trim().is_empty() || value.trim().is_empty() {
            return Err(BusinessError::invalid(
                "model provider id and credential are required",
            ));
        }
        let mut connection = lock(&self.connection)?;
        let changed =
            sql_query("UPDATE llm_model_provider SET api_key=?,updated_at=? WHERE provider_id=?")
                .bind::<Text, _>(value.trim())
                .bind::<Text, _>(&now())
                .bind::<Text, _>(provider_id.trim())
                .execute(&mut *connection)
                .map_err(crate::database_error)?;
        if changed == 0 {
            Err(BusinessError::invalid("model provider does not exist"))
        } else {
            Ok(())
        }
    }

    pub fn delete_llm_provider_api_key(&self, provider_id: &str) -> Result<(), BusinessError> {
        if provider_id.trim().is_empty() {
            return Err(BusinessError::invalid("model provider id is required"));
        }
        let mut connection = lock(&self.connection)?;
        let changed = sql_query(
            "UPDATE llm_model_provider SET api_key=NULL,updated_at=? WHERE provider_id=?",
        )
        .bind::<Text, _>(&now())
        .bind::<Text, _>(provider_id.trim())
        .execute(&mut *connection)
        .map_err(crate::database_error)?;
        if changed == 0 {
            Err(BusinessError::invalid("model provider does not exist"))
        } else {
            Ok(())
        }
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
        enabled: row.enabled != 0,
        sort_order: row.sort_order as i64,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}
