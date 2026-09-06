//! Operations for `checkpoint_manifest`.

use crate::{
    domain::*,
    rows::ManifestRow,
    store::{lock, now, Store, MANIFEST_SELECT},
};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;
use suncode_common::BusinessError;
use uuid::Uuid;

pub(crate) fn from_row(row: ManifestRow) -> Result<CheckpointManifest, BusinessError> {
    Ok(CheckpointManifest {
        manifest_id: row.manifest_id,
        session_id: row.session_id,
        turn_id: row.turn_id,
        status: row.status,
        created_at: row.created_at,
        updated_at: row.updated_at,
        expires_at: row.expires_at,
        restored_at: row.restored_at,
    })
}

pub(crate) fn by_turn(
    c: &mut diesel::sqlite::SqliteConnection,
    turn_id: &str,
) -> Result<Option<CheckpointManifest>, BusinessError> {
    sql_query("SELECT manifest_id,session_id,turn_id,status,created_at,updated_at,expires_at,restored_at FROM checkpoint_manifest WHERE turn_id=?").bind::<Text,_>(turn_id).get_result::<ManifestRow>(c).optional().map_err(crate::database_error)?.map(from_row).transpose()
}

impl Store {
    pub fn ensure_manifest(
        &self,
        session_id: &str,
        turn_id: &str,
    ) -> Result<CheckpointManifest, BusinessError> {
        let mut c = lock(&self.connection)?;
        if let Some(v) = by_turn(&mut c, turn_id)? {
            return Ok(v);
        }
        let id = Uuid::new_v4().to_string();
        let t = now();
        let expires =
            (Utc::now() + Duration::days(30)).to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        sql_query("INSERT INTO checkpoint_manifest(manifest_id,session_id,turn_id,status,created_at,updated_at,expires_at) VALUES (?,?,?,'available',?,?,?)").bind::<Text,_>(&id).bind::<Text,_>(session_id).bind::<Text,_>(turn_id).bind::<Text,_>(&t).bind::<Text,_>(&t).bind::<Text,_>(&expires).execute(&mut *c).map_err(crate::database_error)?;
        by_turn(&mut c, turn_id)?
            .ok_or_else(|| BusinessError::invalid("checkpoint manifest creation failed"))
    }
    pub fn manifests(&self, session_id: &str) -> Result<Vec<CheckpointManifest>, BusinessError> {
        let mut c = lock(&self.connection)?;
        let t = now();
        sql_query("UPDATE checkpoint_manifest SET status='expired',updated_at=? WHERE session_id=? AND status='available' AND expires_at<=?").bind::<Text,_>(&t).bind::<Text,_>(session_id).bind::<Text,_>(&t).execute(&mut *c).map_err(crate::database_error)?;
        sql_query(MANIFEST_SELECT)
            .bind::<Text, _>(session_id)
            .load::<ManifestRow>(&mut *c)
            .map_err(crate::database_error)?
            .into_iter()
            .map(from_row)
            .collect()
    }
    pub fn manifest(&self, id: &str) -> Result<Option<CheckpointManifest>, BusinessError> {
        let mut c = lock(&self.connection)?;
        sql_query("SELECT manifest_id,session_id,turn_id,status,created_at,updated_at,expires_at,restored_at FROM checkpoint_manifest WHERE manifest_id=?").bind::<Text,_>(id).get_result::<ManifestRow>(&mut *c).optional().map_err(crate::database_error)?.map(from_row).transpose()
    }
    pub fn set_manifest_status(&self, id: &str, status: &str) -> Result<(), BusinessError> {
        let mut c = lock(&self.connection)?;
        let t = now();
        sql_query("UPDATE checkpoint_manifest SET status=?,updated_at=?,restored_at=CASE WHEN ?='restored' THEN ? ELSE restored_at END WHERE manifest_id=?").bind::<Text,_>(status).bind::<Text,_>(&t).bind::<Text,_>(status).bind::<Text,_>(&t).bind::<Text,_>(id).execute(&mut *c).map_err(crate::database_error)?;
        Ok(())
    }
}
