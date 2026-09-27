//! Operations for `session_message`.

use crate::{
    domain::*,
    store::{lock, Store},
};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Nullable, Text};
use suncode_common::BusinessError;

#[derive(QueryableByName)]
struct ContextRow {
    #[diesel(sql_type=Text)]
    kind: String,
    #[diesel(sql_type=Text)]
    payload: String,
    #[diesel(sql_type=Nullable<Text>)]
    tool_call_id: Option<String>,
}

pub(crate) fn load_messages(
    c: &mut diesel::sqlite::SqliteConnection,
    session_id: &str,
    turn_id: &str,
) -> Result<Vec<SessionCallMessage>, BusinessError> {
    let rows = sql_query("SELECT message_id,session_id,turn_id,session_call_id,role,message_json,created_at FROM session_message WHERE session_id=? AND turn_id=? AND role IN ('user','assistant','thinking') ORDER BY created_at,rowid").bind::<Text,_>(session_id).bind::<Text,_>(turn_id).load::<crate::rows::MessageRow>(c).map_err(crate::database_error)?;
    rows.into_iter()
        .map(|r| {
            Ok(SessionCallMessage {
                message_id: r.message_id,
                session_id: r.session_id,
                turn_id: r.turn_id,
                session_call_id: r.session_call_id,
                role: r.role,
                message: serde_json::from_str(&r.message_json)?,
                created_at: r.created_at,
            })
        })
        .collect()
}

pub(crate) fn load_call_messages(
    c: &mut diesel::sqlite::SqliteConnection,
    session_id: &str,
    call_id: &str,
) -> Result<Vec<SessionCallMessage>, BusinessError> {
    let rows = sql_query("SELECT message_id,session_id,turn_id,session_call_id,role,message_json,created_at FROM session_message WHERE session_id=? AND session_call_id=? AND role IN ('user','assistant','thinking') ORDER BY created_at,rowid").bind::<Text,_>(session_id).bind::<Text,_>(call_id).load::<crate::rows::MessageRow>(c).map_err(crate::database_error)?;
    rows.into_iter()
        .map(|r| {
            Ok(SessionCallMessage {
                message_id: r.message_id,
                session_id: r.session_id,
                turn_id: r.turn_id,
                session_call_id: r.session_call_id,
                role: r.role,
                message: serde_json::from_str(&r.message_json)?,
                created_at: r.created_at,
            })
        })
        .collect()
}

pub(crate) fn repair_incomplete_tool_exchanges(messages: Vec<Message>) -> Vec<Message> {
    let mut out = Vec::with_capacity(messages.len());
    let mut index = 0;
    while index < messages.len() {
        let message = &messages[index];
        if message.role == "assistant" && !message.tool_calls.is_empty() {
            let expected = message
                .tool_calls
                .iter()
                .map(|c| c.call_id.as_str())
                .collect::<Vec<_>>();
            let end = index + 1 + expected.len();
            if messages.len() < end {
                break;
            }
            let complete = messages[index + 1..end]
                .iter()
                .zip(expected.iter())
                .all(|(tool, id)| tool.role == "tool" && tool.tool_call_id.as_deref() == Some(*id));
            if !complete {
                break;
            }
            out.push(message.clone());
            out.extend_from_slice(&messages[index + 1..end]);
            index = end;
            continue;
        }
        if message.role == "tool" {
            break;
        }
        out.push(message.clone());
        index += 1;
    }
    out
}

impl Store {
    pub fn messages(&self, session_id: &str) -> Result<Vec<Message>, BusinessError> {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type=Text)]
            message_json: String,
        }
        let mut c = lock(&self.connection)?;
        let rows=sql_query("SELECT message_json FROM session_message WHERE session_id=? AND role IN ('user','assistant','thinking') ORDER BY created_at,rowid").bind::<Text,_>(session_id).load::<Row>(&mut *c).map_err(crate::database_error)?;
        rows.into_iter()
            .map(|r| Ok(serde_json::from_str(&r.message_json)?))
            .collect()
    }
    pub fn context_messages(&self, session_id: &str) -> Result<Vec<Message>, BusinessError> {
        let mut c = lock(&self.connection)?;
        #[derive(QueryableByName)]
        struct CompactionRow {
            #[diesel(sql_type=Text)]
            output: String,
        }
        let compaction: Option<CompactionRow> = sql_query("SELECT output_message_json AS output FROM session_call WHERE session_id=? AND finish_reason='context_compacted' ORDER BY rowid DESC LIMIT 1").bind::<Text,_>(session_id).get_result::<CompactionRow>(&mut *c).optional().map_err(crate::database_error)?;
        let Some(compaction) = compaction else {
            return self.load_live_context_messages(session_id, &mut c);
        };
        let value: serde_json::Value = serde_json::from_str(&compaction.output)?;
        let retained = value.get("retained_messages").cloned().ok_or_else(|| {
            BusinessError::database("compaction checkpoint is missing retained messages")
        })?;
        let mut out: Vec<Message> = serde_json::from_value(retained)?;
        let summary = value
            .get("summary")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        if !out.iter().any(|message| {
            message.role == "system" && message.text_content().contains("suncode_context_summary")
        }) {
            out.insert(
                0,
                Message::text(
                    "system",
                    serde_json::to_string(
                        &serde_json::json!({"type":"suncode_context_summary","summary":summary}),
                    )
                    .unwrap_or_default(),
                ),
            );
        }
        let message_boundary = value
            .get("message_rowid_boundary")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| {
                BusinessError::database("compaction checkpoint is missing its message boundary")
            })?;
        let tool_boundary = value
            .get("tool_rowid_boundary")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| {
                BusinessError::database("compaction checkpoint is missing its tool boundary")
            })?;
        out.extend(self.load_context_rows_after(
            session_id,
            message_boundary,
            tool_boundary,
            &mut c,
        )?);
        return Ok(repair_incomplete_tool_exchanges(out));
    }

    fn load_live_context_messages(
        &self,
        session_id: &str,
        c: &mut diesel::sqlite::SqliteConnection,
    ) -> Result<Vec<Message>, BusinessError> {
        self.load_context_rows_after(session_id, 0, 0, c)
    }

    fn load_context_rows_after(
        &self,
        session_id: &str,
        message_boundary: i64,
        tool_boundary: i64,
        c: &mut diesel::sqlite::SqliteConnection,
    ) -> Result<Vec<Message>, BusinessError> {
        let rows = sql_query("SELECT kind,payload,tool_call_id FROM (SELECT message.created_at AS occurred_at,0 AS kind_order,message.rowid AS stable_order,'message' AS kind,message.message_json AS payload,NULL AS tool_call_id,NULL AS ordinal,COALESCE(call.iteration,0) AS call_iteration FROM session_message AS message LEFT JOIN session_call AS call ON call.call_id=message.session_call_id WHERE message.session_id=? AND role IN ('user','assistant','thinking') AND message.rowid>? UNION ALL SELECT COALESCE(tool.completed_at,tool.updated_at) AS occurred_at,1 AS kind_order,tool.rowid AS stable_order,'tool' AS kind,tool.result_json AS payload,tool.tool_call_id,tool.ordinal,COALESCE(call.iteration,9223372036854775807) AS call_iteration FROM session_tool_use AS tool JOIN session_turn AS turn ON turn.turn_id=tool.turn_id LEFT JOIN session_call AS call ON call.call_id=tool.session_call_id WHERE turn.session_id=? AND tool.state IN ('succeeded','failed') AND tool.result_json IS NOT NULL AND tool.rowid>?) ORDER BY occurred_at,call_iteration,kind_order,COALESCE(ordinal,9223372036854775807),stable_order")
            .bind::<Text, _>(session_id)
            .bind::<diesel::sql_types::BigInt, _>(message_boundary)
            .bind::<Text, _>(session_id)
            .bind::<diesel::sql_types::BigInt, _>(tool_boundary)
            .load::<ContextRow>(c)
            .map_err(crate::database_error)?;
        let mut out = Vec::new();
        for r in rows {
            if r.kind == "message" {
                out.push(serde_json::from_str(&r.payload)?)
            } else {
                let mut m = Message::text("tool", r.payload);
                m.tool_call_id = r.tool_call_id;
                out.push(m)
            }
        }
        Ok(repair_incomplete_tool_exchanges(out))
    }
    pub fn session_call_messages(
        &self,
        session_id: &str,
        call_id: &str,
    ) -> Result<Vec<SessionCallMessage>, BusinessError> {
        let mut c = lock(&self.connection)?;
        load_call_messages(&mut c, session_id, call_id)
    }
}
