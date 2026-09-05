use super::{operations, schema};
use crate::domain::*;
use crate::model::{ModelRow, ProviderRow};
use chrono::{Duration, Utc};
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Integer, Nullable, Text};
use diesel::sqlite::SqliteConnection;
use serde_json::{json, Value};
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use suncode_common::BusinessError;
use suncode_database::sqlite;
use uuid::Uuid;

#[derive(Clone)]
pub struct Store {
    pub(crate) connection: Arc<Mutex<SqliteConnection>>,
}

pub struct ApprovalInput<'a> {
    pub project_id: Option<&'a str>,
    pub session_id: &'a str,
    pub turn_id: &'a str,
    pub tool_call_id: &'a str,
    pub operation: &'a str,
    pub arguments: &'a Value,
    pub snapshot: &'a Value,
}

#[derive(QueryableByName)]
struct StringRow {
    #[diesel(sql_type = Text)]
    value: String,
}
#[derive(QueryableByName)]
struct OptionalStringRow {
    #[diesel(sql_type = Nullable<Text>)]
    value: Option<String>,
}
#[derive(QueryableByName)]
struct JournalRow {
    #[diesel(sql_type = Text)]
    journal_mode: String,
}
#[derive(QueryableByName)]
struct ProjectRow {
    #[diesel(sql_type = Text)]
    project_id: String,
    #[diesel(sql_type = Text)]
    canonical_root: String,
    #[diesel(sql_type = Text)]
    display_name: String,
    #[diesel(sql_type = Text)]
    created_at: String,
    #[diesel(sql_type = Text)]
    updated_at: String,
    #[diesel(sql_type = Text)]
    last_opened_at: String,
    #[diesel(sql_type = Nullable<Text>)]
    archived_at: Option<String>,
}
#[derive(QueryableByName)]
struct DependencyRow {
    #[diesel(sql_type = Text)]
    dependency_id: String,
    #[diesel(sql_type = Text)]
    project_id: String,
    #[diesel(sql_type = Text)]
    canonical_root: String,
    #[diesel(sql_type = Text)]
    display_name: String,
    #[diesel(sql_type = Text)]
    created_at: String,
}
#[derive(QueryableByName)]
struct SessionRow {
    #[diesel(sql_type = Text)]
    session_id: String,
    #[diesel(sql_type = Text)]
    project_id: String,
    #[diesel(sql_type = Nullable<Text>)]
    title: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    model_id: Option<String>,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Text)]
    created_at: String,
    #[diesel(sql_type = Text)]
    updated_at: String,
    #[diesel(sql_type = Text)]
    last_activity_at: String,
    #[diesel(sql_type = Nullable<Text>)]
    pin_at: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    archived_at: Option<String>,
}
#[derive(QueryableByName)]
struct SessionImageRow {
    #[diesel(sql_type = Text)]
    image_id: String,
    #[diesel(sql_type = Text)]
    session_id: String,
    #[diesel(sql_type = Text)]
    display_name: String,
    #[diesel(sql_type = Text)]
    source_kind: String,
    #[diesel(sql_type = Nullable<Text>)]
    original_path: Option<String>,
    #[diesel(sql_type = Text)]
    storage_path: String,
    #[diesel(sql_type = Text)]
    thumbnail_base64: String,
    #[diesel(sql_type = Text)]
    created_at: String,
}
#[derive(QueryableByName)]
struct TurnRow {
    #[diesel(sql_type = Text)]
    turn_id: String,
    #[diesel(sql_type = Text)]
    session_id: String,
    #[diesel(sql_type = Text)]
    state: String,
    #[diesel(sql_type = Nullable<Text>)]
    model_id: Option<String>,
    #[diesel(sql_type = Text)]
    created_at: String,
    #[diesel(sql_type = Text)]
    updated_at: String,
    #[diesel(sql_type = Nullable<Text>)]
    started_at: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    completed_at: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    error_code: Option<String>,
    #[diesel(sql_type = Integer)]
    input_tokens: i32,
    #[diesel(sql_type = Integer)]
    output_tokens: i32,
    #[diesel(sql_type = Integer)]
    total_tokens: i32,
}
#[derive(QueryableByName)]
struct SubmissionRow {
    #[diesel(sql_type = Text)]
    state: String,
    #[diesel(sql_type = Text)]
    turn_id: String,
    #[diesel(sql_type = Nullable<Text>)]
    input_json: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    model_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    response_json: Option<String>,
}
#[derive(QueryableByName)]
struct RetryInputRow {
    #[diesel(sql_type = Nullable<Text>)]
    input_json: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    model_id: Option<String>,
}
#[derive(QueryableByName)]
struct ApprovalRow {
    #[diesel(sql_type = Text)]
    approval_id: String,
    #[diesel(sql_type = Nullable<Text>)]
    project_id: Option<String>,
    #[diesel(sql_type = Text)]
    session_id: String,
    #[diesel(sql_type = Text)]
    turn_id: String,
    #[diesel(sql_type = Text)]
    tool_call_id: String,
    #[diesel(sql_type = Text)]
    operation: String,
    #[diesel(sql_type = Text)]
    arguments_json: String,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Nullable<Text>)]
    decision: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    decision_source: Option<String>,
    #[diesel(sql_type = Text)]
    created_at: String,
    #[diesel(sql_type = Text)]
    updated_at: String,
}
#[derive(QueryableByName)]
struct ManifestRow {
    #[diesel(sql_type = Text)]
    manifest_id: String,
    #[diesel(sql_type = Text)]
    session_id: String,
    #[diesel(sql_type = Nullable<Text>)]
    turn_id: Option<String>,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Text)]
    created_at: String,
    #[diesel(sql_type = Text)]
    updated_at: String,
    #[diesel(sql_type = Text)]
    expires_at: String,
    #[diesel(sql_type = Nullable<Text>)]
    restored_at: Option<String>,
}
#[derive(QueryableByName)]
struct CheckpointRow {
    #[diesel(sql_type = Text)]
    checkpoint_id: String,
    #[diesel(sql_type = Nullable<Text>)]
    manifest_id: Option<String>,
    #[diesel(sql_type = Text)]
    session_id: String,
    #[diesel(sql_type = Nullable<Text>)]
    turn_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    tool_call_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    relative_path: Option<String>,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Text)]
    created_at: String,
    #[diesel(sql_type = Nullable<Text>)]
    restored_at: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    invalidated_at: Option<String>,
    #[diesel(sql_type = Nullable<Integer>)]
    ordinal: Option<i32>,
}
#[derive(QueryableByName)]
struct ExchangeRow {
    #[diesel(sql_type = Text)]
    call_id: String,
    #[diesel(sql_type = Text)]
    session_id: String,
    #[diesel(sql_type = Text)]
    turn_id: String,
    #[diesel(sql_type = Text)]
    provider: String,
    #[diesel(sql_type = Text)]
    model_id: String,
    #[diesel(sql_type = Text)]
    wire_model: String,
    #[diesel(sql_type = Nullable<Text>)]
    provider_request_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    provider_response_id: Option<String>,
    #[diesel(sql_type = Text)]
    state: String,
    #[diesel(sql_type = Integer)]
    iteration: i32,
    #[diesel(sql_type = Text)]
    started_at: String,
    #[diesel(sql_type = Nullable<Text>)]
    completed_at: Option<String>,
    #[diesel(sql_type = Text)]
    input_messages_json: String,
    #[diesel(sql_type = Nullable<Text>)]
    output_message_json: Option<String>,
    #[diesel(sql_type = Text)]
    tool_calls_json: String,
    #[diesel(sql_type = Nullable<Text>)]
    usage_json: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    finish_reason: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    error_json: Option<String>,
}
#[derive(QueryableByName)]
struct RecoveryRow {
    #[diesel(sql_type = Text)]
    approval_id: String,
    #[diesel(sql_type = Text)]
    session_id: String,
    #[diesel(sql_type = Text)]
    turn_id: String,
    #[diesel(sql_type = Text)]
    snapshot_json: String,
    #[diesel(sql_type = Text)]
    status: String,
}

fn business_transaction<T, F>(
    connection: &mut SqliteConnection,
    operation: F,
) -> Result<T, BusinessError>
where
    F: FnOnce(&mut SqliteConnection) -> Result<T, BusinessError>,
{
    connection
        .batch_execute("BEGIN")
        .map_err(crate::database_error)?;
    match operation(connection) {
        Ok(value) => {
            connection
                .batch_execute("COMMIT")
                .map_err(crate::database_error)?;
            Ok(value)
        }
        Err(error) => {
            let _ = connection.batch_execute("ROLLBACK");
            Err(error)
        }
    }
}

impl Store {
    pub fn open(path: &Path) -> Result<Self, BusinessError> {
        sqlite::ensure_database(path).map_err(|error| BusinessError::invalid(error.to_string()))?;
        let mut connection = SqliteConnection::establish(
            path.to_str()
                .ok_or_else(|| BusinessError::invalid("database path is not valid UTF-8"))?,
        )
        .map_err(|e| BusinessError::invalid(e.to_string()))?;
        configure(&mut connection)?;
        initialize(&mut connection)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }
    pub fn open_memory() -> Result<Self, BusinessError> {
        let mut connection = SqliteConnection::establish(":memory:")
            .map_err(|e| BusinessError::invalid(e.to_string()))?;
        configure(&mut connection)?;
        initialize(&mut connection)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }
    pub fn health(&self) -> Result<Value, BusinessError> {
        let mut c = lock(&self.connection)?;
        let mode = sql_query("PRAGMA journal_mode")
            .get_result::<JournalRow>(&mut *c)
            .map_err(crate::database_error)?
            .journal_mode;
        Ok(json!({"ok":true,"journal_mode":mode}))
    }
    pub fn append_content(
        &self,
        session_id: &str,
        event_type: &str,
        payload: &Value,
    ) -> Result<SessionEvent, BusinessError> {
        let mut c = lock(&self.connection)?;
        let occurred = now();
        business_transaction(&mut c, |c| {
            operations::projection::apply(c, session_id, &occurred, event_type, payload)?;
            sql_query("UPDATE session SET updated_at=?,last_activity_at=? WHERE session_id=?")
                .bind::<Text, _>(&occurred)
                .bind::<Text, _>(&occurred)
                .bind::<Text, _>(session_id)
                .execute(c)
                .map_err(crate::database_error)?;
            Ok::<_, BusinessError>(())
        })?;
        Ok(SessionEvent {
            session_id: session_id.into(),
            occurred_at: occurred,
            event_type: event_type.into(),
            payload: payload.clone(),
        })
    }
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
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type=Text)]
            kind: String,
            #[diesel(sql_type=Text)]
            payload: String,
            #[diesel(sql_type=Nullable<Text>)]
            tool_call_id: Option<String>,
        }
        let mut c = lock(&self.connection)?;
        let rows=sql_query("SELECT kind,payload,tool_call_id FROM (SELECT message.created_at AS occurred_at,0 AS kind_order,message.rowid AS stable_order,'message' AS kind,message.message_json AS payload,NULL AS tool_call_id,NULL AS ordinal,COALESCE(call.iteration,0) AS call_iteration FROM session_message AS message LEFT JOIN session_call AS call ON call.call_id=message.session_call_id WHERE message.session_id=? AND role IN ('user','assistant','thinking') UNION ALL SELECT COALESCE(tool.completed_at,tool.updated_at) AS occurred_at,1 AS kind_order,tool.rowid AS stable_order,'tool' AS kind,tool.result_json AS payload,tool.tool_call_id,tool.ordinal,COALESCE(call.iteration,9223372036854775807) AS call_iteration FROM session_tool_use AS tool JOIN session_turn AS turn ON turn.turn_id=tool.turn_id LEFT JOIN session_call AS call ON call.call_id=tool.session_call_id WHERE turn.session_id=? AND tool.state IN ('succeeded','failed') AND tool.result_json IS NOT NULL) ORDER BY occurred_at,call_iteration,kind_order,COALESCE(ordinal,9223372036854775807),stable_order").bind::<Text,_>(session_id).bind::<Text,_>(session_id).load::<Row>(&mut *c).map_err(crate::database_error)?;
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
    pub fn session_usage(&self, session_id: &str) -> Result<Usage, BusinessError> {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type=BigInt)]
            input_tokens: i64,
            #[diesel(sql_type=BigInt)]
            output_tokens: i64,
            #[diesel(sql_type=BigInt)]
            total_tokens: i64,
        }
        let mut c = lock(&self.connection)?;
        let r=sql_query("SELECT COALESCE(SUM(input_tokens),0) AS input_tokens,COALESCE(SUM(output_tokens),0) AS output_tokens,COALESCE(SUM(total_tokens),0) AS total_tokens FROM session_turn WHERE session_id=?").bind::<Text,_>(session_id).get_result::<Row>(&mut *c).map_err(crate::database_error)?;
        Ok(Usage {
            input_tokens: nonnegative(r.input_tokens)?,
            output_tokens: nonnegative(r.output_tokens)?,
            total_tokens: nonnegative(r.total_tokens)?,
        })
    }
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
            .map(exchange_from_row)
            .collect()
    }
    pub fn provider_exchange(
        &self,
        session_id: &str,
        exchange_id: &str,
    ) -> Result<Option<ProviderExchange>, BusinessError> {
        let mut c = lock(&self.connection)?;
        sql_query("SELECT call_id,session_id,turn_id,provider,model_id,wire_model,provider_request_id,provider_response_id,state,iteration,started_at,completed_at,input_messages_json,output_message_json,tool_calls_json,usage_json,finish_reason,error_json FROM session_call WHERE session_id=? AND call_id=?").bind::<Text,_>(session_id).bind::<Text,_>(exchange_id).get_result::<ExchangeRow>(&mut *c).optional().map_err(crate::database_error)?.map(exchange_from_row).transpose()
    }
    pub fn session_trace_turns(
        &self,
        session_id: &str,
    ) -> Result<Vec<SessionTraceTurn>, BusinessError> {
        let mut c = lock(&self.connection)?;
        sql_query("SELECT turn_id,session_id,state,model_id,created_at,updated_at,started_at,completed_at,error_code,input_tokens,output_tokens,total_tokens FROM session_turn WHERE session_id=? ORDER BY created_at DESC,turn_id DESC").bind::<Text,_>(session_id).load::<TurnRow>(&mut *c).map_err(crate::database_error)?.into_iter().map(trace_from_row).collect()
    }
    pub fn session_conversation_turns(
        &self,
        session_id: &str,
    ) -> Result<Vec<SessionConversationTurn>, BusinessError> {
        let mut c = lock(&self.connection)?;
        let turns=sql_query("SELECT turn_id,session_id,state,model_id,created_at,updated_at,started_at,completed_at,error_code,input_tokens,output_tokens,total_tokens FROM session_turn WHERE session_id=? ORDER BY created_at,turn_id").bind::<Text,_>(session_id).load::<TurnRow>(&mut *c).map_err(crate::database_error)?;
        let mut out = Vec::new();
        for t in turns {
            out.push(SessionConversationTurn {
                turn_id: t.turn_id.clone(),
                state: t.state,
                created_at: t.created_at,
                messages: load_messages(&mut c, session_id, &t.turn_id)?,
                tool_uses: load_tool_uses(&mut c, &t.turn_id)?,
                todos: load_todos(&mut c, &t.turn_id)?,
            })
        }
        Ok(out)
    }
    pub fn session_images(
        &self,
        session_id: &str,
    ) -> Result<Vec<SessionImageRecord>, BusinessError> {
        let mut c = lock(&self.connection)?;
        sql_query("SELECT image_id,session_id,display_name,source_kind,original_path,storage_path,thumbnail_base64,created_at FROM session_image WHERE session_id=? ORDER BY created_at,image_id")
            .bind::<Text, _>(session_id)
            .load::<SessionImageRow>(&mut *c)
            .map_err(crate::database_error)?
            .into_iter()
            .map(session_image_from_row)
            .collect()
    }
    pub fn session_image_by_id(
        &self,
        session_id: &str,
        image_id: &str,
    ) -> Result<Option<SessionImageRecord>, BusinessError> {
        let mut c = lock(&self.connection)?;
        operations::session_image::by_id(&mut c, session_id, image_id)
    }
    pub fn insert_session_image(
        &self,
        image_id: &str,
        session_id: &str,
        display_name: &str,
        source_kind: &str,
        original_path: Option<&str>,
        storage_path: &Path,
        thumbnail_base64: &str,
    ) -> Result<SessionImageRecord, BusinessError> {
        let display_name = display_name.trim();
        let source_kind = source_kind.trim();
        let thumbnail_base64 = thumbnail_base64.trim();
        if display_name.is_empty()
            || thumbnail_base64.is_empty()
            || !matches!(source_kind, "file" | "clipboard")
        {
            return Err(BusinessError::invalid("session image has invalid fields"));
        }
        let storage_path = storage_path
            .to_str()
            .ok_or_else(|| BusinessError::invalid("image storage path is not valid UTF-8"))?;
        let session = self
            .session_by_id(session_id)?
            .ok_or_else(|| BusinessError::missing("session"))?;
        let project_id = session.project_id.unwrap_or_default();
        let created_at = now();
        let mut c = lock(&self.connection)?;
        business_transaction(&mut c, |c| {
            sql_query("INSERT INTO session_image(image_id,session_id,display_name,source_kind,original_path,storage_path,thumbnail_base64,created_at) VALUES (?,?,?,?,?,?,?,?)")
                .bind::<Text, _>(image_id)
                .bind::<Text, _>(session_id)
                .bind::<Text, _>(display_name)
                .bind::<Text, _>(source_kind)
                .bind::<Nullable<Text>, _>(original_path)
                .bind::<Text, _>(storage_path)
                .bind::<Text, _>(thumbnail_base64)
                .bind::<Text, _>(&created_at)
                .execute(c)
                .map_err(crate::database_error)?;
            sql_query("UPDATE session SET updated_at=?,last_activity_at=? WHERE session_id=?")
                .bind::<Text, _>(&created_at)
                .bind::<Text, _>(&created_at)
                .bind::<Text, _>(session_id)
                .execute(c)
                .map_err(crate::database_error)?;
            if !project_id.is_empty() {
                sql_query("UPDATE project SET updated_at=?,last_opened_at=? WHERE project_id=?")
                    .bind::<Text, _>(&created_at)
                    .bind::<Text, _>(&created_at)
                    .bind::<Text, _>(&project_id)
                    .execute(c)
                    .map_err(crate::database_error)?;
            }
            Ok(())
        })?;
        drop(c);
        self.session_image_by_id(session_id, image_id)?
            .ok_or_else(|| BusinessError::invalid("session image insertion failed"))
    }
    pub fn remove_session_image(
        &self,
        session_id: &str,
        image_id: &str,
    ) -> Result<Option<SessionImageRecord>, BusinessError> {
        let mut c = lock(&self.connection)?;
        let existing = operations::session_image::by_id(&mut c, session_id, image_id)?;
        if existing.is_none() {
            return Ok(None);
        }
        sql_query("DELETE FROM session_image WHERE session_id=? AND image_id=?")
            .bind::<Text, _>(session_id)
            .bind::<Text, _>(image_id)
            .execute(&mut *c)
            .map_err(crate::database_error)?;
        Ok(existing)
    }
    pub fn session_call_messages(
        &self,
        session_id: &str,
        call_id: &str,
    ) -> Result<Vec<SessionCallMessage>, BusinessError> {
        let mut c = lock(&self.connection)?;
        load_call_messages(&mut c, session_id, call_id)
    }
    pub fn session_call_tool_uses(
        &self,
        call_id: &str,
    ) -> Result<Vec<SessionCallToolUse>, BusinessError> {
        let mut c = lock(&self.connection)?;
        load_tool_uses_by_call(&mut c, call_id)
    }
    pub fn begin_turn(
        &self,
        session_id: &str,
        key: &str,
        input: &str,
        model: &str,
    ) -> Result<TurnAdmission, BusinessError> {
        self.begin_turn_with_images(session_id, key, input, model, &[])
    }

    pub fn begin_turn_with_images(
        &self,
        session_id: &str,
        key: &str,
        input: &str,
        model: &str,
        image_ids: &[String],
    ) -> Result<TurnAdmission, BusinessError> {
        let mut c = lock(&self.connection)?;
        let old=sql_query("SELECT state,turn_id,input_json,model_id,response_json FROM session_turn WHERE session_id=? AND submission_idempotency_key=?").bind::<Text,_>(session_id).bind::<Text,_>(key).get_result::<SubmissionRow>(&mut *c).optional().map_err(crate::database_error)?;
        if let Some(r) = old {
            let requested = serde_json::to_string(&json!({"input":input,"image_ids":image_ids}))?;
            if r.input_json.as_deref() != Some(&requested) || r.model_id.as_deref() != Some(model) {
                return Err(BusinessError::invalid(
                    "idempotency key was reused with different turn input",
                ));
            }
            return Ok(TurnAdmission {
                created: false,
                turn_id: r.turn_id,
                status: r.state,
                response: r
                    .response_json
                    .map(|v| serde_json::from_str(&v))
                    .transpose()?,
            });
        }
        let s = session_by_id_conn(&mut c, session_id)?
            .ok_or_else(|| BusinessError::invalid("session not found"))?;
        if s.status != "active" {
            return Err(BusinessError::invalid("session is archived"));
        }
        let id = Uuid::new_v4().to_string();
        let t = now();
        let input_json = serde_json::to_string(&json!({"input":input,"image_ids":image_ids}))?;
        sql_query("INSERT INTO session_turn(session_id,submission_idempotency_key,state,created_at,updated_at,turn_id,input_json,model_id,admitted_at) VALUES (?,?, 'admitted',?,?,?,?,?,?)").bind::<Text,_>(session_id).bind::<Text,_>(key).bind::<Text,_>(&t).bind::<Text,_>(&t).bind::<Text,_>(&id).bind::<Text,_>(&input_json).bind::<Text,_>(model).bind::<Text,_>(&t).execute(&mut *c).map_err(crate::database_error)?;
        Ok(TurnAdmission {
            created: true,
            turn_id: id,
            status: "pending".into(),
            response: None,
        })
    }

    /// Return the most recently failed turn's original input and image attachments.
    /// The caller is responsible for submitting it with a fresh idempotency key.
    pub fn latest_failed_turn_input(
        &self,
        session_id: &str,
    ) -> Result<Option<(String, String, Vec<String>)>, BusinessError> {
        let mut c = lock(&self.connection)?;
        let Some(row) = sql_query(
            "SELECT input_json, model_id FROM session_turn WHERE session_id=? AND state='failed' ORDER BY created_at DESC, turn_id DESC LIMIT 1",
        )
        .bind::<Text, _>(session_id)
        .get_result::<RetryInputRow>(&mut *c)
        .optional()
        .map_err(crate::database_error)? else {
            return Ok(None);
        };
        let Some(input_json) = row.input_json else {
            return Ok(None);
        };
        let value: Value = serde_json::from_str(&input_json)
            .map_err(|_| BusinessError::invalid("stored turn input is invalid"))?;
        let input = value
            .get("input")
            .and_then(Value::as_str)
            .ok_or_else(|| BusinessError::invalid("stored turn input is invalid"))?
            .to_string();
        let image_ids = value
            .get("image_ids")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        Ok(Some((input, row.model_id.unwrap_or_default(), image_ids)))
    }
    pub fn mark_turn_started(&self, session_id: &str, key: &str) -> Result<(), BusinessError> {
        let mut c = lock(&self.connection)?;
        let t = now();
        sql_query("UPDATE session_turn SET started_at=COALESCE(started_at,?),updated_at=? WHERE session_id=? AND submission_idempotency_key=? AND state NOT IN ('completed','failed','cancelled','interrupted')").bind::<Text,_>(&t).bind::<Text,_>(&t).bind::<Text,_>(session_id).bind::<Text,_>(key).execute(&mut *c).map_err(crate::database_error)?;
        Ok(())
    }
    pub fn complete_turn(
        &self,
        session_id: &str,
        key: &str,
        response: &Value,
    ) -> Result<(), BusinessError> {
        let mut c = lock(&self.connection)?;
        let t = now();
        let value = serde_json::to_string(response)?;
        sql_query("UPDATE session_turn SET state='completed',response_json=?,error_json=NULL,completed_at=?,updated_at=? WHERE session_id=? AND submission_idempotency_key=? AND state NOT IN ('completed','failed','cancelled','interrupted')").bind::<Text,_>(&value).bind::<Text,_>(&t).bind::<Text,_>(&t).bind::<Text,_>(session_id).bind::<Text,_>(key).execute(&mut *c).map_err(crate::database_error)?;
        Ok(())
    }
    pub fn fail_turn(
        &self,
        session_id: &str,
        key: &str,
        error: &Value,
    ) -> Result<(), BusinessError> {
        let mut c = lock(&self.connection)?;
        let t = now();
        let value = serde_json::to_string(error)?;
        let code = error.get("code").and_then(Value::as_str);
        sql_query("UPDATE session_turn SET state='failed',error_json=?,error_code=COALESCE(?,error_code),completed_at=COALESCE(completed_at,?),updated_at=? WHERE session_id=? AND submission_idempotency_key=? AND state NOT IN ('completed','cancelled','interrupted')").bind::<Text,_>(&value).bind::<Nullable<Text>,_>(code).bind::<Text,_>(&t).bind::<Text,_>(&t).bind::<Text,_>(session_id).bind::<Text,_>(key).execute(&mut *c).map_err(crate::database_error)?;
        Ok(())
    }
    pub fn create_approval(
        &self,
        input: ApprovalInput<'_>,
    ) -> Result<ApprovalRecord, BusinessError> {
        let mut c = lock(&self.connection)?;
        let key = format!(
            "{}:{}:{}:approval",
            input.session_id, input.turn_id, input.tool_call_id
        );
        if let Some(row) =
            sql_query("SELECT approval_id AS value FROM approval_request WHERE idempotency_key=?")
                .bind::<Text, _>(&key)
                .get_result::<StringRow>(&mut *c)
                .optional()
                .map_err(crate::database_error)?
        {
            return approval_by_id(&mut c, &row.value)?
                .ok_or_else(|| BusinessError::invalid("approval disappeared"));
        }
        let id = Uuid::new_v4().to_string();
        let t = now();
        let args = serde_json::to_string(input.arguments)?;
        let snapshot = serde_json::to_string(input.snapshot)?;
        business_transaction(&mut c, |c| {
            sql_query("INSERT INTO approval_request(approval_id,project_id,session_id,turn_id,tool_call_id,operation,arguments_json,idempotency_key,status,created_at,updated_at) VALUES (?,?,?,?,?,?,?,?, 'pending',?,?)").bind::<Text,_>(&id).bind::<Nullable<Text>,_>(input.project_id).bind::<Text,_>(input.session_id).bind::<Text,_>(input.turn_id).bind::<Text,_>(input.tool_call_id).bind::<Text,_>(input.operation).bind::<Text,_>(&args).bind::<Text,_>(&key).bind::<Text,_>(&t).bind::<Text,_>(&t).execute(c).map_err(crate::database_error)?;
            sql_query("UPDATE session_turn SET recovery_approval_id=?,recovery_snapshot_json=?,recovery_status='pending',recovery_created_at=?,recovery_updated_at=? WHERE turn_id=?").bind::<Text,_>(&id).bind::<Text,_>(&snapshot).bind::<Text,_>(&t).bind::<Text,_>(&t).bind::<Text,_>(input.turn_id).execute(c).map_err(crate::database_error)?;
            Ok(())
        })?;
        approval_by_id(&mut c, &id)?
            .ok_or_else(|| BusinessError::invalid("approval creation failed"))
    }
    pub fn approval(&self, id: &str) -> Result<Option<ApprovalRecord>, BusinessError> {
        let mut c = lock(&self.connection)?;
        approval_by_id(&mut c, id)
    }
    pub fn create_question(
        &self,
        request_id: &str,
        turn_id: &str,
        snapshot: &Value,
    ) -> Result<(), BusinessError> {
        let mut c = lock(&self.connection)?;
        let t = now();
        let value = serde_json::to_string(snapshot)?;
        let changed=sql_query("UPDATE session_turn SET recovery_approval_id=?,recovery_snapshot_json=?,recovery_status='pending',recovery_created_at=?,recovery_updated_at=? WHERE turn_id=? AND (recovery_status IS NULL OR recovery_status='resuming')").bind::<Text,_>(request_id).bind::<Text,_>(&value).bind::<Text,_>(&t).bind::<Text,_>(&t).bind::<Text,_>(turn_id).execute(&mut *c).map_err(crate::database_error)?;
        if changed == 0 {
            Err(BusinessError::invalid(
                "question recovery is already pending or turn is unavailable",
            ))
        } else {
            Ok(())
        }
    }
    pub fn pending_question(&self, session_id: &str) -> Result<Option<Value>, BusinessError> {
        let mut c = lock(&self.connection)?;
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type=Text)]
            request_id: String,
            #[diesel(sql_type=Text)]
            snapshot: String,
        }
        let Some(row)=sql_query("SELECT recovery_approval_id AS request_id,recovery_snapshot_json AS snapshot FROM session_turn WHERE session_id=? AND recovery_status='pending'").bind::<Text,_>(session_id).get_result::<Row>(&mut *c).optional().map_err(crate::database_error)? else{return Ok(None)};
        let snapshot: Value = serde_json::from_str(&row.snapshot)?;
        let Some(call) = snapshot
            .get("pending_call")
            .filter(|v| v.get("name").and_then(Value::as_str) == Some("question"))
        else {
            return Ok(None);
        };
        Ok(Some(
            json!({"request_id":row.request_id,"session_id":session_id,"turn_id":snapshot.get("turn_id"),"tool_call_id":call.get("call_id"),"questions":call.get("arguments").and_then(|v|v.get("questions")).cloned().unwrap_or_else(||json!([]))}),
        ))
    }
    pub fn question_snapshot(&self, request_id: &str) -> Result<Option<Value>, BusinessError> {
        let mut c = lock(&self.connection)?;
        sql_query("SELECT recovery_snapshot_json AS value FROM session_turn WHERE recovery_approval_id=? AND recovery_status='pending'").bind::<Text,_>(request_id).get_result::<StringRow>(&mut *c).optional().map_err(crate::database_error)?.map(|r|Ok(serde_json::from_str(&r.value)?)).transpose()
    }
    pub fn resolve_question(
        &self,
        id: &str,
        answers: &[Vec<String>],
        rejected: bool,
    ) -> Result<Option<SuspendedTurn>, BusinessError> {
        let mut c = lock(&self.connection)?;
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type=Text)]
            approval_id: String,
            #[diesel(sql_type=Text)]
            session_id: String,
            #[diesel(sql_type=Text)]
            turn_id: String,
            #[diesel(sql_type=Text)]
            snapshot: String,
        }
        let Some(row)=sql_query("SELECT recovery_approval_id AS approval_id,session_id,turn_id,recovery_snapshot_json AS snapshot FROM session_turn WHERE recovery_approval_id=? AND recovery_status='pending'").bind::<Text,_>(id).get_result::<Row>(&mut *c).optional().map_err(crate::database_error)? else{return Ok(None)};
        let mut snapshot: Value = serde_json::from_str(&row.snapshot)?;
        snapshot["question_answers"] = serde_json::to_value(answers)?;
        snapshot["question_rejected"] = json!(rejected);
        let value = serde_json::to_string(&snapshot)?;
        sql_query("UPDATE session_turn SET recovery_status='resuming',recovery_snapshot_json=?,recovery_updated_at=? WHERE recovery_approval_id=? AND recovery_status='pending'").bind::<Text,_>(&value).bind::<Text,_>(&now()).bind::<Text,_>(id).execute(&mut *c).map_err(crate::database_error)?;
        Ok(Some(SuspendedTurn {
            approval_id: row.approval_id,
            session_id: row.session_id,
            turn_id: row.turn_id,
            snapshot,
            status: "resuming".into(),
        }))
    }
    pub fn resolve_approval(
        &self,
        id: &str,
        decision: &str,
    ) -> Result<Option<SuspendedTurn>, BusinessError> {
        let mut c = lock(&self.connection)?;
        let approved = decision != "deny";
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type=Text)]
            approval_id: String,
            #[diesel(sql_type=Text)]
            session_id: String,
            #[diesel(sql_type=Text)]
            turn_id: String,
            #[diesel(sql_type=Text)]
            snapshot: String,
        }
        let result = business_transaction(&mut c, |c| {
            let n=sql_query("UPDATE approval_request SET status=?,decision=?,decision_source='user',updated_at=? WHERE approval_id=? AND status='pending'").bind::<Text,_>(if approved{"approved"}else{"denied"}).bind::<Text,_>(decision).bind::<Text,_>(&now()).bind::<Text,_>(id).execute(c).map_err(crate::database_error)?;
            if n == 0 {
                return Ok(None);
            }
            let row=sql_query("SELECT recovery_approval_id AS approval_id,session_id,turn_id,recovery_snapshot_json AS snapshot FROM session_turn WHERE recovery_approval_id=? AND recovery_status='pending'").bind::<Text,_>(id).get_result::<Row>(c).map_err(crate::database_error)?;
            if decision == "allow_session" {
                sql_query("INSERT INTO configuration(scope,session_id,key,value_json,updated_at) VALUES ('session',?,'full_control','true',?) ON CONFLICT(session_id,key) WHERE scope='session' DO UPDATE SET value_json='true',updated_at=excluded.updated_at").bind::<Text,_>(&row.session_id).bind::<Text,_>(&now()).execute(c).map_err(crate::database_error)?;
            }
            sql_query("UPDATE session_turn SET recovery_status=?,recovery_updated_at=? WHERE recovery_approval_id=?").bind::<Text,_>(if approved{"resuming"}else{"denied"}).bind::<Text,_>(&now()).bind::<Text,_>(id).execute(c).map_err(crate::database_error)?;
            Ok(Some(row))
        })?;
        result
            .map(|r| {
                Ok(SuspendedTurn {
                    approval_id: r.approval_id,
                    session_id: r.session_id,
                    turn_id: r.turn_id,
                    snapshot: serde_json::from_str(&r.snapshot)?,
                    status: if approved {
                        "resuming".into()
                    } else {
                        "denied".into()
                    },
                })
            })
            .transpose()
    }
    pub fn finish_suspended(&self, id: &str, status: &str) -> Result<(), BusinessError> {
        let mut c = lock(&self.connection)?;
        let terminal = matches!(status, "completed" | "denied" | "failed");
        sql_query("UPDATE session_turn SET recovery_status=?,recovery_updated_at=?,recovery_snapshot_json=CASE WHEN ? THEN '{}' ELSE recovery_snapshot_json END WHERE recovery_approval_id=?").bind::<Text,_>(status).bind::<Text,_>(&now()).bind::<Integer,_>(terminal as i32).bind::<Text,_>(id).execute(&mut *c).map_err(crate::database_error)?;
        Ok(())
    }
    pub fn ensure_manifest(
        &self,
        session_id: &str,
        turn_id: &str,
    ) -> Result<CheckpointManifest, BusinessError> {
        let mut c = lock(&self.connection)?;
        if let Some(v) = manifest_by_turn(&mut c, turn_id)? {
            return Ok(v);
        }
        let id = Uuid::new_v4().to_string();
        let t = now();
        let expires =
            (Utc::now() + Duration::days(30)).to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        sql_query("INSERT INTO checkpoint_manifest(manifest_id,session_id,turn_id,status,created_at,updated_at,expires_at) VALUES (?,?,?,'available',?,?,?)").bind::<Text,_>(&id).bind::<Text,_>(session_id).bind::<Text,_>(turn_id).bind::<Text,_>(&t).bind::<Text,_>(&t).bind::<Text,_>(&expires).execute(&mut *c).map_err(crate::database_error)?;
        manifest_by_turn(&mut c, turn_id)?
            .ok_or_else(|| BusinessError::invalid("checkpoint manifest creation failed"))
    }
    pub fn manifests(&self, session_id: &str) -> Result<Vec<CheckpointManifest>, BusinessError> {
        let mut c = lock(&self.connection)?;
        let t = now();
        sql_query("UPDATE checkpoint_manifest SET status='expired',updated_at=? WHERE session_id=? AND status='available' AND expires_at<=?").bind::<Text,_>(&t).bind::<Text,_>(session_id).bind::<Text,_>(&t).execute(&mut *c).map_err(crate::database_error)?;
        sql_query(MANIFEST_SELECT)
            .bind::<Text, _>(session_id)
            .load::<ManifestRow>(&mut *c)
            .map_err(crate::database_error)?
            .into_iter()
            .map(manifest_from_row)
            .collect()
    }
    pub fn manifest(&self, id: &str) -> Result<Option<CheckpointManifest>, BusinessError> {
        let mut c = lock(&self.connection)?;
        sql_query("SELECT manifest_id,session_id,turn_id,status,created_at,updated_at,expires_at,restored_at FROM checkpoint_manifest WHERE manifest_id=?").bind::<Text,_>(id).get_result::<ManifestRow>(&mut *c).optional().map_err(crate::database_error)?.map(manifest_from_row).transpose()
    }
    pub fn checkpoint_items(
        &self,
        manifest_id: &str,
    ) -> Result<Vec<CheckpointItem>, BusinessError> {
        let mut c = lock(&self.connection)?;
        sql_query("SELECT checkpoint_id,manifest_id,session_id,turn_id,tool_call_id,relative_path,status,created_at,restored_at,invalidated_at,ordinal FROM checkpoint WHERE manifest_id=? ORDER BY ordinal DESC").bind::<Text,_>(manifest_id).load::<CheckpointRow>(&mut *c).map_err(crate::database_error)?.into_iter().map(checkpoint_from_row).collect()
    }
    pub fn set_manifest_status(&self, id: &str, status: &str) -> Result<(), BusinessError> {
        let mut c = lock(&self.connection)?;
        let t = now();
        sql_query("UPDATE checkpoint_manifest SET status=?,updated_at=?,restored_at=CASE WHEN ?='restored' THEN ? ELSE restored_at END WHERE manifest_id=?").bind::<Text,_>(status).bind::<Text,_>(&t).bind::<Text,_>(status).bind::<Text,_>(&t).bind::<Text,_>(id).execute(&mut *c).map_err(crate::database_error)?;
        Ok(())
    }

    pub fn settings(
        &self,
        project_id: Option<&str>,
        session_id: Option<&str>,
    ) -> Result<Vec<SettingRecord>, BusinessError> {
        let mut c = lock(&self.connection)?;
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type=Text)]
            scope: String,
            #[diesel(sql_type=Text)]
            scope_id: String,
            #[diesel(sql_type=Text)]
            key: String,
            #[diesel(sql_type=Text)]
            value_json: String,
        }
        let rows=sql_query("SELECT scope,COALESCE(project_id,session_id,'global') AS scope_id,key,value_json FROM configuration WHERE scope='global' OR (scope='project' AND project_id=?) OR (scope='session' AND session_id=?) ORDER BY CASE scope WHEN 'global' THEN 0 WHEN 'project' THEN 1 ELSE 2 END,key").bind::<Nullable<Text>,_>(project_id).bind::<Nullable<Text>,_>(session_id).load::<Row>(&mut *c).map_err(crate::database_error)?;
        let mut out = Vec::new();
        for r in rows {
            out.retain(|v: &SettingRecord| v.key != r.key);
            out.push(SettingRecord {
                key: r.key,
                value: serde_json::from_str(&r.value_json)?,
                scope: r.scope,
                scope_id: r.scope_id,
            })
        }
        Ok(out)
    }
    pub fn session_full_control(&self, session_id: &str) -> Result<bool, BusinessError> {
        let mut c = lock(&self.connection)?;
        match sql_query("SELECT value_json AS value FROM configuration WHERE scope='session' AND session_id=? AND key='full_control'").bind::<Text,_>(session_id).get_result::<StringRow>(&mut *c).optional().map_err(crate::database_error)?{None=>Ok(false),Some(v)=>serde_json::from_str::<Value>(&v.value)?.as_bool().ok_or_else(||BusinessError::invalid("session full_control configuration must be a boolean"))}
    }
    pub fn project_default_model(&self, project_id: &str) -> Result<Option<String>, BusinessError> {
        let value = self
            .settings(Some(project_id), None)?
            .into_iter()
            .find(|r| r.key == "default_model")
            .map(|r| r.value);
        let Some(value) = value else { return Ok(None) };
        let model = value
            .as_str()
            .ok_or_else(|| BusinessError::invalid("project default_model must be a string"))?;
        if model.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(model.trim().into()))
        }
    }
    pub fn project_tool_call_limit(&self, project_id: &str) -> Result<Option<u32>, BusinessError> {
        let mut c = lock(&self.connection)?;
        let Some(v)=sql_query("SELECT value_json AS value FROM configuration WHERE scope='project' AND project_id=? AND key='tool_call_limit'").bind::<Text,_>(project_id).get_result::<StringRow>(&mut *c).optional().map_err(crate::database_error)? else{return Ok(None)};
        let n = serde_json::from_str::<Value>(&v.value)?
            .as_u64()
            .ok_or_else(|| {
                BusinessError::invalid(
                    "project tool_call_limit must be an integer between 1 and 256",
                )
            })?;
        if !(1..=256).contains(&n) {
            return Err(BusinessError::invalid(
                "project tool_call_limit must be an integer between 1 and 256",
            ));
        }
        Ok(Some(n as u32))
    }
    pub fn set_setting(
        &self,
        scope: &str,
        scope_id: &str,
        key: &str,
        value: &Value,
    ) -> Result<(), BusinessError> {
        if !matches!(scope, "global" | "project" | "session") || key.trim().is_empty() {
            return Err(BusinessError::invalid(
                "setting scope, scope id, and key are required",
            ));
        }
        let mut c = lock(&self.connection)?;
        let v = serde_json::to_string(value)?;
        let t = now();
        match scope {
            "global" => {
                sql_query("INSERT INTO configuration(scope,key,value_json,updated_at) VALUES ('global',?,?,?) ON CONFLICT(key) WHERE scope='global' DO UPDATE SET value_json=excluded.value_json,updated_at=excluded.updated_at").bind::<Text,_>(key.trim()).bind::<Text,_>(&v).bind::<Text,_>(&t).execute(&mut *c).map_err(crate::database_error)?;
            }
            "project" => {
                sql_query("INSERT INTO configuration(scope,project_id,key,value_json,updated_at) VALUES ('project',?,?,?,?) ON CONFLICT(project_id,key) WHERE scope='project' DO UPDATE SET value_json=excluded.value_json,updated_at=excluded.updated_at").bind::<Text,_>(scope_id).bind::<Text,_>(key.trim()).bind::<Text,_>(&v).bind::<Text,_>(&t).execute(&mut *c).map_err(crate::database_error)?;
            }
            "session" => {
                sql_query("INSERT INTO configuration(scope,session_id,key,value_json,updated_at) VALUES ('session',?,?,?,?) ON CONFLICT(session_id,key) WHERE scope='session' DO UPDATE SET value_json=excluded.value_json,updated_at=excluded.updated_at").bind::<Text,_>(scope_id).bind::<Text,_>(key.trim()).bind::<Text,_>(&v).bind::<Text,_>(&t).execute(&mut *c).map_err(crate::database_error)?;
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    pub fn llm_model_providers(
        &self,
        enabled_only: bool,
    ) -> Result<Vec<LlmModelProviderRecord>, BusinessError> {
        let mut c = lock(&self.connection)?;
        let sql = if enabled_only {
            "SELECT provider_id,display_name,endpoint,default_endpoint,adapter_type,(api_key IS NOT NULL AND length(api_key)>0) AS api_key_configured,enabled,sort_order,created_at,updated_at FROM llm_model_provider WHERE enabled=1 ORDER BY sort_order,provider_id"
        } else {
            "SELECT provider_id,display_name,endpoint,default_endpoint,adapter_type,(api_key IS NOT NULL AND length(api_key)>0) AS api_key_configured,enabled,sort_order,created_at,updated_at FROM llm_model_provider ORDER BY sort_order,provider_id"
        };
        sql_query(sql)
            .load::<ProviderRow>(&mut *c)
            .map_err(crate::database_error)?
            .into_iter()
            .map(provider_from_row)
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
        let mut c = lock(&self.connection)?;
        let t = now();
        sql_query("INSERT INTO llm_model_provider(provider_id,display_name,endpoint,default_endpoint,adapter_type,api_key,enabled,sort_order,created_at,updated_at) VALUES (?,?,?,?,?,?,?,?,?,?) ON CONFLICT(provider_id) DO UPDATE SET display_name=excluded.display_name,endpoint=excluded.endpoint,default_endpoint=excluded.default_endpoint,adapter_type=excluded.adapter_type,enabled=excluded.enabled,sort_order=excluded.sort_order,updated_at=excluded.updated_at").bind::<Text,_>(provider).bind::<Text,_>(name).bind::<Text,_>(endpoint).bind::<Text,_>(default_endpoint).bind::<Text,_>("openai").bind::<Nullable<Text>,_>(None::<String>).bind::<Integer,_>(input.enabled as i32).bind::<Integer,_>(input.sort_order as i32).bind::<Text,_>(&t).bind::<Text,_>(&t).execute(&mut *c).map_err(crate::database_error)?;
        Ok(())
    }
    pub fn llm_models(&self, enabled_only: bool) -> Result<Vec<LlmModelRecord>, BusinessError> {
        let mut c = lock(&self.connection)?;
        let sql = if enabled_only {
            "SELECT model_id,provider_id,display_name,request_model,context_tokens,auto_compact_tokens,max_output_tokens,supports_streaming,supports_tool_use,supports_vision,supports_structured_output,supports_cancellation,supports_reasoning_effort,enabled,sort_order,created_at,updated_at,reasoning_efforts FROM llm_model WHERE enabled=1 ORDER BY sort_order,model_id"
        } else {
            "SELECT model_id,provider_id,display_name,request_model,context_tokens,auto_compact_tokens,max_output_tokens,supports_streaming,supports_tool_use,supports_vision,supports_structured_output,supports_cancellation,supports_reasoning_effort,enabled,sort_order,created_at,updated_at,reasoning_efforts FROM llm_model ORDER BY sort_order,model_id"
        };
        sql_query(sql)
            .load::<ModelRow>(&mut *c)
            .map_err(crate::database_error)?
            .into_iter()
            .map(model_from_row)
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
            .map(|n| {
                i32::try_from(n)
                    .map_err(|_| BusinessError::invalid("max_output_tokens exceeds SQLite"))
            })
            .transpose()?;
        let mut c = lock(&self.connection)?;
        let t = now();
        sql_query("INSERT INTO llm_model(model_id,provider_id,display_name,request_model,context_tokens,auto_compact_tokens,max_output_tokens,supports_streaming,supports_tool_use,supports_vision,supports_structured_output,supports_cancellation,supports_reasoning_effort,enabled,sort_order,created_at,updated_at,reasoning_efforts) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?) ON CONFLICT(model_id) DO UPDATE SET provider_id=excluded.provider_id,display_name=excluded.display_name,request_model=excluded.request_model,context_tokens=excluded.context_tokens,auto_compact_tokens=excluded.auto_compact_tokens,max_output_tokens=excluded.max_output_tokens,supports_streaming=excluded.supports_streaming,supports_tool_use=excluded.supports_tool_use,supports_vision=excluded.supports_vision,supports_structured_output=excluded.supports_structured_output,supports_cancellation=excluded.supports_cancellation,supports_reasoning_effort=excluded.supports_reasoning_effort,enabled=excluded.enabled,sort_order=excluded.sort_order,reasoning_efforts=excluded.reasoning_efforts,updated_at=excluded.updated_at").bind::<Text,_>(input.model_id.trim()).bind::<Text,_>(input.provider_id.trim()).bind::<Text,_>(input.display_name.trim()).bind::<Text,_>(input.request_model.trim()).bind::<Integer,_>(context).bind::<Integer,_>(compact).bind::<Nullable<Integer>,_>(max).bind::<Integer,_>(input.supports_streaming as i32).bind::<Integer,_>(input.supports_tool_use as i32).bind::<Integer,_>(input.supports_vision as i32).bind::<Integer,_>(input.supports_structured_output as i32).bind::<Integer,_>(input.supports_cancellation as i32).bind::<Integer,_>(input.supports_reasoning_effort as i32).bind::<Integer,_>(input.enabled as i32).bind::<Integer,_>(input.sort_order as i32).bind::<Text,_>(&t).bind::<Text,_>(&t).bind::<Text,_>(input.reasoning_efforts.trim()).execute(&mut *c).map_err(crate::database_error)?;
        Ok(())
    }
    pub fn llm_provider_api_key(&self, provider_id: &str) -> Result<Option<String>, BusinessError> {
        if provider_id.trim().is_empty() {
            return Err(BusinessError::invalid("model provider id is required"));
        }
        let mut c = lock(&self.connection)?;
        Ok(sql_query(
            "SELECT api_key AS value FROM llm_model_provider WHERE provider_id=? AND enabled=1",
        )
        .bind::<Text, _>(provider_id.trim())
        .get_result::<OptionalStringRow>(&mut *c)
        .optional()
        .map_err(crate::database_error)?
        .and_then(|r| r.value))
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
        let mut c = lock(&self.connection)?;
        let n =
            sql_query("UPDATE llm_model_provider SET api_key=?,updated_at=? WHERE provider_id=?")
                .bind::<Text, _>(value.trim())
                .bind::<Text, _>(&now())
                .bind::<Text, _>(provider_id.trim())
                .execute(&mut *c)
                .map_err(crate::database_error)?;
        if n == 0 {
            Err(BusinessError::invalid("model provider does not exist"))
        } else {
            Ok(())
        }
    }
    pub fn delete_llm_provider_api_key(&self, provider_id: &str) -> Result<(), BusinessError> {
        if provider_id.trim().is_empty() {
            return Err(BusinessError::invalid("model provider id is required"));
        }
        let mut c = lock(&self.connection)?;
        let n = sql_query(
            "UPDATE llm_model_provider SET api_key=NULL,updated_at=? WHERE provider_id=?",
        )
        .bind::<Text, _>(&now())
        .bind::<Text, _>(provider_id.trim())
        .execute(&mut *c)
        .map_err(crate::database_error)?;
        if n == 0 {
            Err(BusinessError::invalid("model provider does not exist"))
        } else {
            Ok(())
        }
    }

    pub fn recover_startup(&self) -> Result<Vec<SessionEvent>, BusinessError> {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type=Text)]
            turn_id: String,
            #[diesel(sql_type=Text)]
            session_id: String,
            #[diesel(sql_type=Nullable<Text>)]
            key: Option<String>,
            #[diesel(sql_type=Nullable<Text>)]
            model_id: Option<String>,
        }
        let mut c = lock(&self.connection)?;
        let rows=sql_query("SELECT turn_id,session_id,submission_idempotency_key AS key,model_id FROM session_turn WHERE state NOT IN ('completed','failed','cancelled','interrupted') AND (recovery_status IS NULL OR recovery_status NOT IN ('pending','resuming'))").load::<Row>(&mut *c).map_err(crate::database_error)?;
        drop(c);
        let mut events = Vec::new();
        for r in rows {
            let event=self.append_content(&r.session_id,"turn.state",&json!({"turn_id":r.turn_id,"state":"interrupted","reason":"runtime_restarted","submission_idempotency_key":r.key,"model_id":r.model_id}))?;
            if let Some(key) = r.key {
                self.fail_turn(&r.session_id,&key,&json!({"code":"runtime_restarted","message":"Runtime restarted during the turn"}))?
            }
            events.push(event)
        }
        Ok(events)
    }
    pub fn resuming_turns(&self) -> Result<Vec<SuspendedTurn>, BusinessError> {
        let mut c = lock(&self.connection)?;
        let rows=sql_query("SELECT recovery_approval_id AS approval_id,session_id,turn_id,recovery_snapshot_json AS snapshot_json,recovery_status AS status FROM session_turn WHERE recovery_status='resuming'").load::<RecoveryRow>(&mut *c).map_err(crate::database_error)?;
        rows.into_iter()
            .map(|r| {
                Ok(SuspendedTurn {
                    approval_id: r.approval_id,
                    session_id: r.session_id,
                    turn_id: r.turn_id,
                    snapshot: serde_json::from_str(&r.snapshot_json)?,
                    status: r.status,
                })
            })
            .collect()
    }

    pub fn project(&self, root: &str, display_name: &str) -> Result<ProjectRecord, BusinessError> {
        let mut c = lock(&self.connection)?;
        let existing = sql_query("SELECT project_id AS value FROM project WHERE canonical_root=?")
            .bind::<Text, _>(root)
            .get_result::<StringRow>(&mut *c)
            .optional()
            .map_err(crate::database_error)?;
        let id = existing
            .map(|r| r.value)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let t = now();
        sql_query("INSERT INTO project(project_id,canonical_root,display_name,created_at,updated_at,last_opened_at,archived_at) VALUES (?,?,?,?,?,?,NULL) ON CONFLICT(project_id) DO UPDATE SET display_name=excluded.display_name,updated_at=excluded.updated_at,last_opened_at=excluded.last_opened_at,archived_at=NULL").bind::<Text,_>(&id).bind::<Text,_>(root).bind::<Text,_>(display_name).bind::<Text,_>(&t).bind::<Text,_>(&t).bind::<Text,_>(&t).execute(&mut *c).map_err(crate::database_error)?;
        project_by_id_conn(&mut c, &id)?
            .ok_or_else(|| BusinessError::invalid("project was not stored"))
    }
    pub fn projects(&self, include_archived: bool) -> Result<Vec<ProjectRecord>, BusinessError> {
        let mut c = lock(&self.connection)?;
        let sql = if include_archived {
            "SELECT project_id,canonical_root,display_name,created_at,updated_at,last_opened_at,archived_at FROM project ORDER BY last_opened_at DESC"
        } else {
            "SELECT project_id,canonical_root,display_name,created_at,updated_at,last_opened_at,archived_at FROM project WHERE archived_at IS NULL ORDER BY last_opened_at DESC"
        };
        sql_query(sql)
            .load::<ProjectRow>(&mut *c)
            .map_err(crate::database_error)?
            .into_iter()
            .map(project_from_row)
            .collect()
    }
    pub fn project_by_id(&self, id: &str) -> Result<Option<ProjectRecord>, BusinessError> {
        let mut c = lock(&self.connection)?;
        project_by_id_conn(&mut c, id)
    }
    pub fn add_project_dependency(
        &self,
        project_id: &str,
        canonical_root: &str,
        display_name: &str,
    ) -> Result<ProjectDependencyRecord, BusinessError> {
        let mut c = lock(&self.connection)?;
        let project = project_by_id_conn(&mut c, project_id)?
            .ok_or_else(|| BusinessError::invalid("project not found"))?;
        if project.canonical_root == canonical_root {
            return Err(BusinessError::invalid(
                "project cannot depend on its own root",
            ));
        }
        let old=sql_query("SELECT dependency_id AS value FROM project_dependency WHERE project_id=? AND canonical_root=?").bind::<Text,_>(project_id).bind::<Text,_>(canonical_root).get_result::<StringRow>(&mut *c).optional().map_err(crate::database_error)?;
        let id = old
            .map(|r| r.value)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        sql_query("INSERT INTO project_dependency(dependency_id,project_id,canonical_root,display_name,created_at) VALUES (?,?,?,?,?) ON CONFLICT(project_id,canonical_root) DO UPDATE SET display_name=excluded.display_name").bind::<Text,_>(&id).bind::<Text,_>(project_id).bind::<Text,_>(canonical_root).bind::<Text,_>(display_name).bind::<Text,_>(&now()).execute(&mut *c).map_err(crate::database_error)?;
        dependency_by_id(&mut c, project_id, &id)?
            .ok_or_else(|| BusinessError::invalid("dependency was not stored"))
    }
    pub fn project_dependencies(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectDependencyRecord>, BusinessError> {
        let mut c = lock(&self.connection)?;
        sql_query("SELECT dependency_id,project_id,canonical_root,display_name,created_at FROM project_dependency WHERE project_id=? ORDER BY display_name COLLATE NOCASE,dependency_id").bind::<Text,_>(project_id).load::<DependencyRow>(&mut *c).map_err(crate::database_error)?.into_iter().map(dependency_from_row).collect()
    }
    pub fn project_dependency_by_id(
        &self,
        project_id: &str,
        dependency_id: &str,
    ) -> Result<Option<ProjectDependencyRecord>, BusinessError> {
        let mut c = lock(&self.connection)?;
        dependency_by_id(&mut c, project_id, dependency_id)
    }
    pub fn remove_project_dependency(
        &self,
        project_id: &str,
        dependency_id: &str,
    ) -> Result<bool, BusinessError> {
        let mut c = lock(&self.connection)?;
        Ok(
            sql_query("DELETE FROM project_dependency WHERE project_id=? AND dependency_id=?")
                .bind::<Text, _>(project_id)
                .bind::<Text, _>(dependency_id)
                .execute(&mut *c)
                .map_err(crate::database_error)?
                == 1,
        )
    }
    pub fn create_session(
        &self,
        project_id: &str,
        title: Option<&str>,
        model_id: Option<&str>,
    ) -> Result<SessionRecord, BusinessError> {
        let mut c = lock(&self.connection)?;
        if sql_query(
            "SELECT project_id AS value FROM project WHERE project_id=? AND archived_at IS NULL",
        )
        .bind::<Text, _>(project_id)
        .get_result::<StringRow>(&mut *c)
        .optional()
        .map_err(crate::database_error)?
        .is_none()
        {
            return Err(BusinessError::invalid("project not found or archived"));
        }
        let id = Uuid::new_v4().to_string();
        let t = now();
        sql_query("INSERT INTO session(session_id,project_id,title,model_id,status,created_at,updated_at,last_activity_at,pin_at,archived_at) VALUES (?,?,?,?,?,?,?,?,NULL,NULL)").bind::<Text,_>(&id).bind::<Text,_>(project_id).bind::<Nullable<Text>,_>(title).bind::<Nullable<Text>,_>(model_id).bind::<Text,_>("active").bind::<Text,_>(&t).bind::<Text,_>(&t).bind::<Text,_>(&t).execute(&mut *c).map_err(crate::database_error)?;
        session_by_id_conn(&mut c, &id)?
            .ok_or_else(|| BusinessError::invalid("session was not stored"))
    }
    pub fn session_by_id(&self, id: &str) -> Result<Option<SessionRecord>, BusinessError> {
        let mut c = lock(&self.connection)?;
        session_by_id_conn(&mut c, id)
    }
    pub fn session_ui_state(&self, session_id: &str) -> Result<String, BusinessError> {
        let mut c = lock(&self.connection)?;
        let pending_approval = sql_query("SELECT approval_id AS value FROM approval_request WHERE session_id=? AND status='pending' LIMIT 1")
            .bind::<Text, _>(session_id).get_result::<StringRow>(&mut *c).optional().map_err(crate::database_error)?.is_some();
        if pending_approval {
            return Ok("approval".into());
        }
        let pending_question = sql_query("SELECT turn_id AS value FROM session_turn WHERE session_id=? AND recovery_status='pending' AND json_extract(recovery_snapshot_json,'$.pending_call.name')='question' LIMIT 1")
            .bind::<Text, _>(session_id).get_result::<StringRow>(&mut *c).optional().map_err(crate::database_error)?.is_some();
        if pending_question {
            return Ok("question".into());
        }
        let latest = sql_query("SELECT state AS value FROM session_turn WHERE session_id=? ORDER BY created_at DESC,rowid DESC LIMIT 1")
            .bind::<Text, _>(session_id).get_result::<StringRow>(&mut *c).optional().map_err(crate::database_error)?.map(|row| row.value);
        Ok(match latest.as_deref() {
            Some("failed" | "cancelled" | "interrupted") => "failed",
            Some("completed") | None => "idle",
            Some(_) => "running",
        }
        .into())
    }
    pub fn sessions_for_project(
        &self,
        project_id: &str,
        include_archived: bool,
    ) -> Result<Vec<SessionRecord>, BusinessError> {
        let mut c = lock(&self.connection)?;
        let sql = if include_archived {
            "SELECT session_id,project_id,title,model_id,status,created_at,updated_at,last_activity_at,pin_at,archived_at FROM session WHERE project_id=? ORDER BY (pin_at IS NOT NULL) DESC,pin_at DESC,last_activity_at DESC,session_id"
        } else {
            "SELECT session_id,project_id,title,model_id,status,created_at,updated_at,last_activity_at,pin_at,archived_at FROM session WHERE project_id=? AND status='active' ORDER BY (pin_at IS NOT NULL) DESC,pin_at DESC,last_activity_at DESC,session_id"
        };
        sql_query(sql)
            .bind::<Text, _>(project_id)
            .load::<SessionRow>(&mut *c)
            .map_err(crate::database_error)?
            .into_iter()
            .map(session_from_row)
            .collect()
    }
    pub fn set_session_archived(
        &self,
        id: &str,
        archived: bool,
    ) -> Result<SessionRecord, BusinessError> {
        let mut c = lock(&self.connection)?;
        let t = now();
        sql_query(
            "UPDATE session SET status=?,pin_at=NULL,archived_at=?,updated_at=? WHERE session_id=?",
        )
        .bind::<Text, _>(if archived { "archived" } else { "active" })
        .bind::<Nullable<Text>, _>(if archived { Some(t.clone()) } else { None })
        .bind::<Text, _>(&t)
        .bind::<Text, _>(id)
        .execute(&mut *c)
        .map_err(crate::database_error)?;
        session_by_id_conn(&mut c, id)?.ok_or_else(|| BusinessError::invalid("session not found"))
    }
    pub fn set_session_pinned(
        &self,
        id: &str,
        pinned: bool,
    ) -> Result<SessionRecord, BusinessError> {
        let mut c = lock(&self.connection)?;
        let s = session_by_id_conn(&mut c, id)?
            .ok_or_else(|| BusinessError::invalid("session not found"))?;
        if s.status != "active" && pinned {
            return Err(BusinessError::invalid("archived sessions cannot be pinned"));
        }
        sql_query("UPDATE session SET pin_at=? WHERE session_id=?")
            .bind::<Nullable<Text>, _>(if pinned { Some(now()) } else { None })
            .bind::<Text, _>(id)
            .execute(&mut *c)
            .map_err(crate::database_error)?;
        session_by_id_conn(&mut c, id)?.ok_or_else(|| BusinessError::invalid("session not found"))
    }
    pub fn rename_session(&self, id: &str, title: &str) -> Result<SessionRecord, BusinessError> {
        let mut c = lock(&self.connection)?;
        sql_query("UPDATE session SET title=?,updated_at=? WHERE session_id=?")
            .bind::<Text, _>(title)
            .bind::<Text, _>(&now())
            .bind::<Text, _>(id)
            .execute(&mut *c)
            .map_err(crate::database_error)?;
        session_by_id_conn(&mut c, id)?.ok_or_else(|| BusinessError::invalid("session not found"))
    }
}

const EXCHANGE_SELECT: &str = "SELECT call_id,session_id,turn_id,provider,model_id,wire_model,provider_request_id,provider_response_id,state,iteration,started_at,completed_at,input_messages_json,output_message_json,tool_calls_json,usage_json,finish_reason,error_json FROM session_call WHERE session_id=? ORDER BY started_at DESC,call_id DESC";
const MANIFEST_SELECT: &str = "SELECT manifest_id,session_id,turn_id,status,created_at,updated_at,expires_at,restored_at FROM checkpoint_manifest WHERE session_id=? ORDER BY created_at DESC";

fn configure(connection: &mut SqliteConnection) -> Result<(), BusinessError> {
    connection
        .batch_execute(
            "PRAGMA foreign_keys=OFF; PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;",
        )
        .map_err(crate::database_error)?;
    Ok(())
}

fn initialize(connection: &mut SqliteConnection) -> Result<(), BusinessError> {
    business_transaction(connection, |connection| {
        for script in sqlite::schema_scripts() {
            connection
                .batch_execute(script)
                .map_err(crate::database_error)?;
        }
        if !schema::llm_model_provider_includes_default_endpoint(connection)? {
            connection
                .batch_execute(
                    "ALTER TABLE llm_model_provider ADD COLUMN default_endpoint TEXT NOT NULL DEFAULT ''; UPDATE llm_model_provider SET default_endpoint=CASE provider_id WHEN 'deepseek' THEN 'https://api.deepseek.com' WHEN 'zhipu' THEN 'https://open.bigmodel.cn/api/paas/v4' WHEN 'openai' THEN 'https://api.openai.com/v1' WHEN 'kimi' THEN 'https://api.moonshot.ai/v1' WHEN 'claude' THEN 'https://api.anthropic.com/v1' WHEN 'gemini' THEN 'https://generativelanguage.googleapis.com/v1beta/openai' ELSE endpoint END WHERE default_endpoint='';",
                )
                .map_err(crate::database_error)?;
        }
        let actual = schema::table_names(connection)?;
        if actual.iter().map(String::as_str).collect::<Vec<_>>() != sqlite::TABLE_NAMES {
            return Err(BusinessError::invalid(format!(
                "database tables do not match the current schema: {actual:?}"
            )));
        }
        if !schema::session_message_excludes_tool_role(connection)? {
            return Err(BusinessError::invalid(
                "session_message schema still permits the retired tool role",
            ));
        }
        if !schema::session_message_excludes_usage_column(connection)? {
            return Err(BusinessError::invalid(
                "session_message schema still contains the retired usage_json column",
            ));
        }
        if !schema::session_call_includes_provider_ids(connection)? {
            return Err(BusinessError::invalid(
                "session_call schema is missing provider request/response identifiers",
            ));
        }
        for script in sqlite::data_scripts() {
            connection
                .batch_execute(script)
                .map_err(crate::database_error)?;
        }
        Ok(())
    })?;
    Ok(())
}

fn lock<'a>(
    connection: &'a Arc<Mutex<SqliteConnection>>,
) -> Result<MutexGuard<'a, SqliteConnection>, BusinessError> {
    connection
        .lock()
        .map_err(|_| BusinessError::invalid("database lock poisoned"))
}
fn now() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}
fn nonnegative(value: i64) -> Result<u64, BusinessError> {
    u64::try_from(value).map_err(|_| BusinessError::invalid("stored numeric value is negative"))
}

fn project_from_row(row: ProjectRow) -> Result<ProjectRecord, BusinessError> {
    Ok(ProjectRecord {
        project_id: row.project_id,
        canonical_root: row.canonical_root,
        display_name: row.display_name,
        created_at: row.created_at,
        updated_at: row.updated_at,
        last_opened_at: row.last_opened_at,
        archived_at: row.archived_at,
    })
}
fn dependency_from_row(row: DependencyRow) -> Result<ProjectDependencyRecord, BusinessError> {
    Ok(ProjectDependencyRecord {
        dependency_id: row.dependency_id,
        project_id: row.project_id,
        canonical_root: row.canonical_root,
        display_name: row.display_name,
        created_at: row.created_at,
    })
}
fn session_image_from_row(row: SessionImageRow) -> Result<SessionImageRecord, BusinessError> {
    Ok(SessionImageRecord {
        image_id: row.image_id,
        session_id: row.session_id,
        display_name: row.display_name,
        source_kind: row.source_kind,
        original_path: row.original_path,
        storage_path: row.storage_path,
        thumbnail_base64: row.thumbnail_base64,
        created_at: row.created_at,
    })
}
fn session_from_row(row: SessionRow) -> Result<SessionRecord, BusinessError> {
    Ok(SessionRecord {
        session_id: row.session_id,
        project_id: Some(row.project_id),
        title: row.title,
        model_id: row.model_id,
        status: row.status,
        created_at: row.created_at,
        updated_at: row.updated_at,
        last_activity_at: row.last_activity_at,
        archived_at: row.archived_at,
        pin_at: row.pin_at,
    })
}
fn trace_from_row(row: TurnRow) -> Result<SessionTraceTurn, BusinessError> {
    Ok(SessionTraceTurn {
        turn_id: row.turn_id,
        session_id: row.session_id,
        state: row.state,
        model_id: row.model_id,
        created_at: row.created_at,
        updated_at: row.updated_at,
        started_at: row.started_at,
        completed_at: row.completed_at,
        error_code: row.error_code,
        input_tokens: nonnegative(row.input_tokens as i64)?,
        output_tokens: nonnegative(row.output_tokens as i64)?,
        total_tokens: nonnegative(row.total_tokens as i64)?,
    })
}
fn manifest_from_row(row: ManifestRow) -> Result<CheckpointManifest, BusinessError> {
    Ok(CheckpointManifest {
        manifest_id: row.manifest_id,
        session_id: row.session_id,
        turn_id: row.turn_id,
        status: row.status,
        created_at: row.created_at,
        updated_at: row.updated_at,
        expires_at: row.expires_at,
        restored_at: row.restored_at,
    })
}
fn checkpoint_from_row(row: CheckpointRow) -> Result<CheckpointItem, BusinessError> {
    Ok(CheckpointItem {
        checkpoint_id: row.checkpoint_id,
        manifest_id: row.manifest_id,
        session_id: row.session_id,
        turn_id: row.turn_id,
        tool_call_id: row.tool_call_id,
        relative_path: row.relative_path,
        status: row.status,
        created_at: row.created_at,
        restored_at: row.restored_at,
        invalidated_at: row.invalidated_at,
        ordinal: row.ordinal.map(i64::from),
    })
}
fn provider_from_row(row: ProviderRow) -> Result<LlmModelProviderRecord, BusinessError> {
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
fn model_from_row(row: ModelRow) -> Result<LlmModelRecord, BusinessError> {
    Ok(LlmModelRecord {
        model_id: row.model_id,
        provider_id: row.provider_id,
        display_name: row.display_name,
        request_model: row.request_model,
        context_tokens: nonnegative(row.context_tokens as i64)?,
        auto_compact_tokens: nonnegative(row.auto_compact_tokens as i64)?,
        max_output_tokens: row
            .max_output_tokens
            .map(|v| nonnegative(v as i64))
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
fn exchange_from_row(row: ExchangeRow) -> Result<ProviderExchange, BusinessError> {
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

fn project_by_id_conn(
    c: &mut SqliteConnection,
    id: &str,
) -> Result<Option<ProjectRecord>, BusinessError> {
    operations::project::by_id(c, id)
}
fn dependency_by_id(
    c: &mut SqliteConnection,
    project_id: &str,
    id: &str,
) -> Result<Option<ProjectDependencyRecord>, BusinessError> {
    operations::project_dependency::by_id(c, project_id, id)
}
fn session_by_id_conn(
    c: &mut SqliteConnection,
    id: &str,
) -> Result<Option<SessionRecord>, BusinessError> {
    operations::session::by_id(c, id)
}
fn approval_by_id(
    c: &mut SqliteConnection,
    id: &str,
) -> Result<Option<ApprovalRecord>, BusinessError> {
    let row=sql_query("SELECT approval_id,project_id,session_id,turn_id,tool_call_id,operation,arguments_json,status,decision,decision_source,created_at,updated_at FROM approval_request WHERE approval_id=?").bind::<Text,_>(id).get_result::<ApprovalRow>(c).optional().map_err(crate::database_error)?;
    row.map(|r| {
        Ok(ApprovalRecord {
            approval_id: r.approval_id,
            project_id: r.project_id,
            session_id: r.session_id,
            turn_id: r.turn_id,
            tool_call_id: r.tool_call_id,
            operation: r.operation,
            arguments: serde_json::from_str(&r.arguments_json)?,
            status: r.status,
            decision: r.decision,
            decision_source: r.decision_source,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    })
    .transpose()
}
fn manifest_by_turn(
    c: &mut SqliteConnection,
    turn_id: &str,
) -> Result<Option<CheckpointManifest>, BusinessError> {
    sql_query("SELECT manifest_id,session_id,turn_id,status,created_at,updated_at,expires_at,restored_at FROM checkpoint_manifest WHERE turn_id=?").bind::<Text,_>(turn_id).get_result::<ManifestRow>(c).optional().map_err(crate::database_error)?.map(manifest_from_row).transpose()
}

#[derive(QueryableByName)]
struct MessageRow {
    #[diesel(sql_type=Text)]
    message_id: String,
    #[diesel(sql_type=Text)]
    session_id: String,
    #[diesel(sql_type=Nullable<Text>)]
    turn_id: Option<String>,
    #[diesel(sql_type=Nullable<Text>)]
    session_call_id: Option<String>,
    #[diesel(sql_type=Text)]
    role: String,
    #[diesel(sql_type=Text)]
    message_json: String,
    #[diesel(sql_type=Text)]
    created_at: String,
}
fn load_messages(
    c: &mut SqliteConnection,
    session_id: &str,
    turn_id: &str,
) -> Result<Vec<SessionCallMessage>, BusinessError> {
    let rows=sql_query("SELECT message_id,session_id,turn_id,session_call_id,role,message_json,created_at FROM session_message WHERE session_id=? AND turn_id=? AND role IN ('user','assistant','thinking') ORDER BY created_at,rowid").bind::<Text,_>(session_id).bind::<Text,_>(turn_id).load::<MessageRow>(c).map_err(crate::database_error)?;
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
fn load_call_messages(
    c: &mut SqliteConnection,
    session_id: &str,
    call_id: &str,
) -> Result<Vec<SessionCallMessage>, BusinessError> {
    let rows=sql_query("SELECT message_id,session_id,turn_id,session_call_id,role,message_json,created_at FROM session_message WHERE session_id=? AND session_call_id=? AND role IN ('user','assistant','thinking') ORDER BY created_at,rowid").bind::<Text,_>(session_id).bind::<Text,_>(call_id).load::<MessageRow>(c).map_err(crate::database_error)?;
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

#[derive(QueryableByName)]
struct ToolRow {
    #[diesel(sql_type=Text)]
    turn_id: String,
    #[diesel(sql_type=Text)]
    tool_call_id: String,
    #[diesel(sql_type=Nullable<Text>)]
    session_call_id: Option<String>,
    #[diesel(sql_type=Text)]
    name: String,
    #[diesel(sql_type=Nullable<Text>)]
    request_json: Option<String>,
    #[diesel(sql_type=Nullable<Text>)]
    result_json: Option<String>,
    #[diesel(sql_type=Text)]
    state: String,
    #[diesel(sql_type=Nullable<Integer>)]
    ordinal: Option<i32>,
    #[diesel(sql_type=Text)]
    created_at: String,
    #[diesel(sql_type=Text)]
    updated_at: String,
    #[diesel(sql_type=Nullable<Text>)]
    completed_at: Option<String>,
    #[diesel(sql_type=Nullable<Text>)]
    error_code: Option<String>,
}
fn tool_rows(
    c: &mut SqliteConnection,
    sql: &str,
    key: &str,
) -> Result<Vec<SessionCallToolUse>, BusinessError> {
    let rows = sql_query(sql)
        .bind::<Text, _>(key)
        .load::<ToolRow>(c)
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
fn load_tool_uses(
    c: &mut SqliteConnection,
    turn_id: &str,
) -> Result<Vec<SessionCallToolUse>, BusinessError> {
    tool_rows(c,"SELECT turn_id,tool_call_id,session_call_id,name,request_json,result_json,state,ordinal,created_at,updated_at,completed_at,error_code FROM session_tool_use WHERE turn_id=? ORDER BY created_at,COALESCE(ordinal,9223372036854775807),tool_call_id",turn_id)
}
fn load_tool_uses_by_call(
    c: &mut SqliteConnection,
    call_id: &str,
) -> Result<Vec<SessionCallToolUse>, BusinessError> {
    tool_rows(c,"SELECT turn_id,tool_call_id,session_call_id,name,request_json,result_json,state,ordinal,created_at,updated_at,completed_at,error_code FROM session_tool_use WHERE session_call_id=? ORDER BY COALESCE(ordinal,9223372036854775807),created_at,tool_call_id",call_id)
}
#[derive(QueryableByName)]
struct TodoRow {
    #[diesel(sql_type=Text)]
    turn_id: String,
    #[diesel(sql_type=Integer)]
    ordinal: i32,
    #[diesel(sql_type=Text)]
    content: String,
    #[diesel(sql_type=Text)]
    status: String,
    #[diesel(sql_type=Text)]
    priority: String,
    #[diesel(sql_type=Text)]
    created_at: String,
    #[diesel(sql_type=Text)]
    updated_at: String,
    #[diesel(sql_type=Nullable<Text>)]
    completed_at: Option<String>,
}
fn load_todos(
    c: &mut SqliteConnection,
    turn_id: &str,
) -> Result<Vec<SessionTurnTodo>, BusinessError> {
    Ok(sql_query("SELECT turn_id,ordinal,content,status,priority,created_at,updated_at,completed_at FROM session_turn_todo WHERE turn_id=? ORDER BY ordinal").bind::<Text,_>(turn_id).load::<TodoRow>(c).map_err(crate::database_error)?.into_iter().map(|r|SessionTurnTodo{turn_id:r.turn_id,ordinal:r.ordinal as i64,content:r.content,status:r.status,priority:r.priority,created_at:r.created_at,updated_at:r.updated_at,completed_at:r.completed_at}).collect())
}
fn repair_incomplete_tool_exchanges(messages: Vec<Message>) -> Vec<Message> {
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
        index += 1
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diesel_store_round_trips_project_and_session() {
        let store = Store::open_memory().unwrap();
        assert_eq!(
            store
                .settings(None, None)
                .unwrap()
                .into_iter()
                .find(|setting| setting.key == "verify_https_certificates")
                .map(|setting| setting.value),
            Some(json!(true))
        );
        let project = store.project("/tmp/suncode-diesel", "Diesel").unwrap();
        let session = store
            .create_session(&project.project_id, Some("Test"), None)
            .unwrap();
        assert_eq!(
            store
                .project_by_id(&project.project_id)
                .unwrap()
                .unwrap()
                .display_name,
            "Diesel"
        );
        assert_eq!(
            store
                .session_by_id(&session.session_id)
                .unwrap()
                .unwrap()
                .title
                .as_deref(),
            Some("Test")
        );
    }

    #[test]
    fn seeded_model_catalog_matches_current_provider_limits_and_capabilities() {
        let store = Store::open_memory().unwrap();
        let models = store.llm_models(false).unwrap();
        assert_eq!(models.len(), 12);

        let model = |id: &str| {
            models
                .iter()
                .find(|item| item.model_id == id)
                .unwrap_or_else(|| panic!("missing seeded model {id}"))
        };

        let deepseek_flash = model("deepseek-v4-flash");
        assert_eq!(deepseek_flash.request_model, "deepseek-v4-flash-vision-exp");
        assert_eq!(deepseek_flash.context_tokens, 1_000_000);
        assert_eq!(deepseek_flash.max_output_tokens, Some(128_000));
        assert!(deepseek_flash.supports_vision);

        let deepseek_pro = model("deepseek-v4-pro");
        assert_eq!(deepseek_pro.context_tokens, 1_000_000);
        assert!(!deepseek_pro.supports_vision);

        for id in ["glm-5.2", "glm-5.3"] {
            let glm = model(id);
            assert_eq!(glm.context_tokens, 1_000_000);
            assert_eq!(glm.max_output_tokens, Some(128_000));
            assert!(glm.supports_reasoning_effort);
        }
        assert_eq!(model("glm-5.3").reasoning_efforts, ["low", "high"]);

        for id in ["gpt-5.6-sol", "gpt-5.5"] {
            let openai = model(id);
            assert_eq!(openai.context_tokens, 1_048_576);
            assert_eq!(openai.max_output_tokens, Some(128_000));
            assert!(openai.supports_vision);
            assert!(openai.supports_structured_output);
            assert!(openai.supports_reasoning_effort);
        }
        assert_eq!(model("gpt-5.5").request_model, "gpt-5.6-terra");

        let kimi_k2 = model("kimi-k2.7-code");
        assert_eq!(kimi_k2.context_tokens, 262_144);
        assert_eq!(kimi_k2.max_output_tokens, Some(262_144));
        assert!(kimi_k2.supports_vision);
        let kimi_k3 = model("kimi-k3");
        assert_eq!(kimi_k3.context_tokens, 1_000_000);
        assert!(kimi_k3.supports_vision);

        for id in ["claude-opus-5", "claude-sonnet-5"] {
            let claude = model(id);
            assert_eq!(claude.context_tokens, 1_000_000);
            assert_eq!(claude.max_output_tokens, Some(128_000));
            assert!(claude.supports_vision);
            assert!(claude.supports_structured_output);
            assert!(claude.supports_reasoning_effort);
        }

        for id in ["gemini-3.6-flash", "gemini-3.5"] {
            let gemini = model(id);
            assert_eq!(gemini.context_tokens, 1_048_576);
            assert_eq!(gemini.max_output_tokens, Some(65_536));
            assert!(gemini.supports_vision);
            assert!(gemini.supports_structured_output);
            assert!(gemini.supports_reasoning_effort);
        }
        assert_eq!(model("gemini-3.5").request_model, "gemini-3.5-flash");
    }

    #[test]
    fn diesel_projection_persists_messages_tools_and_todos() {
        let store = Store::open_memory().unwrap();
        let project = store
            .project("/tmp/suncode-diesel-projection", "Projection")
            .unwrap();
        let session = store
            .create_session(&project.project_id, None, None)
            .unwrap();
        store
            .append_content(
                &session.session_id,
                "turn.state",
                &json!({"turn_id":"turn-1","state":"resolving_calls"}),
            )
            .unwrap();
        store.append_content(&session.session_id, "message.user", &json!({"message_id":"message-1","turn_id":"turn-1","message":Message::text("user", "hello")})).unwrap();
        store.append_content(&session.session_id, "todo.updated", &json!({"turn_id":"turn-1","todos":[{"content":"Inspect","status":"completed","priority":"high"}]})).unwrap();
        let conversation = store
            .session_conversation_turns(&session.session_id)
            .unwrap();
        assert_eq!(
            conversation[0].messages[0].message,
            serde_json::to_value(Message::text("user", "hello")).unwrap()
        );
        assert_eq!(conversation[0].todos[0].status, "completed");
    }

    #[test]
    fn latest_failed_turn_input_returns_persisted_submission() {
        let store = Store::open_memory().unwrap();
        let project = store.project("/tmp/suncode-retry", "Retry").unwrap();
        let session = store
            .create_session(&project.project_id, None, None)
            .unwrap();
        let admission = store
            .begin_turn_with_images(
                &session.session_id,
                "key-1",
                "retry me",
                "model-x",
                &["img-1".to_string()],
            )
            .unwrap();
        store
            .fail_turn(
                &session.session_id,
                "key-1",
                &json!({"code":"provider_failed","message":"boom"}),
            )
            .unwrap();
        let latest = store
            .latest_failed_turn_input(&session.session_id)
            .unwrap()
            .unwrap();
        assert_eq!(latest.0, "retry me");
        assert_eq!(latest.1, "model-x");
        assert_eq!(latest.2, vec!["img-1"]);
        assert!(!admission.turn_id.is_empty());
    }

    #[test]
    fn session_ui_state_tracks_running_failure_and_idle_turns() {
        let store = Store::open_memory().unwrap();
        let project = store
            .project("/tmp/suncode-session-state", "State")
            .unwrap();
        let session = store
            .create_session(&project.project_id, None, None)
            .unwrap();

        assert_eq!(store.session_ui_state(&session.session_id).unwrap(), "idle");
        store
            .begin_turn(&session.session_id, "state-1", "run", "gpt-5.5")
            .unwrap();
        assert_eq!(
            store.session_ui_state(&session.session_id).unwrap(),
            "running"
        );
        store
            .fail_turn(&session.session_id, "state-1", &json!({"code":"test"}))
            .unwrap();
        assert_eq!(
            store.session_ui_state(&session.session_id).unwrap(),
            "failed"
        );
        store
            .begin_turn(&session.session_id, "state-2", "finish", "gpt-5.5")
            .unwrap();
        store
            .complete_turn(
                &session.session_id,
                "state-2",
                &json!({"status":"completed"}),
            )
            .unwrap();
        assert_eq!(store.session_ui_state(&session.session_id).unwrap(), "idle");
    }
}
