//! Operations for `session_image`.

use crate::{
    domain::SessionImageRecord,
    store::{business_transaction, lock, now, Store},
};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Nullable, Text};
use diesel::sqlite::SqliteConnection;
use std::path::Path;
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

impl Store {
    pub fn session_images(
        &self,
        session_id: &str,
    ) -> Result<Vec<SessionImageRecord>, BusinessError> {
        let mut c = lock(&self.connection)?;
        sql_query("SELECT image_id,session_id,display_name,source_kind,original_path,storage_path,thumbnail_base64,created_at FROM session_image WHERE session_id=? ORDER BY created_at,image_id")
            .bind::<Text, _>(session_id)
            .load::<Row>(&mut *c)
            .map_err(crate::database_error)?
            .into_iter()
            .map(to_record)
            .collect()
    }
    pub fn session_image_by_id(
        &self,
        session_id: &str,
        image_id: &str,
    ) -> Result<Option<SessionImageRecord>, BusinessError> {
        let mut c = lock(&self.connection)?;
        by_id(&mut c, session_id, image_id)
    }
    #[allow(clippy::too_many_arguments)]
    pub fn insert_session_image(
        &self,
        image_id: &str,
        session_id: &str,
        display_name: &str,
        source_kind: &str,
        original_path: Option<&str>,
        storage_path: &Path,
        thumbnail_base64: &str,
    ) -> Result<SessionImageRecord, BusinessError> {
        let display_name = display_name.trim();
        let source_kind = source_kind.trim();
        let thumbnail_base64 = thumbnail_base64.trim();
        if display_name.is_empty()
            || thumbnail_base64.is_empty()
            || !matches!(source_kind, "file" | "clipboard")
        {
            return Err(BusinessError::invalid("session image has invalid fields"));
        }
        let storage_path = storage_path
            .to_str()
            .ok_or_else(|| BusinessError::invalid("image storage path is not valid UTF-8"))?;
        let session = self
            .session_by_id(session_id)?
            .ok_or_else(|| BusinessError::missing("session"))?;
        let project_id = session.project_id.unwrap_or_default();
        let created_at = now();
        let mut c = lock(&self.connection)?;
        business_transaction(&mut c, |c| {
            sql_query("INSERT INTO session_image(image_id,session_id,display_name,source_kind,original_path,storage_path,thumbnail_base64,created_at) VALUES (?,?,?,?,?,?,?,?)")
                .bind::<Text, _>(image_id)
                .bind::<Text, _>(session_id)
                .bind::<Text, _>(display_name)
                .bind::<Text, _>(source_kind)
                .bind::<Nullable<Text>, _>(original_path)
                .bind::<Text, _>(storage_path)
                .bind::<Text, _>(thumbnail_base64)
                .bind::<Text, _>(&created_at)
                .execute(c)
                .map_err(crate::database_error)?;
            sql_query("UPDATE session SET updated_at=?,last_activity_at=? WHERE session_id=?")
                .bind::<Text, _>(&created_at)
                .bind::<Text, _>(&created_at)
                .bind::<Text, _>(session_id)
                .execute(c)
                .map_err(crate::database_error)?;
            if !project_id.is_empty() {
                sql_query("UPDATE project SET updated_at=?,last_opened_at=? WHERE project_id=?")
                    .bind::<Text, _>(&created_at)
                    .bind::<Text, _>(&created_at)
                    .bind::<Text, _>(&project_id)
                    .execute(c)
                    .map_err(crate::database_error)?;
            }
            Ok(())
        })?;
        drop(c);
        self.session_image_by_id(session_id, image_id)?
            .ok_or_else(|| BusinessError::invalid("session image insertion failed"))
    }
    pub fn remove_session_image(
        &self,
        session_id: &str,
        image_id: &str,
    ) -> Result<Option<SessionImageRecord>, BusinessError> {
        let mut c = lock(&self.connection)?;
        let existing = by_id(&mut c, session_id, image_id)?;
        if existing.is_none() {
            return Ok(None);
        }
        sql_query("DELETE FROM session_image WHERE session_id=? AND image_id=?")
            .bind::<Text, _>(session_id)
            .bind::<Text, _>(image_id)
            .execute(&mut *c)
            .map_err(crate::database_error)?;
        Ok(existing)
    }
}
