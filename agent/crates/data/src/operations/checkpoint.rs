//! Operations for `session_checkpoint`.

use crate::{
    domain::*,
    rows::CheckpointRow,
    store::{lock, Store},
};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;
use suncode_common::BusinessError;

fn from_row(row: CheckpointRow) -> Result<CheckpointItem, BusinessError> {
    Ok(CheckpointItem {
        checkpoint_id: row.checkpoint_id,
        manifest_id: row.manifest_id,
        session_id: row.session_id,
        turn_id: row.turn_id,
        tool_call_id: row.tool_call_id,
        relative_path: row.relative_path,
        status: row.status,
        created_at: row.created_at,
        restored_at: row.restored_at,
        invalidated_at: row.invalidated_at,
        ordinal: row.ordinal.map(i64::from),
    })
}

impl Store {
    pub fn checkpoint_items(
        &self,
        manifest_id: &str,
    ) -> Result<Vec<CheckpointItem>, BusinessError> {
        let mut c = lock(&self.connection)?;
        sql_query("SELECT checkpoint_id,manifest_id,session_id,turn_id,tool_call_id,relative_path,status,created_at,restored_at,invalidated_at,ordinal FROM session_checkpoint WHERE manifest_id=? ORDER BY ordinal DESC").bind::<Text,_>(manifest_id).load::<CheckpointRow>(&mut *c).map_err(crate::database_error)?.into_iter().map(from_row).collect()
    }
}

#[derive(QueryableByName)]
struct ChangedPathRow {
    #[diesel(sql_type = Text)]
    relative_path: String,
}

/// Paths the agent checkpointed during one turn, including rows later restored
/// or invalidated, deduplicated and ordered by when each path was first touched.
pub(crate) fn load_changed_paths(
    c: &mut diesel::sqlite::SqliteConnection,
    turn_id: &str,
) -> Result<Vec<String>, BusinessError> {
    Ok(sql_query("SELECT relative_path FROM session_checkpoint WHERE turn_id=? AND relative_path IS NOT NULL AND relative_path<>'' GROUP BY relative_path ORDER BY MIN(COALESCE(ordinal,9223372036854775807)),MIN(created_at),relative_path").bind::<Text,_>(turn_id).load::<ChangedPathRow>(c).map_err(crate::database_error)?.into_iter().map(|r| r.relative_path).collect())
}
