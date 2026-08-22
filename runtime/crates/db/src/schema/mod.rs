use rusqlite::Connection;

pub(super) const SCRIPTS: &[&str] = &[
    include_str!("audit_record.sql"),
    include_str!("project.sql"),
    include_str!("session.sql"),
    include_str!("configuration.sql"),
    include_str!("session_turn.sql"),
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
    "session",
    "session_call",
    "session_message",
    "session_tool_use",
    "session_turn",
];

pub(super) fn apply(connection: &Connection) -> rusqlite::Result<()> {
    for script in SCRIPTS {
        connection.execute_batch(script)?;
    }
    Ok(())
}

pub(super) fn table_names(connection: &Connection) -> rusqlite::Result<Vec<String>> {
    let mut statement = connection.prepare(
        "SELECT name FROM sqlite_schema
         WHERE type='table' AND name NOT LIKE 'sqlite_%'
         ORDER BY name",
    )?;
    let names = statement
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(names)
}

pub(super) fn session_message_excludes_tool_role(
    connection: &Connection,
) -> rusqlite::Result<bool> {
    let sql = connection.query_row(
        "SELECT sql FROM sqlite_schema WHERE type='table' AND name='session_message'",
        [],
        |row| row.get::<_, String>(0),
    )?;
    Ok(!sql.contains("'tool'"))
}

pub(super) fn session_message_excludes_usage_column(
    connection: &Connection,
) -> rusqlite::Result<bool> {
    let mut statement = connection.prepare("PRAGMA table_info(session_message)")?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(!columns.iter().any(|column| column == "usage_json"))
}

pub(super) fn session_call_includes_provider_ids(
    connection: &Connection,
) -> rusqlite::Result<bool> {
    let mut statement = connection.prepare("PRAGMA table_info(session_call)")?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(columns.iter().any(|column| column == "provider_request_id")
        && columns
            .iter()
            .any(|column| column == "provider_response_id"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_contains_one_script_per_table() {
        assert_eq!(SCRIPTS.len(), TABLE_NAMES.len());
    }

    #[test]
    fn session_message_schema_excludes_tool_role() {
        let connection = Connection::open_in_memory().unwrap();
        apply(&connection).unwrap();
        assert!(session_message_excludes_tool_role(&connection).unwrap());
    }

    #[test]
    fn session_message_schema_excludes_usage_column() {
        let connection = Connection::open_in_memory().unwrap();
        apply(&connection).unwrap();
        assert!(session_message_excludes_usage_column(&connection).unwrap());
    }

    #[test]
    fn session_call_schema_includes_provider_ids() {
        let connection = Connection::open_in_memory().unwrap();
        apply(&connection).unwrap();
        assert!(session_call_includes_provider_ids(&connection).unwrap());
    }
}
