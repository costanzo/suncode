use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sqlite::SqliteConnection;

pub(crate) mod tables;

#[derive(diesel::QueryableByName)]
struct SqlRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    sql: String,
}

pub(crate) fn table_names(
    connection: &mut SqliteConnection,
) -> Result<Vec<String>, crate::BusinessError> {
    #[derive(diesel::QueryableByName)]
    struct NameRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }
    Ok(sql_query("SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .load::<NameRow>(connection).map_err(crate::database_error)?
        .into_iter()
        .map(|row| row.name)
        .collect())
}

pub(crate) fn rename_legacy_session_tables(
    connection: &mut SqliteConnection,
) -> Result<(), crate::BusinessError> {
    let tables = table_names(connection)?;
    let renames = [
        ("approval_request", "session_approval_request"),
        ("checkpoint", "session_checkpoint"),
        ("checkpoint_manifest", "session_checkpoint_manifest"),
        ("subagent_invocation", "session_subagent_invocation"),
    ];
    for (legacy, current) in renames {
        if tables.iter().any(|name| name == legacy) && !tables.iter().any(|name| name == current) {
            connection
                .batch_execute(&format!("ALTER TABLE {legacy} RENAME TO {current};"))
                .map_err(crate::database_error)?;
        }
    }
    for index in [
        "approval_request_session_status_idx",
        "checkpoint_manifest_ordinal_idx",
        "checkpoint_manifest_session_status_idx",
        "checkpoint_manifest_expiry_idx",
        "checkpoint_manifest_turn_idx",
        "subagent_invocation_parent_idx",
        "subagent_invocation_turn_idx",
    ] {
        connection
            .batch_execute(&format!("DROP INDEX IF EXISTS {index};"))
            .map_err(crate::database_error)?;
    }
    Ok(())
}

pub(crate) fn session_message_excludes_tool_role(
    connection: &mut SqliteConnection,
) -> Result<bool, crate::BusinessError> {
    let sql =
        sql_query("SELECT sql FROM sqlite_schema WHERE type='table' AND name='session_message'")
            .get_result::<SqlRow>(connection)
            .map_err(crate::database_error)?
            .sql;
    Ok(!sql.contains("'tool'"))
}

pub(crate) fn session_message_excludes_usage_column(
    connection: &mut SqliteConnection,
) -> Result<bool, crate::BusinessError> {
    #[derive(diesel::QueryableByName)]
    struct ColumnRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }
    let columns = sql_query("PRAGMA table_info(session_message)")
        .load::<ColumnRow>(connection)
        .map_err(crate::database_error)?;
    Ok(!columns.iter().any(|column| column.name == "usage_json"))
}

pub(crate) fn session_call_includes_provider_ids(
    connection: &mut SqliteConnection,
) -> Result<bool, crate::BusinessError> {
    #[derive(diesel::QueryableByName)]
    struct ColumnRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }
    let columns = sql_query("PRAGMA table_info(session_call)")
        .load::<ColumnRow>(connection)
        .map_err(crate::database_error)?;
    Ok(columns
        .iter()
        .any(|column| column.name == "provider_request_id")
        && columns
            .iter()
            .any(|column| column.name == "provider_response_id"))
}

pub(crate) fn session_includes_subagent_columns(
    connection: &mut SqliteConnection,
) -> Result<bool, crate::BusinessError> {
    #[derive(diesel::QueryableByName)]
    struct ColumnRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }
    let columns = sql_query("PRAGMA table_info(session)")
        .load::<ColumnRow>(connection)
        .map_err(crate::database_error)?;
    Ok(["kind", "parent_session_id", "agent_id", "agent_version"]
        .iter()
        .all(|name| columns.iter().any(|column| column.name == *name)))
}

pub(crate) fn llm_model_provider_includes_default_endpoint(
    connection: &mut SqliteConnection,
) -> Result<bool, crate::BusinessError> {
    #[derive(diesel::QueryableByName)]
    struct ColumnRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }
    let columns = sql_query("PRAGMA table_info(llm_model_provider)")
        .load::<ColumnRow>(connection)
        .map_err(crate::database_error)?;
    Ok(columns
        .iter()
        .any(|column| column.name == "default_endpoint"))
}

pub(crate) fn llm_model_provider_supports_anthropic(
    connection: &mut SqliteConnection,
) -> Result<bool, crate::BusinessError> {
    let sql =
        sql_query("SELECT sql FROM sqlite_schema WHERE type='table' AND name='llm_model_provider'")
            .get_result::<SqlRow>(connection)
            .map_err(crate::database_error)?
            .sql;
    Ok(sql.contains("'anthropic'"))
}

pub(crate) fn llm_model_includes_computer_use(
    connection: &mut SqliteConnection,
) -> Result<bool, crate::BusinessError> {
    #[derive(diesel::QueryableByName)]
    struct ColumnRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }
    let columns = sql_query("PRAGMA table_info(llm_model)")
        .load::<ColumnRow>(connection)
        .map_err(crate::database_error)?;
    Ok(columns
        .iter()
        .any(|column| column.name == "supports_computer_use"))
}
