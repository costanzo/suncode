use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

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

pub(super) fn apply(connection: &mut SqliteConnection) -> QueryResult<usize> {
    let mut count = 0;
    for script in SCRIPTS {
        let _ = script.id;
        connection.batch_execute(script.sql)?;
        count += 1;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::sql_query;
    use diesel::Connection;

    #[derive(diesel::QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        value: i64,
    }

    #[test]
    fn built_in_llm_data_is_idempotent() {
        let mut connection = SqliteConnection::establish(":memory:").unwrap();
        crate::schema::apply(&mut connection).unwrap();
        apply(&mut connection).unwrap();
        apply(&mut connection).unwrap();
        assert_eq!(SCRIPTS.len(), 2);
        assert_eq!(
            sql_query("SELECT COUNT(*) AS value FROM llm_model_provider")
                .get_result::<CountRow>(&mut connection)
                .unwrap()
                .value,
            6
        );
        assert_eq!(
            sql_query(
                "SELECT COUNT(*) AS value FROM llm_model_provider WHERE adapter_type='openai'"
            )
            .get_result::<CountRow>(&mut connection)
            .unwrap()
            .value,
            6
        );
        assert_eq!(
            sql_query("SELECT COUNT(*) AS value FROM llm_model")
                .get_result::<CountRow>(&mut connection)
                .unwrap()
                .value,
            12
        );
    }
}
