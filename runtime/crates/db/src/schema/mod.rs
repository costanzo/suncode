use rusqlite::Connection;

pub(super) const SCRIPTS: &[&str] = &[
    include_str!("audit_record.sql"),
    include_str!("projects.sql"),
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
    "projects",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_contains_one_script_per_table() {
        assert_eq!(SCRIPTS.len(), TABLE_NAMES.len());
    }
}
