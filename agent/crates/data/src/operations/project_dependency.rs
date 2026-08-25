//! Operations for `project_dependency`.

use crate::domain::ProjectDependencyRecord;
use crate::store::PersistenceError;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;
use diesel::sqlite::SqliteConnection;

#[derive(QueryableByName)]
struct Row {
    #[diesel(sql_type = Text)]
    dependency_id: String,
    #[diesel(sql_type = Text)]
    project_id: String,
    #[diesel(sql_type = Text)]
    canonical_root: String,
    #[diesel(sql_type = Text)]
    display_name: String,
    #[diesel(sql_type = Text)]
    created_at: String,
}

pub(crate) fn by_id(
    c: &mut SqliteConnection,
    project_id: &str,
    id: &str,
) -> Result<Option<ProjectDependencyRecord>, PersistenceError> {
    sql_query("SELECT dependency_id,project_id,canonical_root,display_name,created_at FROM project_dependency WHERE project_id=? AND dependency_id=?")
        .bind::<Text, _>(project_id).bind::<Text, _>(id).get_result::<Row>(c).optional()?.map(to_record).transpose()
}

fn to_record(row: Row) -> Result<ProjectDependencyRecord, PersistenceError> {
    Ok(ProjectDependencyRecord {
        dependency_id: row.dependency_id,
        project_id: row.project_id,
        canonical_root: row.canonical_root,
        display_name: row.display_name,
        created_at: row.created_at,
    })
}
