use super::{rows::JournalRow, schema};
use chrono::Utc;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sqlite::SqliteConnection;
use serde_json::{json, Value};
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use suncode_common::BusinessError;
use suncode_database::sqlite;

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

pub(crate) fn business_transaction<T, F>(
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
}

pub(crate) const EXCHANGE_SELECT: &str = "SELECT call_id,session_id,turn_id,provider,model_id,wire_model,provider_request_id,provider_response_id,state,iteration,started_at,completed_at,input_messages_json,output_message_json,tool_calls_json,usage_json,finish_reason,error_json FROM session_call WHERE session_id=? ORDER BY started_at DESC,call_id DESC";
pub(crate) const MANIFEST_SELECT: &str = "SELECT manifest_id,session_id,turn_id,status,created_at,updated_at,expires_at,restored_at FROM checkpoint_manifest WHERE session_id=? ORDER BY created_at DESC";

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

pub(crate) fn lock<'a>(
    connection: &'a Arc<Mutex<SqliteConnection>>,
) -> Result<MutexGuard<'a, SqliteConnection>, BusinessError> {
    connection
        .lock()
        .map_err(|_| BusinessError::invalid("database lock poisoned"))
}
pub(crate) fn now() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}
pub(crate) fn nonnegative(value: i64) -> Result<u64, BusinessError> {
    u64::try_from(value).map_err(|_| BusinessError::invalid("stored numeric value is negative"))
}
