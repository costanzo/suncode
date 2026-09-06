//! Operations for `session_turn`.

use crate::operations::session::by_id as session_by_id;
use crate::{
    domain::*,
    rows::{RetryInputRow, StringRow, SubmissionRow, TurnRow},
    store::{lock, nonnegative, now, Store},
};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Integer, Nullable, Text};
use serde_json::{json, Value};
use suncode_common::BusinessError;
use uuid::Uuid;

fn to_trace(row: TurnRow) -> Result<SessionTraceTurn, BusinessError> {
    Ok(SessionTraceTurn {
        turn_id: row.turn_id,
        session_id: row.session_id,
        state: row.state,
        model_id: row.model_id,
        created_at: row.created_at,
        updated_at: row.updated_at,
        started_at: row.started_at,
        completed_at: row.completed_at,
        error_code: row.error_code,
        input_tokens: nonnegative(row.input_tokens as i64)?,
        output_tokens: nonnegative(row.output_tokens as i64)?,
        total_tokens: nonnegative(row.total_tokens as i64)?,
    })
}

impl Store {
    pub fn session_usage(&self, session_id: &str) -> Result<Usage, BusinessError> {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type=BigInt)]
            input_tokens: i64,
            #[diesel(sql_type=BigInt)]
            output_tokens: i64,
            #[diesel(sql_type=BigInt)]
            total_tokens: i64,
        }
        let mut c = lock(&self.connection)?;
        let r=sql_query("SELECT COALESCE(SUM(input_tokens),0) AS input_tokens,COALESCE(SUM(output_tokens),0) AS output_tokens,COALESCE(SUM(total_tokens),0) AS total_tokens FROM session_turn WHERE session_id=?").bind::<Text,_>(session_id).get_result::<Row>(&mut *c).map_err(crate::database_error)?;
        Ok(Usage {
            input_tokens: nonnegative(r.input_tokens)?,
            output_tokens: nonnegative(r.output_tokens)?,
            total_tokens: nonnegative(r.total_tokens)?,
        })
    }
    pub fn session_trace_turns(
        &self,
        session_id: &str,
    ) -> Result<Vec<SessionTraceTurn>, BusinessError> {
        let mut c = lock(&self.connection)?;
        sql_query("SELECT turn_id,session_id,state,model_id,created_at,updated_at,started_at,completed_at,error_code,input_tokens,output_tokens,total_tokens FROM session_turn WHERE session_id=? ORDER BY created_at DESC,turn_id DESC").bind::<Text,_>(session_id).load::<TurnRow>(&mut *c).map_err(crate::database_error)?.into_iter().map(to_trace).collect()
    }
    pub fn begin_turn(
        &self,
        session_id: &str,
        key: &str,
        input: &str,
        model: &str,
    ) -> Result<TurnAdmission, BusinessError> {
        self.begin_turn_with_images(session_id, key, input, model, &[])
    }

    pub fn begin_turn_with_images(
        &self,
        session_id: &str,
        key: &str,
        input: &str,
        model: &str,
        image_ids: &[String],
    ) -> Result<TurnAdmission, BusinessError> {
        let mut c = lock(&self.connection)?;
        let old=sql_query("SELECT state,turn_id,input_json,model_id,response_json FROM session_turn WHERE session_id=? AND submission_idempotency_key=?").bind::<Text,_>(session_id).bind::<Text,_>(key).get_result::<SubmissionRow>(&mut *c).optional().map_err(crate::database_error)?;
        if let Some(r) = old {
            let requested = serde_json::to_string(&json!({"input":input,"image_ids":image_ids}))?;
            if r.input_json.as_deref() != Some(&requested) || r.model_id.as_deref() != Some(model) {
                return Err(BusinessError::invalid(
                    "idempotency key was reused with different turn input",
                ));
            }
            return Ok(TurnAdmission {
                created: false,
                turn_id: r.turn_id,
                status: r.state,
                response: r
                    .response_json
                    .map(|v| serde_json::from_str(&v))
                    .transpose()?,
            });
        }
        let s = session_by_id(&mut c, session_id)?
            .ok_or_else(|| BusinessError::invalid("session not found"))?;
        if s.status != "active" {
            return Err(BusinessError::invalid("session is archived"));
        }
        let id = Uuid::new_v4().to_string();
        let t = now();
        let input_json = serde_json::to_string(&json!({"input":input,"image_ids":image_ids}))?;
        sql_query("INSERT INTO session_turn(session_id,submission_idempotency_key,state,created_at,updated_at,turn_id,input_json,model_id,admitted_at) VALUES (?,?, 'admitted',?,?,?,?,?,?)").bind::<Text,_>(session_id).bind::<Text,_>(key).bind::<Text,_>(&t).bind::<Text,_>(&t).bind::<Text,_>(&id).bind::<Text,_>(&input_json).bind::<Text,_>(model).bind::<Text,_>(&t).execute(&mut *c).map_err(crate::database_error)?;
        Ok(TurnAdmission {
            created: true,
            turn_id: id,
            status: "pending".into(),
            response: None,
        })
    }

    /// Return the most recently failed turn's original input and image attachments.
    /// The caller is responsible for submitting it with a fresh idempotency key.
    pub fn latest_failed_turn_input(
        &self,
        session_id: &str,
    ) -> Result<Option<(String, String, Vec<String>)>, BusinessError> {
        let mut c = lock(&self.connection)?;
        let Some(row) = sql_query(
            "SELECT input_json, model_id FROM session_turn WHERE session_id=? AND state='failed' ORDER BY created_at DESC, turn_id DESC LIMIT 1",
        )
        .bind::<Text, _>(session_id)
        .get_result::<RetryInputRow>(&mut *c)
        .optional()
        .map_err(crate::database_error)? else {
            return Ok(None);
        };
        let Some(input_json) = row.input_json else {
            return Ok(None);
        };
        let value: Value = serde_json::from_str(&input_json)
            .map_err(|_| BusinessError::invalid("stored turn input is invalid"))?;
        let input = value
            .get("input")
            .and_then(Value::as_str)
            .ok_or_else(|| BusinessError::invalid("stored turn input is invalid"))?
            .to_string();
        let image_ids = value
            .get("image_ids")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        Ok(Some((input, row.model_id.unwrap_or_default(), image_ids)))
    }
    pub fn mark_turn_started(&self, session_id: &str, key: &str) -> Result<(), BusinessError> {
        let mut c = lock(&self.connection)?;
        let t = now();
        sql_query("UPDATE session_turn SET started_at=COALESCE(started_at,?),updated_at=? WHERE session_id=? AND submission_idempotency_key=? AND state NOT IN ('completed','failed','cancelled','interrupted')").bind::<Text,_>(&t).bind::<Text,_>(&t).bind::<Text,_>(session_id).bind::<Text,_>(key).execute(&mut *c).map_err(crate::database_error)?;
        Ok(())
    }
    pub fn complete_turn(
        &self,
        session_id: &str,
        key: &str,
        response: &Value,
    ) -> Result<(), BusinessError> {
        let mut c = lock(&self.connection)?;
        let t = now();
        let value = serde_json::to_string(response)?;
        sql_query("UPDATE session_turn SET state='completed',response_json=?,error_json=NULL,completed_at=?,updated_at=? WHERE session_id=? AND submission_idempotency_key=? AND state NOT IN ('completed','failed','cancelled','interrupted')").bind::<Text,_>(&value).bind::<Text,_>(&t).bind::<Text,_>(&t).bind::<Text,_>(session_id).bind::<Text,_>(key).execute(&mut *c).map_err(crate::database_error)?;
        Ok(())
    }
    pub fn fail_turn(
        &self,
        session_id: &str,
        key: &str,
        error: &Value,
    ) -> Result<(), BusinessError> {
        let mut c = lock(&self.connection)?;
        let t = now();
        let value = serde_json::to_string(error)?;
        let code = error.get("code").and_then(Value::as_str);
        sql_query("UPDATE session_turn SET state='failed',error_json=?,error_code=COALESCE(?,error_code),completed_at=COALESCE(completed_at,?),updated_at=? WHERE session_id=? AND submission_idempotency_key=? AND state NOT IN ('completed','cancelled','interrupted')").bind::<Text,_>(&value).bind::<Nullable<Text>,_>(code).bind::<Text,_>(&t).bind::<Text,_>(&t).bind::<Text,_>(session_id).bind::<Text,_>(key).execute(&mut *c).map_err(crate::database_error)?;
        Ok(())
    }
    pub fn create_question(
        &self,
        request_id: &str,
        turn_id: &str,
        snapshot: &Value,
    ) -> Result<(), BusinessError> {
        let mut c = lock(&self.connection)?;
        let t = now();
        let value = serde_json::to_string(snapshot)?;
        let changed=sql_query("UPDATE session_turn SET recovery_approval_id=?,recovery_snapshot_json=?,recovery_status='pending',recovery_created_at=?,recovery_updated_at=? WHERE turn_id=? AND (recovery_status IS NULL OR recovery_status='resuming')").bind::<Text,_>(request_id).bind::<Text,_>(&value).bind::<Text,_>(&t).bind::<Text,_>(&t).bind::<Text,_>(turn_id).execute(&mut *c).map_err(crate::database_error)?;
        if changed == 0 {
            Err(BusinessError::invalid(
                "question recovery is already pending or turn is unavailable",
            ))
        } else {
            Ok(())
        }
    }
    pub fn pending_question(&self, session_id: &str) -> Result<Option<Value>, BusinessError> {
        let mut c = lock(&self.connection)?;
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type=Text)]
            request_id: String,
            #[diesel(sql_type=Text)]
            snapshot: String,
        }
        let Some(row)=sql_query("SELECT recovery_approval_id AS request_id,recovery_snapshot_json AS snapshot FROM session_turn WHERE session_id=? AND recovery_status='pending'").bind::<Text,_>(session_id).get_result::<Row>(&mut *c).optional().map_err(crate::database_error)? else{return Ok(None)};
        let snapshot: Value = serde_json::from_str(&row.snapshot)?;
        let Some(call) = snapshot
            .get("pending_call")
            .filter(|v| v.get("name").and_then(Value::as_str) == Some("question"))
        else {
            return Ok(None);
        };
        Ok(Some(
            json!({"request_id":row.request_id,"session_id":session_id,"turn_id":snapshot.get("turn_id"),"tool_call_id":call.get("call_id"),"questions":call.get("arguments").and_then(|v|v.get("questions")).cloned().unwrap_or_else(||json!([]))}),
        ))
    }
    pub fn question_snapshot(&self, request_id: &str) -> Result<Option<Value>, BusinessError> {
        let mut c = lock(&self.connection)?;
        sql_query("SELECT recovery_snapshot_json AS value FROM session_turn WHERE recovery_approval_id=? AND recovery_status='pending'").bind::<Text,_>(request_id).get_result::<StringRow>(&mut *c).optional().map_err(crate::database_error)?.map(|r|Ok(serde_json::from_str(&r.value)?)).transpose()
    }
    pub fn resolve_question(
        &self,
        id: &str,
        answers: &[Vec<String>],
        rejected: bool,
    ) -> Result<Option<SuspendedTurn>, BusinessError> {
        let mut c = lock(&self.connection)?;
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
        let Some(row)=sql_query("SELECT recovery_approval_id AS approval_id,session_id,turn_id,recovery_snapshot_json AS snapshot FROM session_turn WHERE recovery_approval_id=? AND recovery_status='pending'").bind::<Text,_>(id).get_result::<Row>(&mut *c).optional().map_err(crate::database_error)? else{return Ok(None)};
        let mut snapshot: Value = serde_json::from_str(&row.snapshot)?;
        snapshot["question_answers"] = serde_json::to_value(answers)?;
        snapshot["question_rejected"] = json!(rejected);
        let value = serde_json::to_string(&snapshot)?;
        sql_query("UPDATE session_turn SET recovery_status='resuming',recovery_snapshot_json=?,recovery_updated_at=? WHERE recovery_approval_id=? AND recovery_status='pending'").bind::<Text,_>(&value).bind::<Text,_>(&now()).bind::<Text,_>(id).execute(&mut *c).map_err(crate::database_error)?;
        Ok(Some(SuspendedTurn {
            approval_id: row.approval_id,
            session_id: row.session_id,
            turn_id: row.turn_id,
            snapshot,
            status: "resuming".into(),
        }))
    }
    pub fn finish_suspended(&self, id: &str, status: &str) -> Result<(), BusinessError> {
        let mut c = lock(&self.connection)?;
        let terminal = matches!(status, "completed" | "denied" | "failed");
        sql_query("UPDATE session_turn SET recovery_status=?,recovery_updated_at=?,recovery_snapshot_json=CASE WHEN ? THEN '{}' ELSE recovery_snapshot_json END WHERE recovery_approval_id=?").bind::<Text,_>(status).bind::<Text,_>(&now()).bind::<Integer,_>(terminal as i32).bind::<Text,_>(id).execute(&mut *c).map_err(crate::database_error)?;
        Ok(())
    }
}
