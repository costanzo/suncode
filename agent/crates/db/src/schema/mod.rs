use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sqlite::SqliteConnection;

pub(crate) mod tables;

pub(super) const SCRIPTS: &[&str] = &[
    include_str!("audit_record.sql"),
    include_str!("project.sql"),
    include_str!("project_dependency.sql"),
    include_str!("session.sql"),
    include_str!("configuration.sql"),
    include_str!("session_turn.sql"),
    include_str!("session_turn_todo.sql"),
    include_str!("session_call.sql"),
    include_str!("session_tool_use.sql"),
    include_str!("approval_request.sql"),
    include_str!("checkpoint_manifest.sql"),
    include_str!("checkpoint.sql"),
    include_str!("session_message.sql"),
    include_str!("llm_model_provider.sql"),
    include_str!("llm_model.sql"),
];

pub(super) const TABLE_NAMES: &[&str] = &[
    "approval_request",
    "audit_record",
    "checkpoint",
    "checkpoint_manifest",
    "configuration",
    "llm_model",
    "llm_model_provider",
    "project",
    "project_dependency",
    "session",
    "session_call",
    "session_message",
    "session_tool_use",
    "session_turn",
    "session_turn_todo",
];

pub(super) fn apply(connection: &mut SqliteConnection) -> QueryResult<usize> {
    let mut count = 0;
    for script in SCRIPTS {
        connection.batch_execute(script)?;
        count += 1;
    }
    Ok(count)
}

#[derive(diesel::QueryableByName)]
struct NameRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
}

pub(super) fn table_names(connection: &mut SqliteConnection) -> QueryResult<Vec<String>> {
    Ok(sql_query("SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .load::<NameRow>(connection)?
        .into_iter()
        .map(|row| row.name)
        .collect())
}

#[derive(diesel::QueryableByName)]
struct SqlRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    sql: String,
}

pub(super) fn session_message_excludes_tool_role(
    connection: &mut SqliteConnection,
) -> QueryResult<bool> {
    let sql =
        sql_query("SELECT sql FROM sqlite_schema WHERE type='table' AND name='session_message'")
            .get_result::<SqlRow>(connection)?
            .sql;
    Ok(!sql.contains("'tool'"))
}

#[derive(diesel::QueryableByName)]
struct TableInfoRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
}

pub(super) fn session_message_excludes_usage_column(
    connection: &mut SqliteConnection,
) -> QueryResult<bool> {
    let columns =
        sql_query("PRAGMA table_info(session_message)").load::<TableInfoRow>(connection)?;
    Ok(!columns.iter().any(|column| column.name == "usage_json"))
}

pub(super) fn session_call_includes_provider_ids(
    connection: &mut SqliteConnection,
) -> QueryResult<bool> {
    let columns = sql_query("PRAGMA table_info(session_call)").load::<TableInfoRow>(connection)?;
    Ok(columns
        .iter()
        .any(|column| column.name == "provider_request_id")
        && columns
            .iter()
            .any(|column| column.name == "provider_response_id"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::Connection;

    #[test]
    fn manifest_contains_one_script_per_table() {
        assert_eq!(SCRIPTS.len(), TABLE_NAMES.len());
    }

    #[test]
    fn session_message_schema_excludes_tool_role() {
        let mut connection = SqliteConnection::establish(":memory:").unwrap();
        apply(&mut connection).unwrap();
        assert!(session_message_excludes_tool_role(&mut connection).unwrap());
    }

    #[test]
    fn session_message_schema_excludes_usage_column() {
        let mut connection = SqliteConnection::establish(":memory:").unwrap();
        apply(&mut connection).unwrap();
        assert!(session_message_excludes_usage_column(&mut connection).unwrap());
    }

    #[test]
    fn session_call_schema_includes_provider_ids() {
        let mut connection = SqliteConnection::establish(":memory:").unwrap();
        apply(&mut connection).unwrap();
        assert!(session_call_includes_provider_ids(&mut connection).unwrap());
    }

    #[test]
    fn session_schema_includes_pin_at() {
        let mut connection = SqliteConnection::establish(":memory:").unwrap();
        apply(&mut connection).unwrap();
        #[derive(diesel::QueryableByName)]
        struct Column {
            #[diesel(sql_type = diesel::sql_types::Text)]
            name: String,
        }
        let columns = diesel::sql_query("PRAGMA table_info(session)")
            .load::<Column>(&mut connection)
            .unwrap();
        assert!(columns.iter().any(|column| column.name == "pin_at"));
    }
}
