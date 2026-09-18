//! Operations for durable subagent invocations.

use crate::{
    domain::SubagentInvocationRecord,
    store::{lock, now, Store},
};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Integer, Nullable, Text};
use serde_json::Value;
use suncode_common::BusinessError;

#[derive(QueryableByName)]
struct Row {
    #[diesel(sql_type = Text)]
    invocation_id: String,
    #[diesel(sql_type = Text)]
    parent_session_id: String,
    #[diesel(sql_type = Text)]
    parent_turn_id: String,
    #[diesel(sql_type = Text)]
    parent_tool_call_id: String,
    #[diesel(sql_type = Text)]
    child_session_id: String,
    #[diesel(sql_type = Text)]
    agent_id: String,
    #[diesel(sql_type = Integer)]
    agent_version: i32,
    #[diesel(sql_type = Text)]
    task_json: String,
    #[diesel(sql_type = Text)]
    allowed_tools_json: String,
    #[diesel(sql_type = Text)]
    model_id: String,
    #[diesel(sql_type = Text)]
    state: String,
    #[diesel(sql_type = Nullable<Text>)]
    result_json: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    error_code: Option<String>,
    #[diesel(sql_type = Text)]
    created_at: String,
    #[diesel(sql_type = Nullable<Text>)]
    started_at: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    completed_at: Option<String>,
}

const SELECT: &str = "SELECT invocation_id,parent_session_id,parent_turn_id,parent_tool_call_id,child_session_id,agent_id,agent_version,task_json,allowed_tools_json,model_id,state,result_json,error_code,created_at,started_at,completed_at FROM subagent_invocation";

fn to_record(row: Row) -> Result<SubagentInvocationRecord, BusinessError> {
    Ok(SubagentInvocationRecord {
        invocation_id: row.invocation_id,
        parent_session_id: row.parent_session_id,
        parent_turn_id: row.parent_turn_id,
        parent_tool_call_id: row.parent_tool_call_id,
        child_session_id: row.child_session_id,
        agent_id: row.agent_id,
        agent_version: i64::from(row.agent_version),
        task: serde_json::from_str(&row.task_json)
            .map_err(|_| BusinessError::invalid("stored subagent task is invalid"))?,
        allowed_tools: serde_json::from_str(&row.allowed_tools_json)
            .map_err(|_| BusinessError::invalid("stored subagent tool allowlist is invalid"))?,
        model_id: row.model_id,
        state: row.state,
        result: row
            .result_json
            .map(|value| serde_json::from_str(&value))
            .transpose()
            .map_err(|_| BusinessError::invalid("stored subagent result is invalid"))?,
        error_code: row.error_code,
        created_at: row.created_at,
        started_at: row.started_at,
        completed_at: row.completed_at,
    })
}

impl Store {
    #[allow(clippy::too_many_arguments)]
    pub fn create_subagent_invocation(
        &self,
        invocation_id: &str,
        parent_session_id: &str,
        parent_turn_id: &str,
        parent_tool_call_id: &str,
        child_session_id: &str,
        agent_id: &str,
        agent_version: i64,
        task: &Value,
        allowed_tools: &Value,
        model_id: &str,
    ) -> Result<SubagentInvocationRecord, BusinessError> {
        let mut connection = lock(&self.connection)?;
        let timestamp = now();
        sql_query("INSERT INTO subagent_invocation(invocation_id,parent_session_id,parent_turn_id,parent_tool_call_id,child_session_id,agent_id,agent_version,task_json,allowed_tools_json,model_id,state,result_json,error_code,created_at,started_at,completed_at) VALUES (?,?,?,?,?,?,?,?,?,?,'created',NULL,NULL,?,NULL,NULL)")
            .bind::<Text,_>(invocation_id)
            .bind::<Text,_>(parent_session_id)
            .bind::<Text,_>(parent_turn_id)
            .bind::<Text,_>(parent_tool_call_id)
            .bind::<Text,_>(child_session_id)
            .bind::<Text,_>(agent_id)
            .bind::<Integer,_>(i32::try_from(agent_version).map_err(|_| BusinessError::invalid("agent version is invalid"))?)
            .bind::<Text,_>(serde_json::to_string(task).map_err(|_| BusinessError::invalid("subagent task is invalid"))?)
            .bind::<Text,_>(serde_json::to_string(allowed_tools).map_err(|_| BusinessError::invalid("subagent tools are invalid"))?)
            .bind::<Text,_>(model_id)
            .bind::<Text,_>(&timestamp)
            .execute(&mut *connection)
            .map_err(crate::database_error)?;
        sql_query(format!("{SELECT} WHERE invocation_id=?"))
            .bind::<Text, _>(invocation_id)
            .get_result::<Row>(&mut *connection)
            .map_err(crate::database_error)
            .and_then(to_record)
    }

    pub fn update_subagent_invocation(
        &self,
        child_session_id: &str,
        state: &str,
        result: Option<&Value>,
        error_code: Option<&str>,
    ) -> Result<(), BusinessError> {
        let mut connection = lock(&self.connection)?;
        let timestamp = now();
        let started_at = matches!(state, "running").then_some(timestamp.clone());
        let completed_at = matches!(state, "completed" | "failed" | "cancelled" | "interrupted")
            .then_some(timestamp);
        sql_query("UPDATE subagent_invocation SET state=?,result_json=?,error_code=?,started_at=COALESCE(started_at,?),completed_at=? WHERE child_session_id=?")
            .bind::<Text,_>(state)
            .bind::<Nullable<Text>,_>(result.map(serde_json::to_string).transpose().map_err(|_| BusinessError::invalid("subagent result is invalid"))?)
            .bind::<Nullable<Text>,_>(error_code)
            .bind::<Nullable<Text>,_>(started_at)
            .bind::<Nullable<Text>,_>(completed_at)
            .bind::<Text,_>(child_session_id)
            .execute(&mut *connection)
            .map_err(crate::database_error)?;
        Ok(())
    }

    pub fn subagent_invocations_for_parent(
        &self,
        parent_session_id: &str,
    ) -> Result<Vec<SubagentInvocationRecord>, BusinessError> {
        let mut connection = lock(&self.connection)?;
        sql_query(format!(
            "{SELECT} WHERE parent_session_id=? ORDER BY created_at DESC,invocation_id"
        ))
        .bind::<Text, _>(parent_session_id)
        .load::<Row>(&mut *connection)
        .map_err(crate::database_error)?
        .into_iter()
        .map(to_record)
        .collect()
    }
}
