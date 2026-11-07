use diesel::prelude::*;
use diesel::sql_query;
use diesel::sqlite::SqliteConnection;

pub(crate) mod tables;

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
