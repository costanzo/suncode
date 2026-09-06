//! Cross-table startup and suspended-turn recovery operations.

use crate::operations::projection;
use crate::{
    domain::*,
    rows::RecoveryRow,
    store::{lock, Store},
};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Nullable, Text};
use serde_json::json;
use suncode_common::BusinessError;

impl Store {
    pub fn recover_startup(&self) -> Result<Vec<SessionEvent>, BusinessError> {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type=Text)]
            turn_id: String,
            #[diesel(sql_type=Text)]
            session_id: String,
            #[diesel(sql_type=Nullable<Text>)]
            key: Option<String>,
            #[diesel(sql_type=Nullable<Text>)]
            model_id: Option<String>,
        }
        let mut c = lock(&self.connection)?;
        let rows=sql_query("SELECT turn_id,session_id,submission_idempotency_key AS key,model_id FROM session_turn WHERE state NOT IN ('completed','failed','cancelled','interrupted') AND (recovery_status IS NULL OR recovery_status NOT IN ('pending','resuming'))").load::<Row>(&mut *c).map_err(crate::database_error)?;
        drop(c);
        let mut events = Vec::new();
        for r in rows {
            let event = projection::append_content(
                self,
                &r.session_id,
                "turn.state",
                &json!({"turn_id":r.turn_id,"state":"interrupted","reason":"runtime_restarted","submission_idempotency_key":r.key,"model_id":r.model_id}),
            )?;
            if let Some(key) = r.key {
                self.fail_turn(&r.session_id,&key,&json!({"code":"runtime_restarted","message":"Runtime restarted during the turn"}))?
            }
            events.push(event)
        }
        Ok(events)
    }
    pub fn resuming_turns(&self) -> Result<Vec<SuspendedTurn>, BusinessError> {
        let mut c = lock(&self.connection)?;
        let rows=sql_query("SELECT recovery_approval_id AS approval_id,session_id,turn_id,recovery_snapshot_json AS snapshot_json,recovery_status AS status FROM session_turn WHERE recovery_status='resuming'").load::<RecoveryRow>(&mut *c).map_err(crate::database_error)?;
        rows.into_iter()
            .map(|r| {
                Ok(SuspendedTurn {
                    approval_id: r.approval_id,
                    session_id: r.session_id,
                    turn_id: r.turn_id,
                    snapshot: serde_json::from_str(&r.snapshot_json)?,
                    status: r.status,
                })
            })
            .collect()
    }
}
