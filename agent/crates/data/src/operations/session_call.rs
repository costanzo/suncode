//! Operations for `session_call`.

use crate::{
    domain::*,
    rows::ExchangeRow,
    store::{lock, Store, EXCHANGE_SELECT},
};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;
use suncode_common::BusinessError;

fn from_row(row: ExchangeRow) -> Result<ProviderExchange, BusinessError> {
    Ok(ProviderExchange {
        exchange_id: row.call_id,
        session_id: row.session_id,
        turn_id: row.turn_id,
        provider: row.provider,
        model_id: row.model_id,
        wire_model: row.wire_model,
        provider_request_id: row.provider_request_id,
        provider_response_id: row.provider_response_id,
        state: row.state,
        iteration: row.iteration as i64,
        started_at: row.started_at,
        completed_at: row.completed_at,
        input_messages: serde_json::from_str(&row.input_messages_json)?,
        output_message: row
            .output_message_json
            .map(|v| serde_json::from_str(&v))
            .transpose()?,
        tool_calls: serde_json::from_str(&row.tool_calls_json)?,
        usage: row
            .usage_json
            .map(|v| serde_json::from_str(&v))
            .transpose()?,
        finish_reason: row.finish_reason,
        error: row
            .error_json
            .map(|v| serde_json::from_str(&v))
            .transpose()?,
    })
}

impl Store {
    pub fn provider_exchanges(
        &self,
        session_id: &str,
    ) -> Result<Vec<ProviderExchange>, BusinessError> {
        let mut c = lock(&self.connection)?;
        sql_query(EXCHANGE_SELECT)
            .bind::<Text, _>(session_id)
            .load::<ExchangeRow>(&mut *c)
            .map_err(crate::database_error)?
            .into_iter()
            .map(from_row)
            .collect()
    }
    pub fn provider_exchange(
        &self,
        session_id: &str,
        exchange_id: &str,
    ) -> Result<Option<ProviderExchange>, BusinessError> {
        let mut c = lock(&self.connection)?;
        sql_query("SELECT call_id,session_id,turn_id,provider,model_id,wire_model,provider_request_id,provider_response_id,state,iteration,started_at,completed_at,input_messages_json,output_message_json,tool_calls_json,usage_json,finish_reason,error_json FROM session_call WHERE session_id=? AND call_id=?").bind::<Text,_>(session_id).bind::<Text,_>(exchange_id).get_result::<ExchangeRow>(&mut *c).optional().map_err(crate::database_error)?.map(from_row).transpose()
    }
}
