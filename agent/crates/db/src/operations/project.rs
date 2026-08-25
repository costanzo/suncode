//! Operations for `project`.

use crate::domain::ProjectRecord;
use crate::store::PersistenceError;
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use diesel::sqlite::SqliteConnection;

use crate::schema::tables::project::dsl as projects;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::tables::project)]
#[diesel(check_for_backend(Sqlite))]
struct Row {
    project_id: String,
    canonical_root: String,
    display_name: String,
    created_at: String,
    updated_at: String,
    last_opened_at: String,
    archived_at: Option<String>,
}

pub(crate) fn by_id(
    c: &mut SqliteConnection,
    id: &str,
) -> Result<Option<ProjectRecord>, PersistenceError> {
    projects::project
        .filter(projects::project_id.eq(id))
        .select(Row::as_select())
        .first::<Row>(c)
        .optional()?
        .map(to_record)
        .transpose()
}

fn to_record(row: Row) -> Result<ProjectRecord, PersistenceError> {
    Ok(ProjectRecord {
        project_id: row.project_id,
        canonical_root: row.canonical_root,
        display_name: row.display_name,
        created_at: row.created_at,
        updated_at: row.updated_at,
        last_opened_at: row.last_opened_at,
        archived_at: row.archived_at,
    })
}
