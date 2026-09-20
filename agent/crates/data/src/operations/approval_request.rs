//! Operations for `approval_request`.

use crate::{
    domain::*,
    rows::StringRow,
    store::{business_transaction, lock, now, Store},
    ApprovalInput,
};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Nullable, Text};
use suncode_common::BusinessError;
use uuid::Uuid;

pub(crate) fn by_id(
    c: &mut diesel::sqlite::SqliteConnection,
    id: &str,
) -> Result<Option<ApprovalRecord>, BusinessError> {
    let row = sql_query("SELECT approval_id,project_id,session_id,turn_id,tool_call_id,operation,arguments_json,status,decision,decision_source,created_at,updated_at FROM approval_request WHERE approval_id=?").bind::<Text,_>(id).get_result::<crate::rows::ApprovalRow>(c).optional().map_err(crate::database_error)?;
    row.map(to_record).transpose()
}

fn to_record(row: crate::rows::ApprovalRow) -> Result<ApprovalRecord, BusinessError> {
    Ok(ApprovalRecord {
        approval_id: row.approval_id,
        project_id: row.project_id,
        session_id: row.session_id,
        turn_id: row.turn_id,
        tool_call_id: row.tool_call_id,
        operation: row.operation,
        arguments: serde_json::from_str(&row.arguments_json)?,
        status: row.status,
        decision: row.decision,
        decision_source: row.decision_source,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

impl Store {
    pub fn create_approval(
        &self,
        input: ApprovalInput<'_>,
    ) -> Result<ApprovalRecord, BusinessError> {
        let mut c = lock(&self.connection)?;
        let key = format!(
            "{}:{}:{}:approval",
            input.session_id, input.turn_id, input.tool_call_id
        );
        if let Some(row) =
            sql_query("SELECT approval_id AS value FROM approval_request WHERE idempotency_key=?")
                .bind::<Text, _>(&key)
                .get_result::<StringRow>(&mut *c)
                .optional()
                .map_err(crate::database_error)?
        {
            return by_id(&mut c, &row.value)?
                .ok_or_else(|| BusinessError::invalid("approval disappeared"));
        }
        let id = Uuid::new_v4().to_string();
        let t = now();
        let args = serde_json::to_string(input.arguments)?;
        let snapshot = serde_json::to_string(input.snapshot)?;
        business_transaction(&mut c, |c| {
            sql_query("INSERT INTO approval_request(approval_id,project_id,session_id,turn_id,tool_call_id,operation,arguments_json,idempotency_key,status,created_at,updated_at) VALUES (?,?,?,?,?,?,?,?, 'pending',?,?)").bind::<Text,_>(&id).bind::<Nullable<Text>,_>(input.project_id).bind::<Text,_>(input.session_id).bind::<Text,_>(input.turn_id).bind::<Text,_>(input.tool_call_id).bind::<Text,_>(input.operation).bind::<Text,_>(&args).bind::<Text,_>(&key).bind::<Text,_>(&t).bind::<Text,_>(&t).execute(c).map_err(crate::database_error)?;
            sql_query("UPDATE session_turn SET recovery_approval_id=?,recovery_snapshot_json=?,recovery_status='pending',recovery_created_at=?,recovery_updated_at=? WHERE turn_id=?").bind::<Text,_>(&id).bind::<Text,_>(&snapshot).bind::<Text,_>(&t).bind::<Text,_>(&t).bind::<Text,_>(input.turn_id).execute(c).map_err(crate::database_error)?;
            Ok(())
        })?;
        by_id(&mut c, &id)?.ok_or_else(|| BusinessError::invalid("approval creation failed"))
    }
    pub fn approval(&self, id: &str) -> Result<Option<ApprovalRecord>, BusinessError> {
        let mut c = lock(&self.connection)?;
        by_id(&mut c, id)
    }
    pub fn pending_approval(
        &self,
        session_id: &str,
    ) -> Result<Option<ApprovalRecord>, BusinessError> {
        let mut c = lock(&self.connection)?;
        sql_query("SELECT approval_id,project_id,session_id,turn_id,tool_call_id,operation,arguments_json,status,decision,decision_source,created_at,updated_at FROM approval_request WHERE session_id=? AND status='pending' ORDER BY created_at DESC,approval_id DESC LIMIT 1")
            .bind::<Text, _>(session_id)
            .get_result::<crate::rows::ApprovalRow>(&mut *c)
            .optional()
            .map_err(crate::database_error)?
            .map(to_record)
            .transpose()
    }
    pub fn resolve_approval(
        &self,
        id: &str,
        decision: &str,
    ) -> Result<Option<SuspendedTurn>, BusinessError> {
        let mut c = lock(&self.connection)?;
        let approved = decision != "deny";
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type=Text)]
            approval_id: String,
            #[diesel(sql_type=Text)]
            session_id: String,
            #[diesel(sql_type=Text)]
            turn_id: String,
            #[diesel(sql_type=Text)]
            snapshot: String,
        }
        let result = business_transaction(&mut c, |c| {
            let n=sql_query("UPDATE approval_request SET status=?,decision=?,decision_source='user',updated_at=? WHERE approval_id=? AND status='pending'").bind::<Text,_>(if approved{"approved"}else{"denied"}).bind::<Text,_>(decision).bind::<Text,_>(&now()).bind::<Text,_>(id).execute(c).map_err(crate::database_error)?;
            if n == 0 {
                return Ok(None);
            }
            let row=sql_query("SELECT recovery_approval_id AS approval_id,session_id,turn_id,recovery_snapshot_json AS snapshot FROM session_turn WHERE recovery_approval_id=? AND recovery_status='pending'").bind::<Text,_>(id).get_result::<Row>(c).map_err(crate::database_error)?;
            if decision == "allow_session" {
                sql_query("INSERT INTO configuration(scope,session_id,key,value_json,updated_at) VALUES ('session',?,'full_control','true',?) ON CONFLICT(session_id,key) WHERE scope='session' DO UPDATE SET value_json='true',updated_at=excluded.updated_at").bind::<Text,_>(&row.session_id).bind::<Text,_>(&now()).execute(c).map_err(crate::database_error)?;
            }
            sql_query("UPDATE session_turn SET recovery_status=?,recovery_updated_at=? WHERE recovery_approval_id=?").bind::<Text,_>(if approved{"resuming"}else{"denied"}).bind::<Text,_>(&now()).bind::<Text,_>(id).execute(c).map_err(crate::database_error)?;
            Ok(Some(row))
        })?;
        result
            .map(|r| {
                Ok(SuspendedTurn {
                    approval_id: r.approval_id,
                    session_id: r.session_id,
                    turn_id: r.turn_id,
                    snapshot: serde_json::from_str(&r.snapshot)?,
                    status: if approved {
                        "resuming".into()
                    } else {
                        "denied".into()
                    },
                })
            })
            .transpose()
    }
}
