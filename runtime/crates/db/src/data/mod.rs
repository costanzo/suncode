use rusqlite::Connection;

pub(super) struct Script {
    pub(super) id: &'static str,
    pub(super) sql: &'static str,
}

pub(super) const SCRIPTS: &[Script] = &[
    Script {
        id: "llm_model_provider",
        sql: include_str!("llm_model_provider.sql"),
    },
    Script {
        id: "llm_model",
        sql: include_str!("llm_model.sql"),
    },
];

pub(super) fn apply(connection: &Connection) -> rusqlite::Result<()> {
    for script in SCRIPTS {
        let _ = script.id;
        connection.execute_batch(script.sql)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_llm_data_is_idempotent() {
        let connection = Connection::open_in_memory().unwrap();
        crate::schema::apply(&connection).unwrap();
        apply(&connection).unwrap();
        apply(&connection).unwrap();
        assert_eq!(SCRIPTS.len(), 2);
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM llm_model_provider", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            6
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM llm_model_provider WHERE adapter_type='openai'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            6
        );
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM llm_model", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            12
        );
    }
}
