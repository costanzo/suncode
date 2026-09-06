//! Operations for `session`.

use crate::{
    domain::SessionRecord,
    store::{lock, now, Store},
};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Nullable, Text};
use diesel::sqlite::SqliteConnection;
use suncode_common::BusinessError;
use uuid::Uuid;

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
) -> Result<Option<SessionRecord>, BusinessError> {
    sql_query("SELECT session_id,project_id,title,model_id,status,created_at,updated_at,last_activity_at,pin_at,archived_at FROM session WHERE session_id=?")
        .bind::<Text, _>(id).get_result::<Row>(c).optional().map_err(crate::database_error)?.map(to_record).transpose()
}

fn to_record(row: Row) -> Result<SessionRecord, BusinessError> {
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

#[derive(QueryableByName)]
struct ValueRow {
    #[diesel(sql_type = Text)]
    value: String,
}

impl Store {
    pub fn create_session(
        &self,
        project_id: &str,
        title: Option<&str>,
        model_id: Option<&str>,
    ) -> Result<SessionRecord, BusinessError> {
        let mut connection = lock(&self.connection)?;
        if sql_query(
            "SELECT project_id AS value FROM project WHERE project_id=? AND archived_at IS NULL",
        )
        .bind::<Text, _>(project_id)
        .get_result::<ValueRow>(&mut *connection)
        .optional()
        .map_err(crate::database_error)?
        .is_none()
        {
            return Err(BusinessError::invalid("project not found or archived"));
        }
        let id = Uuid::new_v4().to_string();
        let timestamp = now();
        sql_query("INSERT INTO session(session_id,project_id,title,model_id,status,created_at,updated_at,last_activity_at,pin_at,archived_at) VALUES (?,?,?,?,?,?,?,?,NULL,NULL)")
            .bind::<Text, _>(&id)
            .bind::<Text, _>(project_id)
            .bind::<Nullable<Text>, _>(title)
            .bind::<Nullable<Text>, _>(model_id)
            .bind::<Text, _>("active")
            .bind::<Text, _>(&timestamp)
            .bind::<Text, _>(&timestamp)
            .bind::<Text, _>(&timestamp)
            .execute(&mut *connection)
            .map_err(crate::database_error)?;
        by_id(&mut connection, &id)?.ok_or_else(|| BusinessError::invalid("session was not stored"))
    }

    pub fn session_by_id(&self, id: &str) -> Result<Option<SessionRecord>, BusinessError> {
        let mut connection = lock(&self.connection)?;
        by_id(&mut connection, id)
    }

    pub fn session_ui_state(&self, session_id: &str) -> Result<String, BusinessError> {
        let mut connection = lock(&self.connection)?;
        let pending_approval = sql_query("SELECT approval_id AS value FROM approval_request WHERE session_id=? AND status='pending' LIMIT 1")
            .bind::<Text, _>(session_id).get_result::<ValueRow>(&mut *connection).optional().map_err(crate::database_error)?.is_some();
        if pending_approval {
            return Ok("approval".into());
        }
        let pending_question = sql_query("SELECT turn_id AS value FROM session_turn WHERE session_id=? AND recovery_status='pending' AND json_extract(recovery_snapshot_json,'$.pending_call.name')='question' LIMIT 1")
            .bind::<Text, _>(session_id).get_result::<ValueRow>(&mut *connection).optional().map_err(crate::database_error)?.is_some();
        if pending_question {
            return Ok("question".into());
        }
        let latest = sql_query("SELECT state AS value FROM session_turn WHERE session_id=? ORDER BY created_at DESC,rowid DESC LIMIT 1")
            .bind::<Text, _>(session_id).get_result::<ValueRow>(&mut *connection).optional().map_err(crate::database_error)?.map(|row| row.value);
        Ok(match latest.as_deref() {
            Some("failed" | "cancelled" | "interrupted") => "failed",
            Some("completed") | None => "idle",
            Some(_) => "running",
        }
        .into())
    }

    pub fn sessions_for_project(
        &self,
        project_id: &str,
        include_archived: bool,
    ) -> Result<Vec<SessionRecord>, BusinessError> {
        let mut connection = lock(&self.connection)?;
        let sql = if include_archived {
            "SELECT session_id,project_id,title,model_id,status,created_at,updated_at,last_activity_at,pin_at,archived_at FROM session WHERE project_id=? ORDER BY (pin_at IS NOT NULL) DESC,pin_at DESC,last_activity_at DESC,session_id"
        } else {
            "SELECT session_id,project_id,title,model_id,status,created_at,updated_at,last_activity_at,pin_at,archived_at FROM session WHERE project_id=? AND status='active' ORDER BY (pin_at IS NOT NULL) DESC,pin_at DESC,last_activity_at DESC,session_id"
        };
        sql_query(sql)
            .bind::<Text, _>(project_id)
            .load::<Row>(&mut *connection)
            .map_err(crate::database_error)?
            .into_iter()
            .map(to_record)
            .collect()
    }

    pub fn set_session_archived(
        &self,
        id: &str,
        archived: bool,
    ) -> Result<SessionRecord, BusinessError> {
        let mut connection = lock(&self.connection)?;
        let timestamp = now();
        sql_query(
            "UPDATE session SET status=?,pin_at=NULL,archived_at=?,updated_at=? WHERE session_id=?",
        )
        .bind::<Text, _>(if archived { "archived" } else { "active" })
        .bind::<Nullable<Text>, _>(if archived {
            Some(timestamp.clone())
        } else {
            None
        })
        .bind::<Text, _>(&timestamp)
        .bind::<Text, _>(id)
        .execute(&mut *connection)
        .map_err(crate::database_error)?;
        by_id(&mut connection, id)?.ok_or_else(|| BusinessError::invalid("session not found"))
    }

    pub fn set_session_pinned(
        &self,
        id: &str,
        pinned: bool,
    ) -> Result<SessionRecord, BusinessError> {
        let mut connection = lock(&self.connection)?;
        let session = by_id(&mut connection, id)?
            .ok_or_else(|| BusinessError::invalid("session not found"))?;
        if session.status != "active" && pinned {
            return Err(BusinessError::invalid("archived sessions cannot be pinned"));
        }
        sql_query("UPDATE session SET pin_at=? WHERE session_id=?")
            .bind::<Nullable<Text>, _>(if pinned { Some(now()) } else { None })
            .bind::<Text, _>(id)
            .execute(&mut *connection)
            .map_err(crate::database_error)?;
        by_id(&mut connection, id)?.ok_or_else(|| BusinessError::invalid("session not found"))
    }

    pub fn rename_session(&self, id: &str, title: &str) -> Result<SessionRecord, BusinessError> {
        let mut connection = lock(&self.connection)?;
        sql_query("UPDATE session SET title=?,updated_at=? WHERE session_id=?")
            .bind::<Text, _>(title)
            .bind::<Text, _>(&now())
            .bind::<Text, _>(id)
            .execute(&mut *connection)
            .map_err(crate::database_error)?;
        by_id(&mut connection, id)?.ok_or_else(|| BusinessError::invalid("session not found"))
    }
}
