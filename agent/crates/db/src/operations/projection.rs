//! Cross-table event projection into normalized table-owned rows.

use crate::store::PersistenceError;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Integer, Nullable, Text};
use diesel::sqlite::SqliteConnection;
use serde_json::{to_string, Value};

pub(crate) fn apply(
    connection: &mut SqliteConnection,
    session_id: &str,
    occurred_at: &str,
    event_type: &str,
    payload: &Value,
) -> Result<(), PersistenceError> {
    if let Some(role) = event_type.strip_prefix("message.") {
        if matches!(role, "user" | "assistant" | "thinking") {
            let message = payload.get("message").ok_or_else(|| {
                PersistenceError::Invalid("message event is missing message".into())
            })?;
            let id = payload
                .get("message_id")
                .and_then(Value::as_str)
                .unwrap_or_else(|| payload.get("id").and_then(Value::as_str).unwrap_or(""));
            if id.is_empty() {
                return Err(PersistenceError::Invalid(
                    "message event is missing message_id".into(),
                ));
            }
            sql_query("INSERT OR IGNORE INTO session_message(message_id,session_id,turn_id,session_call_id,role,message_json,created_at) VALUES (?,?,?,?,?,?,?)")
                .bind::<Text, _>(id)
                .bind::<Text, _>(session_id)
                .bind::<Nullable<Text>, _>(payload.get("turn_id").and_then(Value::as_str))
                .bind::<Nullable<Text>, _>(payload.get("call_id").or_else(|| payload.get("exchange_id")).and_then(Value::as_str))
                .bind::<Text, _>(role)
                .bind::<Text, _>(&to_string(message)?)
                .bind::<Text, _>(occurred_at)
                .execute(connection)?;
        }
    }

    if event_type == "turn.state" {
        let turn_id = required(payload, "turn_id")?;
        let state = required(payload, "state")?;
        let terminal = matches!(state, "completed" | "failed" | "cancelled" | "interrupted");
        sql_query("INSERT INTO session_turn(turn_id,session_id,submission_idempotency_key,state,model_id,created_at,updated_at,completed_at,error_code) VALUES (?,?,?,?,?,?,?,?,?) ON CONFLICT(turn_id) DO UPDATE SET state=excluded.state,updated_at=excluded.updated_at,completed_at=excluded.completed_at,error_code=excluded.error_code,model_id=COALESCE(excluded.model_id,session_turn.model_id),submission_idempotency_key=COALESCE(excluded.submission_idempotency_key,session_turn.submission_idempotency_key)")
            .bind::<Text, _>(turn_id)
            .bind::<Text, _>(session_id)
            .bind::<Nullable<Text>, _>(payload.get("submission_idempotency_key").and_then(Value::as_str))
            .bind::<Text, _>(state)
            .bind::<Nullable<Text>, _>(payload.get("model_id").and_then(Value::as_str))
            .bind::<Text, _>(occurred_at)
            .bind::<Text, _>(occurred_at)
            .bind::<Nullable<Text>, _>(if terminal { Some(occurred_at) } else { None })
            .bind::<Nullable<Text>, _>(payload.get("reason").and_then(Value::as_str))
            .execute(connection)?;
    }

    if event_type == "usage.updated" {
        let usage = payload
            .get("usage")
            .ok_or_else(|| PersistenceError::Invalid("usage event is missing usage".into()))?;
        let input = usage_i64(usage, "input_tokens")?;
        let output = usage_i64(usage, "output_tokens")?;
        let total = usage_i64(usage, "total_tokens")?;
        sql_query("UPDATE session_turn SET input_tokens=?,output_tokens=?,total_tokens=?,updated_at=? WHERE turn_id=?")
            .bind::<Integer, _>(input).bind::<Integer, _>(output).bind::<Integer, _>(total)
            .bind::<Text, _>(occurred_at).bind::<Text, _>(required(payload, "turn_id")?).execute(connection)?;
    }

    if event_type == "provider.exchange.started" {
        let input = payload
            .get("input_messages")
            .cloned()
            .unwrap_or_else(|| Value::Array(Vec::new()));
        sql_query("INSERT INTO session_call(call_id,session_id,turn_id,provider,model_id,wire_model,state,iteration,started_at,input_messages_json,tool_calls_json) VALUES (?,?,?,?,? ,?,'started',?,?,?,?) ON CONFLICT(call_id) DO UPDATE SET input_messages_json=excluded.input_messages_json,state='started',started_at=excluded.started_at")
            .bind::<Text, _>(required_any(payload, &["exchange_id", "call_id"])?).bind::<Text, _>(session_id)
            .bind::<Text, _>(required(payload, "turn_id")?).bind::<Text, _>(required(payload, "provider")?)
            .bind::<Text, _>(required(payload, "model_id")?).bind::<Text, _>(required(payload, "wire_model")?)
            .bind::<Integer, _>(payload.get("iteration").and_then(Value::as_i64).unwrap_or(1) as i32)
            .bind::<Text, _>(occurred_at).bind::<Text, _>(&to_string(&input)?).bind::<Text, _>("[]").execute(connection)?;
    }

    if matches!(
        event_type,
        "provider.exchange.completed" | "provider.exchange.failed"
    ) {
        let state = if event_type.ends_with("failed") {
            "failed"
        } else {
            "completed"
        };
        let call_id = required_any(payload, &["exchange_id", "call_id"])?;
        sql_query("UPDATE session_call SET state=?,completed_at=?,provider_request_id=COALESCE(?,provider_request_id),provider_response_id=COALESCE(?,provider_response_id),output_message_json=?,tool_calls_json=?,usage_json=?,finish_reason=?,error_json=? WHERE call_id=?")
            .bind::<Text, _>(state).bind::<Text, _>(occurred_at)
            .bind::<Nullable<Text>, _>(payload.get("provider_request_id").and_then(Value::as_str))
            .bind::<Nullable<Text>, _>(payload.get("provider_response_id").and_then(Value::as_str))
            .bind::<Nullable<Text>, _>(payload.get("output_message").map(to_string).transpose()?)
            .bind::<Text, _>(&to_string(&payload.get("tool_calls").cloned().unwrap_or_else(|| Value::Array(Vec::new())))?)
            .bind::<Nullable<Text>, _>(payload.get("usage").map(to_string).transpose()?)
            .bind::<Nullable<Text>, _>(payload.get("finish_reason").and_then(Value::as_str))
            .bind::<Nullable<Text>, _>(payload.get("error").map(to_string).transpose()?.as_deref())
            .bind::<Text, _>(call_id).execute(connection)?;
    }

    if event_type == "todo.updated" {
        let turn_id = required(payload, "turn_id")?;
        let todos = payload
            .get("todos")
            .and_then(Value::as_array)
            .ok_or_else(|| PersistenceError::Invalid("todo event is missing todos".into()))?;
        if todos.len() > 100 {
            return Err(PersistenceError::Invalid(
                "todo event contains more than 100 items".into(),
            ));
        }
        sql_query("DELETE FROM session_turn_todo WHERE turn_id=?")
            .bind::<Text, _>(turn_id)
            .execute(connection)?;
        let mut in_progress = false;
        for (ordinal, todo) in todos.iter().enumerate() {
            let object = todo.as_object().ok_or_else(|| {
                PersistenceError::Invalid("todo event contains a non-object item".into())
            })?;
            let content = object
                .get("content")
                .and_then(Value::as_str)
                .filter(|v| !v.trim().is_empty() && v.chars().count() <= 500)
                .ok_or_else(|| PersistenceError::Invalid("todo content is required".into()))?;
            let status = object
                .get("status")
                .and_then(Value::as_str)
                .filter(|v| matches!(*v, "pending" | "in_progress" | "completed" | "cancelled"))
                .ok_or_else(|| PersistenceError::Invalid("todo status is invalid".into()))?;
            if status == "in_progress" && std::mem::replace(&mut in_progress, true) {
                return Err(PersistenceError::Invalid(
                    "todo event contains multiple in_progress items".into(),
                ));
            }
            let priority = object
                .get("priority")
                .and_then(Value::as_str)
                .filter(|v| matches!(*v, "high" | "medium" | "low"))
                .ok_or_else(|| PersistenceError::Invalid("todo priority is invalid".into()))?;
            sql_query("INSERT INTO session_turn_todo(turn_id,ordinal,content,status,priority,created_at,updated_at,completed_at) VALUES (?,?,?,?,?,?,?,?)")
                .bind::<Text, _>(turn_id).bind::<Integer, _>(ordinal as i32).bind::<Text, _>(content).bind::<Text, _>(status).bind::<Text, _>(priority).bind::<Text, _>(occurred_at).bind::<Text, _>(occurred_at).bind::<Nullable<Text>, _>(if matches!(status,"completed"|"cancelled") { Some(occurred_at) } else { None }).execute(connection)?;
        }
    }

    if event_type == "tool.requested" {
        if let (Some(turn_id), Some(tool_call_id), Some(name), Some(arguments)) = (
            payload.get("turn_id").and_then(Value::as_str),
            payload.get("tool_call_id").and_then(Value::as_str),
            payload.get("name").and_then(Value::as_str),
            payload.get("arguments"),
        ) {
            sql_query("INSERT INTO session_tool_use(turn_id,tool_call_id,session_call_id,name,request_json,state,ordinal,created_at,updated_at) VALUES (?,?,?,?,?,'requested',?,?,?) ON CONFLICT(turn_id,tool_call_id) DO UPDATE SET session_call_id=COALESCE(excluded.session_call_id,session_tool_use.session_call_id),request_json=excluded.request_json,name=excluded.name,ordinal=COALESCE(excluded.ordinal,session_tool_use.ordinal),updated_at=excluded.updated_at")
                .bind::<Text, _>(turn_id).bind::<Text, _>(tool_call_id).bind::<Nullable<Text>, _>(payload.get("call_id").and_then(Value::as_str)).bind::<Text, _>(name).bind::<Text, _>(&to_string(arguments)?).bind::<Nullable<Integer>, _>(payload.get("ordinal").and_then(Value::as_i64).map(|v|v as i32)).bind::<Text, _>(occurred_at).bind::<Text, _>(occurred_at).execute(connection)?;
        }
    }
    if event_type == "tool.state" {
        if let (Some(turn_id), Some(tool_call_id), Some(name), Some(state)) = (
            payload.get("turn_id").and_then(Value::as_str),
            payload.get("tool_call_id").and_then(Value::as_str),
            payload.get("name").and_then(Value::as_str),
            payload.get("state").and_then(Value::as_str),
        ) {
            let terminal = matches!(state, "denied" | "succeeded" | "failed" | "timed_out");
            sql_query("INSERT INTO session_tool_use(turn_id,tool_call_id,session_call_id,name,state,ordinal,created_at,updated_at,completed_at,error_code) VALUES (?,?,?,?,?,?,?,?,?,?) ON CONFLICT(turn_id,tool_call_id) DO UPDATE SET session_call_id=COALESCE(excluded.session_call_id,session_tool_use.session_call_id),name=excluded.name,state=excluded.state,ordinal=COALESCE(excluded.ordinal,session_tool_use.ordinal),updated_at=excluded.updated_at,completed_at=excluded.completed_at,error_code=excluded.error_code")
                .bind::<Text, _>(turn_id).bind::<Text, _>(tool_call_id).bind::<Nullable<Text>, _>(payload.get("call_id").and_then(Value::as_str)).bind::<Text, _>(name).bind::<Text, _>(state).bind::<Nullable<Integer>, _>(payload.get("ordinal").and_then(Value::as_i64).map(|v|v as i32)).bind::<Text, _>(occurred_at).bind::<Text, _>(occurred_at).bind::<Nullable<Text>, _>(if terminal {Some(occurred_at)} else {None}).bind::<Nullable<Text>, _>(payload.get("reason").and_then(Value::as_str)).execute(connection)?;
        }
    }
    if event_type == "tool.result" {
        if let (Some(turn_id), Some(tool_call_id), Some(result)) = (
            payload.get("turn_id").and_then(Value::as_str),
            payload.get("tool_call_id").and_then(Value::as_str),
            payload.get("result"),
        ) {
            sql_query("UPDATE session_tool_use SET result_json=?,updated_at=? WHERE turn_id=? AND tool_call_id=?").bind::<Text,_>(&to_string(result)?).bind::<Text,_>(occurred_at).bind::<Text,_>(turn_id).bind::<Text,_>(tool_call_id).execute(connection)?;
        }
    }
    if event_type == "checkpoint.captured" {
        if let Some(id) = payload.get("checkpoint_id").and_then(Value::as_str) {
            sql_query("INSERT OR IGNORE INTO checkpoint(checkpoint_id,manifest_id,session_id,turn_id,tool_call_id,relative_path,status,created_at,ordinal) VALUES (?,?,?,?,?,?,'available',?,?)").bind::<Text,_>(id).bind::<Nullable<Text>,_>(payload.get("manifest_id").and_then(Value::as_str)).bind::<Text,_>(session_id).bind::<Nullable<Text>,_>(payload.get("turn_id").and_then(Value::as_str)).bind::<Nullable<Text>,_>(payload.get("tool_call_id").and_then(Value::as_str)).bind::<Nullable<Text>,_>(payload.get("path").and_then(Value::as_str)).bind::<Text,_>(occurred_at).bind::<Nullable<Integer>,_>(payload.get("ordinal").and_then(Value::as_i64).map(|v|v as i32)).execute(connection)?;
        }
    }
    if event_type == "checkpoint.item_restored" {
        if let Some(id) = payload.get("checkpoint_id").and_then(Value::as_str) {
            sql_query("UPDATE checkpoint SET status='restored',restored_at=? WHERE checkpoint_id=? AND session_id=?").bind::<Text,_>(occurred_at).bind::<Text,_>(id).bind::<Text,_>(session_id).execute(connection)?;
        }
    }
    Ok(())
}

fn required<'a>(payload: &'a Value, name: &str) -> Result<&'a str, PersistenceError> {
    payload
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| PersistenceError::Invalid(format!("event is missing {name}")))
}
fn required_any<'a>(payload: &'a Value, names: &[&str]) -> Result<&'a str, PersistenceError> {
    names
        .iter()
        .find_map(|name| payload.get(*name).and_then(Value::as_str))
        .ok_or_else(|| PersistenceError::Invalid("event is missing exchange id".into()))
}
fn usage_i64(usage: &Value, field: &str) -> Result<i32, PersistenceError> {
    let value = usage
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| PersistenceError::Invalid(format!("usage event has invalid {field}")))?;
    i32::try_from(value)
        .map_err(|_| PersistenceError::Invalid(format!("usage event {field} exceeds SQLite")))
}
