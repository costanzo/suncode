//! Operations for `session_turn_todo`.

use crate::{domain::SessionTurnTodo, rows::TodoRow};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;
use suncode_common::BusinessError;

pub(crate) fn load_todos(
    c: &mut diesel::sqlite::SqliteConnection,
    turn_id: &str,
) -> Result<Vec<SessionTurnTodo>, BusinessError> {
    Ok(sql_query("SELECT turn_id,ordinal,content,status,priority,created_at,updated_at,completed_at FROM session_turn_todo WHERE turn_id=? ORDER BY ordinal").bind::<Text,_>(turn_id).load::<TodoRow>(c).map_err(crate::database_error)?.into_iter().map(|r| SessionTurnTodo { turn_id: r.turn_id, ordinal: r.ordinal as i64, content: r.content, status: r.status, priority: r.priority, created_at: r.created_at, updated_at: r.updated_at, completed_at: r.completed_at }).collect())
}
