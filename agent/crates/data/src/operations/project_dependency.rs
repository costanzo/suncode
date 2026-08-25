//! Operations for `project_dependency`.

use crate::domain::ProjectDependencyRecord;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;
use diesel::sqlite::SqliteConnection;
use suncode_common::BusinessError;

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
) -> Result<Option<ProjectDependencyRecord>, BusinessError> {
    sql_query("SELECT dependency_id,project_id,canonical_root,display_name,created_at FROM project_dependency WHERE project_id=? AND dependency_id=?")
        .bind::<Text, _>(project_id).bind::<Text, _>(id).get_result::<Row>(c).optional().map_err(crate::database_error)?.map(to_record).transpose()
}

fn to_record(row: Row) -> Result<ProjectDependencyRecord, BusinessError> {
    Ok(ProjectDependencyRecord {
        dependency_id: row.dependency_id,
        project_id: row.project_id,
        canonical_root: row.canonical_root,
        display_name: row.display_name,
        created_at: row.created_at,
    })
}
