//! Operations for `project_dependency`.

use crate::{
    domain::ProjectDependencyRecord,
    store::{lock, now, Store},
};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;
use diesel::sqlite::SqliteConnection;
use suncode_common::BusinessError;
use uuid::Uuid;

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

#[derive(QueryableByName)]
struct IdRow {
    #[diesel(sql_type = Text)]
    value: String,
}

impl Store {
    pub fn add_project_dependency(
        &self,
        project_id: &str,
        canonical_root: &str,
        display_name: &str,
    ) -> Result<ProjectDependencyRecord, BusinessError> {
        let mut connection = lock(&self.connection)?;
        let project = super::project::by_id(&mut connection, project_id)?
            .ok_or_else(|| BusinessError::invalid("project not found"))?;
        if project.canonical_root == canonical_root {
            return Err(BusinessError::invalid(
                "project cannot depend on its own root",
            ));
        }
        let existing = sql_query("SELECT dependency_id AS value FROM project_dependency WHERE project_id=? AND canonical_root=?")
            .bind::<Text, _>(project_id)
            .bind::<Text, _>(canonical_root)
            .get_result::<IdRow>(&mut *connection)
            .optional()
            .map_err(crate::database_error)?;
        let id = existing
            .map(|row| row.value)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        sql_query("INSERT INTO project_dependency(dependency_id,project_id,canonical_root,display_name,created_at) VALUES (?,?,?,?,?) ON CONFLICT(project_id,canonical_root) DO UPDATE SET display_name=excluded.display_name")
            .bind::<Text, _>(&id)
            .bind::<Text, _>(project_id)
            .bind::<Text, _>(canonical_root)
            .bind::<Text, _>(display_name)
            .bind::<Text, _>(&now())
            .execute(&mut *connection)
            .map_err(crate::database_error)?;
        by_id(&mut connection, project_id, &id)?
            .ok_or_else(|| BusinessError::invalid("dependency was not stored"))
    }

    pub fn project_dependencies(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectDependencyRecord>, BusinessError> {
        let mut connection = lock(&self.connection)?;
        sql_query("SELECT dependency_id,project_id,canonical_root,display_name,created_at FROM project_dependency WHERE project_id=? ORDER BY display_name COLLATE NOCASE,dependency_id")
            .bind::<Text, _>(project_id)
            .load::<Row>(&mut *connection)
            .map_err(crate::database_error)?
            .into_iter()
            .map(to_record)
            .collect()
    }

    pub fn project_dependency_by_id(
        &self,
        project_id: &str,
        dependency_id: &str,
    ) -> Result<Option<ProjectDependencyRecord>, BusinessError> {
        let mut connection = lock(&self.connection)?;
        by_id(&mut connection, project_id, dependency_id)
    }

    pub fn remove_project_dependency(
        &self,
        project_id: &str,
        dependency_id: &str,
    ) -> Result<bool, BusinessError> {
        let mut connection = lock(&self.connection)?;
        Ok(
            sql_query("DELETE FROM project_dependency WHERE project_id=? AND dependency_id=?")
                .bind::<Text, _>(project_id)
                .bind::<Text, _>(dependency_id)
                .execute(&mut *connection)
                .map_err(crate::database_error)?
                == 1,
        )
    }
}
