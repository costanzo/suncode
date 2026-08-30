//! Operations for `session_image`.

use crate::domain::SessionImageRecord;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Nullable, Text};
use diesel::sqlite::SqliteConnection;
use suncode_common::BusinessError;

#[derive(QueryableByName)]
struct Row {
    #[diesel(sql_type = Text)]
    image_id: String,
    #[diesel(sql_type = Text)]
    session_id: String,
    #[diesel(sql_type = Text)]
    display_name: String,
    #[diesel(sql_type = Text)]
    source_kind: String,
    #[diesel(sql_type = Nullable<Text>)]
    original_path: Option<String>,
    #[diesel(sql_type = Text)]
    storage_path: String,
    #[diesel(sql_type = Text)]
    thumbnail_base64: String,
    #[diesel(sql_type = Text)]
    created_at: String,
}

pub(crate) fn by_id(
    c: &mut SqliteConnection,
    session_id: &str,
    image_id: &str,
) -> Result<Option<SessionImageRecord>, BusinessError> {
    sql_query("SELECT image_id,session_id,display_name,source_kind,original_path,storage_path,thumbnail_base64,created_at FROM session_image WHERE session_id=? AND image_id=?")
        .bind::<Text, _>(session_id)
        .bind::<Text, _>(image_id)
        .get_result::<Row>(c)
        .optional()
        .map_err(crate::database_error)?
        .map(to_record)
        .transpose()
}

fn to_record(row: Row) -> Result<SessionImageRecord, BusinessError> {
    Ok(SessionImageRecord {
        image_id: row.image_id,
        session_id: row.session_id,
        display_name: row.display_name,
        source_kind: row.source_kind,
        original_path: row.original_path,
        storage_path: row.storage_path,
        thumbnail_base64: row.thumbnail_base64,
        created_at: row.created_at,
    })
}
