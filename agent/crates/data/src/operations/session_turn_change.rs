//! Operations for `session_turn_change`.
//!
//! One row per turn and path keeps the first-touch baseline and the latest
//! version, so repeated `write`/`edit`/`delete` calls collapse into one net
//! change.
//! Each version also records the UTC day folder it was stored under; the
//! operations store drops whole day folders once no row can reference them.

use crate::{
    domain::{change_status, StoredFileVersion, TurnFileChange, TurnFileVersion},
    store::{lock, nonnegative, now, Store},
};
use chrono::{Days, NaiveDate, Utc};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Integer, Nullable, Text};
use diesel::sqlite::SqliteConnection;
use serde_json::Value;
use suncode_common::BusinessError;

/// Full days a stored version is kept after the day it was stored. A change
/// expires at the start of `first stored day + RETENTION_DAYS + 1`, the same
/// instant its earliest day folder becomes eligible for removal.
const RETENTION_DAYS: u64 = 30;
const DAY_FORMAT: &str = "%Y-%m-%d";

#[derive(QueryableByName)]
struct ChangeRow {
    #[diesel(sql_type = Text)]
    relative_path: String,
    #[diesel(sql_type = Nullable<Text>)]
    baseline_sha256: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    baseline_stored_on: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    current_sha256: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    current_stored_on: Option<String>,
    #[diesel(sql_type = Integer)]
    is_binary: i32,
    #[diesel(sql_type = BigInt)]
    additions: i64,
    #[diesel(sql_type = BigInt)]
    deletions: i64,
}

#[derive(QueryableByName)]
struct VersionRow {
    #[diesel(sql_type = Text)]
    stored_on: String,
    #[diesel(sql_type = Text)]
    sha256: String,
}

/// Binds `turn_id` then the current time, so expired changes stay hidden until
/// cleanup removes them.
const CHANGE_SELECT: &str = "SELECT relative_path, baseline_sha256, baseline_stored_on, current_sha256, current_stored_on, is_binary, additions, deletions FROM session_turn_change WHERE turn_id = ? AND expires_at > ?";

fn version(row: ChangeRow) -> TurnFileVersion {
    TurnFileVersion {
        path: row.relative_path,
        baseline: row
            .baseline_sha256
            .zip(row.baseline_stored_on)
            .map(|(sha256, stored_on)| StoredFileVersion { stored_on, sha256 }),
        current: row
            .current_sha256
            .zip(row.current_stored_on)
            .map(|(sha256, stored_on)| StoredFileVersion { stored_on, sha256 }),
    }
}

fn day(payload: &Value, name: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(payload.get(name)?.as_str()?, DAY_FORMAT).ok()
}

/// Projects `checkpoint.capture`: the first capture of a path fixes its
/// baseline, later captures replace the latest version and counts. A null
/// latest version records a deleted file. A path whose latest version equals
/// its baseline, including a file created and deleted in the same turn, has
/// no net change and is removed.
pub(crate) fn record(
    connection: &mut SqliteConnection,
    session_id: &str,
    occurred_at: &str,
    payload: &Value,
) -> Result<(), BusinessError> {
    let (Some(turn_id), Some(path)) = (
        payload.get("turn_id").and_then(Value::as_str),
        payload.get("path").and_then(Value::as_str),
    ) else {
        return Ok(());
    };
    if turn_id.is_empty() || path.is_empty() {
        return Ok(());
    }
    let current = payload.get("current_sha256").and_then(Value::as_str);
    let current_day = current.and(day(payload, "current_stored_on"));
    let baseline = payload.get("baseline_sha256").and_then(Value::as_str);
    let baseline_day = baseline.and(day(payload, "baseline_stored_on"));
    if current.is_some() != current_day.is_some() || baseline.is_some() != baseline_day.is_some() {
        return Ok(());
    }
    let count = |name: &str| {
        payload
            .get(name)
            .and_then(Value::as_u64)
            .and_then(|value| i64::try_from(value).ok())
            .unwrap_or(0)
    };
    let binary = payload
        .get("binary")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    // Later capture only move the latest version to the same or later day,
    // so the expiry fixed here never outlives a folder the row references.
    let expires_at = baseline_day
        .into_iter()
        .chain(current_day)
        .min()
        .and_then(|day| day.checked_add_days(Days::new(RETENTION_DAYS + 1)))
        .map(|day| {
            day.and_time(chrono::NaiveTime::MIN)
                .and_utc()
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        });
    if let Some(expires_at) = &expires_at {
        sql_query("INSERT INTO session_turn_change (turn_id, relative_path, session_id, ordinal, baseline_sha256, baseline_stored_on, current_sha256, current_stored_on, is_binary, additions, deletions, created_at, updated_at, expires_at) \
        VALUES (?1,?2,?3,( \
            SELECT COALESCE(MAX(ordinal)+1,0) \
            FROM session_turn_change \
            WHERE turn_id = ?1\
            ),?4,?5,?6,?7,?8,?9,?10,?11,?11,?12) \
        ON CONFLICT(turn_id, relative_path) DO UPDATE SET \
            current_sha256 = excluded.current_sha256, \
            current_stored_on = excluded.current_stored_on, \
            is_binary = excluded.is_binary, \
            additions = excluded.additions, \
            deletions = excluded.deletions, \
            updated_at = excluded.updated_at")
            .bind::<Text, _>(turn_id)
            .bind::<Text, _>(path)
            .bind::<Text, _>(session_id)
            .bind::<Nullable<Text>, _>(baseline)
            .bind::<Nullable<Text>, _>(baseline_day.map(|day| day.format(DAY_FORMAT).to_string()))
            .bind::<Nullable<Text>, _>(current)
            .bind::<Nullable<Text>, _>(current_day.map(|day| day.format(DAY_FORMAT).to_string()))
            .bind::<Integer, _>(i32::from(binary))
            .bind::<BigInt, _>(count("additions"))
            .bind::<BigInt, _>(count("deletions"))
            .bind::<Text, _>(occurred_at)
            .bind::<Text, _>(expires_at)
            .execute(connection)
            .map_err(crate::database_error)?;
    }
    // Without either version the capture deleted a file this turn created.
    let unchanged = if expires_at.is_some() {
        " AND baseline_sha256 IS current_sha256"
    } else {
        ""
    };
    sql_query(format!(
        "DELETE FROM session_turn_change WHERE turn_id = ?1 AND relative_path = ?2{unchanged}"
    ))
    .bind::<Text, _>(turn_id)
    .bind::<Text, _>(path)
    .execute(connection)
    .map_err(crate::database_error)?;
    Ok(())
}

/// Net file changes of one turn, ordered by when each path was first touched.
pub(crate) fn load_changes(
    c: &mut SqliteConnection,
    turn_id: &str,
) -> Result<Vec<TurnFileChange>, BusinessError> {
    sql_query(format!("{CHANGE_SELECT} ORDER BY ordinal"))
        .bind::<Text, _>(turn_id)
        .bind::<Text, _>(now())
        .load::<ChangeRow>(c)
        .map_err(crate::database_error)?
        .into_iter()
        .map(|row| {
            Ok(TurnFileChange {
                status: change_status(row.baseline_sha256.is_some(), row.current_sha256.is_some())
                    .to_string(),
                additions: nonnegative(row.additions)?,
                deletions: nonnegative(row.deletions)?,
                binary: row.is_binary != 0,
                path: row.relative_path,
            })
        })
        .collect()
}

impl Store {
    /// Stored versions behind the net change of `path` in one turn, if any.
    pub fn turn_file_version(
        &self,
        session_id: &str,
        turn_id: &str,
        path: &str,
    ) -> Result<Option<TurnFileVersion>, BusinessError> {
        let mut c = lock(&self.connection)?;
        Ok(sql_query(format!(
            "{CHANGE_SELECT} AND relative_path = ? AND session_id = ?"
        ))
        .bind::<Text, _>(turn_id)
        .bind::<Text, _>(now())
        .bind::<Text, _>(path)
        .bind::<Text, _>(session_id)
        .get_result::<ChangeRow>(&mut *c)
        .optional()
        .map_err(crate::database_error)?
        .map(version))
    }

    /// Store versions behind every net change of one turn, first-touched order.
    pub fn turn_file_versions(
        &self,
        session_id: &str,
        turn_id: &str,
    ) -> Result<Vec<TurnFileVersion>, BusinessError> {
        let mut c = lock(&self.connection)?;
        Ok(
            sql_query(format!("{CHANGE_SELECT} AND session_id = ? ORDER BY ordinal"))
                .bind::<Text, _>(turn_id)
                .bind::<Text, _>(now())
                .bind::<Text, _>(session_id)
                .load::<ChangeRow>(&mut *c)
                .map_err(crate::database_error)?
                .into_iter()
                .map(version)
                .collect(),
        )
    }

    /// Drops expired turn changes and returns the oldest UTC day whose stored
    /// Versions a remaining change can still reference; older day folders can
    /// be removed whole.
    pub fn retain_turn_changes(&self) -> Result<String, BusinessError> {
        let mut c = lock(&self.connection)?;
        sql_query("DELETE FROM session_turn_change WHERE expires_at <= ?")
            .bind::<Text, _>(now())
            .execute(&mut *c)
            .map_err(crate::database_error)?;
        Ok((Utc::now().date_naive() - Days::new(RETENTION_DAYS))
            .format(DAY_FORMAT)
            .to_string())
    }
}

/// Deletes one session's turn chagnes and returns the stored versions they
/// referenced.
pub(crate) fn delete_for_session(
    connection: &mut SqliteConnection,
    session_id: &str,
) -> Result<Vec<StoredFileVersion>, BusinessError> {
    let versions = sql_query("\
    SELECT baseline_stored_on AS stored_on, baseline_sha256 AS sha256\
        FROM session_turn_change \
        WHERE session_id = ?1 AND baseline_sha256 IS NOT NULL\
    UNION SELECT current_stored_on, current_sha256\
        FROM session_turn_change \
        WHERE session_id = ?1 AND current_sha256 IS NOT NULL")
        .bind::<Text, _>(session_id)
        .load::<VersionRow>(connection)
        .map_err(crate::database_error)?;
    sql_query("DELETE FROM session_turn_change WHERE session_id = ?")
        .bind::<Text, _>(session_id)
        .execute(connection)
        .map_err(crate::database_error)?;
    Ok(versions
        .into_iter()
        .map(|row| StoredFileVersion {
            stored_on: row.stored_on,
            sha256: row.sha256,
        })
        .collect())
}

/// Whether any remaining turn change references `version`. Versions are
/// deduplicated per day, so another session may share one.
pub(crate) fn is_reference(
    connection: &mut SqliteConnection,
    version: &StoredFileVersion,
) -> Result<bool, BusinessError> {
    Ok(sql_query("SELECT ?1 AS stored_on, ?2 AS sha256 FROM session_turn_change WHERE (baseline_stored_on = ?1 AND baseline_sha256 = ?2) OR (current_stored_on = ?1 AND current_sha256 = ?2) limit 1")
        .bind::<Text, _>(&version.stored_on)
        .bind::<Text, _>(&version.sha256)
        .get_result::<VersionRow>(connection)
        .optional()
        .map_err(crate::database_error)?
        .is_some())
}
