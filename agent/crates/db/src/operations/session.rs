//! Operations for `session`.

use crate::domain::SessionRecord;
use crate::store::PersistenceError;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Nullable, Text};
use diesel::sqlite::SqliteConnection;

#[derive(QueryableByName)]
struct Row {
    #[diesel(sql_type = Text)]
    session_id: String,
    #[diesel(sql_type = Text)]
    project_id: String,
    #[diesel(sql_type = Nullable<Text>)]
    title: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    model_id: Option<String>,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Text)]
    created_at: String,
    #[diesel(sql_type = Text)]
    updated_at: String,
    #[diesel(sql_type = Text)]
    last_activity_at: String,
    #[diesel(sql_type = Nullable<Text>)]
    pin_at: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    archived_at: Option<String>,
}

pub(crate) fn by_id(
    c: &mut SqliteConnection,
    id: &str,
) -> Result<Option<SessionRecord>, PersistenceError> {
    sql_query("SELECT session_id,project_id,title,model_id,status,created_at,updated_at,last_activity_at,pin_at,archived_at FROM session WHERE session_id=?")
        .bind::<Text, _>(id).get_result::<Row>(c).optional()?.map(to_record).transpose()
}

fn to_record(row: Row) -> Result<SessionRecord, PersistenceError> {
    Ok(SessionRecord {
        session_id: row.session_id,
        project_id: Some(row.project_id),
        title: row.title,
        model_id: row.model_id,
        status: row.status,
        created_at: row.created_at,
        updated_at: row.updated_at,
        last_activity_at: row.last_activity_at,
        archived_at: row.archived_at,
        pin_at: row.pin_at,
    })
}
