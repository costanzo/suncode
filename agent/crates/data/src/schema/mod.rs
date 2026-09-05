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
