//! Operations for `project`.

use crate::{
    domain::ProjectRecord,
    store::{lock, now, Store},
};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;
use diesel::sqlite::Sqlite;
use diesel::sqlite::SqliteConnection;
use suncode_common::BusinessError;
use uuid::Uuid;

use crate::schema::tables::project::dsl as projects;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::tables::project)]
#[diesel(check_for_backend(Sqlite))]
struct Row {
    project_id: String,
    user_id: String,
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
) -> Result<Option<ProjectRecord>, BusinessError> {
    projects::project
        .filter(projects::project_id.eq(id))
        .select(Row::as_select())
        .first::<Row>(c)
        .optional()
        .map_err(crate::database_error)?
        .map(to_record)
        .transpose()
}

pub(crate) fn by_id_for_user(
    c: &mut SqliteConnection,
    user_id: &str,
    id: &str,
) -> Result<Option<ProjectRecord>, BusinessError> {
    projects::project
        .filter(projects::project_id.eq(id))
        .filter(projects::user_id.eq(user_id))
        .select(Row::as_select())
        .first::<Row>(c)
        .optional()
        .map_err(crate::database_error)?
        .map(to_record)
        .transpose()
}

fn to_record(row: Row) -> Result<ProjectRecord, BusinessError> {
    Ok(ProjectRecord {
        project_id: row.project_id,
        user_id: row.user_id,
        canonical_root: row.canonical_root,
        display_name: row.display_name,
        created_at: row.created_at,
        updated_at: row.updated_at,
        last_opened_at: row.last_opened_at,
        archived_at: row.archived_at,
    })
}

#[derive(QueryableByName)]
struct IdRow {
    #[diesel(sql_type = Text)]
    value: String,
}

impl Store {
    pub fn project(&self, root: &str, display_name: &str) -> Result<ProjectRecord, BusinessError> {
        self.project_for_user("default", root, display_name)
    }

    pub fn project_for_user(
        &self,
        user_id: &str,
        root: &str,
        display_name: &str,
    ) -> Result<ProjectRecord, BusinessError> {
        if user_id.trim().is_empty() {
            return Err(BusinessError::invalid("user_id is required"));
        }
        let mut connection = lock(&self.connection)?;
        let existing = sql_query(
            "SELECT project_id AS value FROM project WHERE user_id=? AND canonical_root=?",
        )
        .bind::<Text, _>(user_id)
        .bind::<Text, _>(root)
        .get_result::<IdRow>(&mut *connection)
        .optional()
        .map_err(crate::database_error)?;
        let id = existing
            .map(|row| row.value)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let timestamp = now();
        sql_query("INSERT INTO project(project_id,user_id,canonical_root,display_name,created_at,updated_at,last_opened_at,archived_at) VALUES (?,?,?,?,?,?,?,NULL) ON CONFLICT(project_id) DO UPDATE SET user_id=excluded.user_id,display_name=excluded.display_name,updated_at=excluded.updated_at,last_opened_at=excluded.last_opened_at,archived_at=NULL")
            .bind::<Text, _>(&id)
            .bind::<Text, _>(user_id)
            .bind::<Text, _>(root)
            .bind::<Text, _>(display_name)
            .bind::<Text, _>(&timestamp)
            .bind::<Text, _>(&timestamp)
            .bind::<Text, _>(&timestamp)
            .execute(&mut *connection)
            .map_err(crate::database_error)?;
        by_id_for_user(&mut connection, user_id, &id)?
            .ok_or_else(|| BusinessError::invalid("project was not stored"))
    }

    pub fn projects(&self, include_archived: bool) -> Result<Vec<ProjectRecord>, BusinessError> {
        self.projects_for_user("default", include_archived)
    }

    pub fn projects_for_user(
        &self,
        user_id: &str,
        include_archived: bool,
    ) -> Result<Vec<ProjectRecord>, BusinessError> {
        if user_id.trim().is_empty() {
            return Err(BusinessError::invalid("user_id is required"));
        }
        let mut connection = lock(&self.connection)?;
        let rows = if include_archived {
            projects::project
                .filter(projects::user_id.eq(user_id))
                .order(projects::last_opened_at.desc())
                .select(Row::as_select())
                .load::<Row>(&mut *connection)
        } else {
            projects::project
                .filter(projects::user_id.eq(user_id))
                .filter(projects::archived_at.is_null())
                .order(projects::last_opened_at.desc())
                .select(Row::as_select())
                .load::<Row>(&mut *connection)
        }
        .map_err(crate::database_error)?;
        rows.into_iter().map(to_record).collect()
    }

    pub fn project_by_id(&self, id: &str) -> Result<Option<ProjectRecord>, BusinessError> {
        self.project_by_id_for_user("default", id)
    }

    pub fn project_by_id_for_user(
        &self,
        user_id: &str,
        id: &str,
    ) -> Result<Option<ProjectRecord>, BusinessError> {
        if user_id.trim().is_empty() {
            return Err(BusinessError::invalid("user_id is required"));
        }
        let mut connection = lock(&self.connection)?;
        by_id_for_user(&mut connection, user_id, id)
    }
}
