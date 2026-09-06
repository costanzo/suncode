//! Operations for `session_tool_use`.

use crate::{
    domain::*,
    store::{lock, Store},
};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;
use suncode_common::BusinessError;

pub(crate) fn tool_rows(
    c: &mut diesel::sqlite::SqliteConnection,
    sql: &str,
    key: &str,
) -> Result<Vec<SessionCallToolUse>, BusinessError> {
    let rows = sql_query(sql)
        .bind::<Text, _>(key)
        .load::<crate::rows::ToolRow>(c)
        .map_err(crate::database_error)?;
    rows.into_iter()
        .map(|r| {
            Ok(SessionCallToolUse {
                turn_id: r.turn_id,
                tool_call_id: r.tool_call_id,
                session_call_id: r.session_call_id,
                name: r.name,
                request: r
                    .request_json
                    .map(|v| serde_json::from_str(&v))
                    .transpose()?,
                result: r
                    .result_json
                    .map(|v| serde_json::from_str(&v))
                    .transpose()?,
                state: r.state,
                ordinal: r.ordinal.map(i64::from),
                created_at: r.created_at,
                updated_at: r.updated_at,
                completed_at: r.completed_at,
                error_code: r.error_code,
            })
        })
        .collect()
}
pub(crate) fn load_tool_uses(
    c: &mut diesel::sqlite::SqliteConnection,
    turn_id: &str,
) -> Result<Vec<SessionCallToolUse>, BusinessError> {
    tool_rows(c, "SELECT turn_id,tool_call_id,session_call_id,name,request_json,result_json,state,ordinal,created_at,updated_at,completed_at,error_code FROM session_tool_use WHERE turn_id=? ORDER BY created_at,COALESCE(ordinal,9223372036854775807),tool_call_id", turn_id)
}
pub(crate) fn load_tool_uses_by_call(
    c: &mut diesel::sqlite::SqliteConnection,
    call_id: &str,
) -> Result<Vec<SessionCallToolUse>, BusinessError> {
    tool_rows(c, "SELECT turn_id,tool_call_id,session_call_id,name,request_json,result_json,state,ordinal,created_at,updated_at,completed_at,error_code FROM session_tool_use WHERE session_call_id=? ORDER BY COALESCE(ordinal,9223372036854775807),created_at,tool_call_id", call_id)
}

impl Store {
    pub fn session_call_tool_uses(
        &self,
        call_id: &str,
    ) -> Result<Vec<SessionCallToolUse>, BusinessError> {
        let mut c = lock(&self.connection)?;
        load_tool_uses_by_call(&mut c, call_id)
    }
}
