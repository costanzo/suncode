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
    pub(crate) secrets: Arc<dyn suncode_secrets::SecretStore>,
    pub(crate) credential_lock: Arc<Mutex<()>>,
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
            secrets: Arc::new(suncode_secrets::SystemSecretStore),
            credential_lock: Arc::new(Mutex::new(())),
        })
    }
    pub fn open_memory() -> Result<Self, BusinessError> {
        let mut connection = SqliteConnection::establish(":memory:")
            .map_err(|e| BusinessError::invalid(e.to_string()))?;
        configure(&mut connection)?;
        initialize(&mut connection)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            secrets: Arc::new(suncode_secrets::MemorySecretStore::default()),
            credential_lock: Arc::new(Mutex::new(())),
        })
    }
    /// Supply a credential backend for an embedding host or deterministic tests.
    pub fn with_secret_store(mut self, secrets: Arc<dyn suncode_secrets::SecretStore>) -> Self {
        self.secrets = secrets;
        self
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

pub(crate) const EXCHANGE_SELECT: &str = "SELECT call_id,session_id,turn_id,provider,model_id,wire_model,provider_request_id,provider_response_id,state,iteration,started_at,completed_at,output_message_json,tool_calls_json,usage_json,finish_reason,error_json FROM session_call WHERE session_id=? ORDER BY started_at DESC,call_id DESC";

fn configure(connection: &mut SqliteConnection) -> Result<(), BusinessError> {
    connection
        .batch_execute(
            "PRAGMA foreign_keys=OFF; PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;",
        )
        .map_err(crate::database_error)?;
    Ok(())
}

/// Schema version recorded in `PRAGMA user_version`. Bump it for any schema
/// change. There is no upgrade path: a database with another version is 
/// rejected and must be deleted or replaced by a new data directory.
pub(crate) const SCHEMA_VERSION: i32 = 1;

fn initialize(connection: &mut SqliteConnection) -> Result<(), BusinessError> {
    business_transaction(connection, |connection| {
        let version = user_version(connection)?;
        let fresh = version == 0 && schema::table_names(connection)?.is_empty();
        if version != SCHEMA_VERSION && !fresh {
            return Err(BusinessError::new(
                "database_schema_incompatible",
                format!(
                "This database uses schema version {version}, but this build requires version {SCHEMA_VERSION}. Delete the database or selected a new data directory."
            )));
        }
        for script in sqlite::schema_scripts() {
            connection
                .batch_execute(script)
                .map_err(crate::database_error)?;
        }
        
        let actual = schema::table_names(connection)?;
        if actual.iter().map(String::as_str).collect::<Vec<_>>() != sqlite::TABLE_NAMES {
            return Err(BusinessError::invalid(format!(
                "database tables do not match the current schema: {actual:?}"
            )));
        }
        for script in sqlite::data_scripts() {
            connection
                .batch_execute(script)
                .map_err(crate::database_error)?;
        }
        connection
            .batch_execute(&format!("PRAGMA user_version={SCHEMA_VERSION}"))
            .map_err(crate::database_error)?;
        Ok(())
    })?;
    Ok(())
}

pub(crate) fn user_version(connection: &mut SqliteConnection) -> Result<i32, BusinessError> {
    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Integer)]
        user_version: i32,
    }
    sql_query("PRAGMA user_version")
        .get_result::<Row>(connection)
        .map(|row| row.user_version)
        .map_err(crate::database_error)
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
