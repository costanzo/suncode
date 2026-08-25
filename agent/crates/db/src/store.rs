use super::{data, schema};
use crate::domain::*;
use chrono::{Duration, Utc};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde_json::{json, Value};
use std::path::Path;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("database error: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("invalid database: {0}")]
    Invalid(String),
    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone)]
pub struct Store {
    connection: Arc<Mutex<Connection>>,
}

type SubmissionRow = (
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);
type ApprovalRow = (
    String,
    Option<String>,
    String,
    String,
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    String,
    String,
);

pub struct ApprovalInput<'a> {
    pub project_id: Option<&'a str>,
    pub session_id: &'a str,
    pub turn_id: &'a str,
    pub tool_call_id: &'a str,
    pub operation: &'a str,
    pub arguments: &'a Value,
    pub snapshot: &'a Value,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self, PersistenceError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| PersistenceError::Invalid(error.to_string()))?;
        }
        let mut connection = Connection::open(path)?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "busy_timeout", 5000i64)?;
        initialize(&mut connection)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub fn open_memory() -> Result<Self, PersistenceError> {
        let mut connection = Connection::open_in_memory()?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        initialize(&mut connection)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub fn health(&self) -> Result<Value, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let journal_mode: String =
            connection.query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
        Ok(json!({"ok": true, "journal_mode": journal_mode}))
    }

    pub fn append_content(
        &self,
        session_id: &str,
        event_type: &str,
        payload: &Value,
    ) -> Result<SessionEvent, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let transaction = connection.unchecked_transaction()?;
        let now = now();
        apply_projection(&transaction, session_id, &now, event_type, payload)?;
        transaction.execute(
            "UPDATE session SET updated_at = ?, last_activity_at = ? WHERE session_id = ?",
            params![now, now, session_id],
        )?;
        transaction.commit()?;
        Ok(SessionEvent {
            session_id: session_id.to_string(),
            occurred_at: now,
            event_type: event_type.to_string(),
            payload: payload.clone(),
        })
    }

    pub fn messages(&self, session_id: &str) -> Result<Vec<Message>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let mut statement = connection.prepare(
            "SELECT message_json FROM session_message
             WHERE session_id=? AND role IN ('user','assistant','thinking')
             ORDER BY created_at,rowid",
        )?;
        let rows = statement.query_map([session_id], |row| row.get::<_, String>(0))?;
        rows.map(|row| Ok(serde_json::from_str(&row?)?)).collect()
    }

    pub fn context_messages(&self, session_id: &str) -> Result<Vec<Message>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let mut statement = connection.prepare(
            "SELECT kind,payload,tool_call_id
             FROM (
                 SELECT message.created_at AS occurred_at,0 AS kind_order,
                        message.rowid AS stable_order,
                        'message' AS kind,message.message_json AS payload,NULL AS tool_call_id,
                        NULL AS ordinal,COALESCE(call.iteration,0) AS call_iteration
                 FROM session_message AS message
                 LEFT JOIN session_call AS call ON call.call_id=message.session_call_id
                 WHERE message.session_id=?1 AND role IN ('user','assistant','thinking')
                 UNION ALL
                 SELECT COALESCE(tool.completed_at,tool.updated_at) AS occurred_at,
                        1 AS kind_order,tool.rowid AS stable_order,
                        'tool' AS kind,tool.result_json AS payload,tool.tool_call_id,
                        tool.ordinal,COALESCE(call.iteration,9223372036854775807) AS call_iteration
                 FROM session_tool_use AS tool
                 JOIN session_turn AS turn ON turn.turn_id=tool.turn_id
                 LEFT JOIN session_call AS call ON call.call_id=tool.session_call_id
                 WHERE turn.session_id=?1 AND tool.state IN ('succeeded','failed')
                   AND tool.result_json IS NOT NULL
             )
             ORDER BY occurred_at,call_iteration,kind_order,
                      COALESCE(ordinal,9223372036854775807),stable_order",
        )?;
        let rows = statement.query_map([session_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })?;
        let mut messages: Vec<Message> = Vec::new();
        for row in rows {
            let (kind, payload, tool_call_id) = row?;
            if kind == "message" {
                messages.push(serde_json::from_str(&payload)?);
            } else {
                let mut message = Message::text("tool", payload);
                message.tool_call_id = tool_call_id;
                messages.push(message);
            }
        }
        Ok(repair_incomplete_tool_exchanges(messages))
    }

    pub fn session_usage(&self, session_id: &str) -> Result<Usage, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let (input_tokens, output_tokens, total_tokens): (i64, i64, i64) = connection.query_row(
            "SELECT COALESCE(SUM(input_tokens),0),
                        COALESCE(SUM(output_tokens),0),
                        COALESCE(SUM(total_tokens),0)
                FROM session_turn
                 WHERE session_id=?",
            [session_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        Ok(Usage {
            input_tokens: usage_from_sql(input_tokens)?,
            output_tokens: usage_from_sql(output_tokens)?,
            total_tokens: usage_from_sql(total_tokens)?,
        })
    }

    pub fn provider_exchanges(
        &self,
        session_id: &str,
    ) -> Result<Vec<ProviderExchange>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let mut statement = connection.prepare(
            "SELECT call_id,session_id,turn_id,provider,model_id,wire_model,provider_request_id,provider_response_id,state,iteration,started_at,completed_at,input_messages_json,output_message_json,tool_calls_json,usage_json,finish_reason,error_json
             FROM session_call
             WHERE session_id=?
             ORDER BY started_at DESC, call_id DESC",
        )?;
        let rows = statement.query_map([session_id], provider_exchange_from_row)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn provider_exchange(
        &self,
        session_id: &str,
        exchange_id: &str,
    ) -> Result<Option<ProviderExchange>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        connection
            .query_row(
                "SELECT call_id,session_id,turn_id,provider,model_id,wire_model,provider_request_id,provider_response_id,state,iteration,started_at,completed_at,input_messages_json,output_message_json,tool_calls_json,usage_json,finish_reason,error_json
                 FROM session_call
                 WHERE session_id=? AND call_id=?",
                params![session_id, exchange_id],
                provider_exchange_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn session_trace_turns(
        &self,
        session_id: &str,
    ) -> Result<Vec<SessionTraceTurn>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let mut statement = connection.prepare(
            "SELECT turn_id,session_id,state,model_id,created_at,updated_at,started_at,completed_at,error_code,input_tokens,output_tokens,total_tokens
             FROM session_turn
             WHERE session_id=?
             ORDER BY created_at DESC,turn_id DESC",
        )?;
        let rows = statement.query_map([session_id], session_trace_turn_from_row)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn session_conversation_turns(
        &self,
        session_id: &str,
    ) -> Result<Vec<SessionConversationTurn>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let mut turn_statement = connection.prepare(
            "SELECT turn_id,state,created_at
             FROM session_turn
             WHERE session_id=?
             ORDER BY created_at,turn_id",
        )?;
        let turn_rows = turn_statement.query_map([session_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        let turns = turn_rows.collect::<Result<Vec<_>, _>>()?;
        let mut result = Vec::with_capacity(turns.len());
        for (turn_id, state, created_at) in turns {
            let mut message_statement = connection.prepare(
                "SELECT message_id,session_id,turn_id,session_call_id,role,message_json,created_at
                 FROM session_message
                 WHERE session_id=? AND turn_id=? AND role IN ('user','assistant','thinking')
                 ORDER BY created_at,rowid",
            )?;
            let message_rows =
                message_statement.query_map(params![session_id, turn_id], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                    ))
                })?;
            let messages = message_rows
                .map(|row| {
                    let (
                        message_id,
                        session_id,
                        turn_id,
                        session_call_id,
                        role,
                        message,
                        created_at,
                    ) = row?;
                    Ok(SessionCallMessage {
                        message_id,
                        session_id,
                        turn_id,
                        session_call_id,
                        role,
                        message: serde_json::from_str(&message)?,
                        created_at,
                    })
                })
                .collect::<Result<Vec<_>, PersistenceError>>()?;

            let mut tool_statement = connection.prepare(
                "SELECT turn_id,tool_call_id,session_call_id,name,request_json,result_json,state,ordinal,created_at,updated_at,completed_at,error_code
                 FROM session_tool_use
                 WHERE turn_id=?
                 ORDER BY created_at,COALESCE(ordinal,9223372036854775807),tool_call_id",
            )?;
            let tool_rows = tool_statement.query_map([&turn_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, Option<i64>>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, Option<String>>(10)?,
                    row.get::<_, Option<String>>(11)?,
                ))
            })?;
            let tool_uses = tool_rows
                .map(|row| {
                    let (
                        turn_id,
                        tool_call_id,
                        session_call_id,
                        name,
                        request,
                        result,
                        state,
                        ordinal,
                        created_at,
                        updated_at,
                        completed_at,
                        error_code,
                    ) = row?;
                    Ok(SessionCallToolUse {
                        turn_id,
                        tool_call_id,
                        session_call_id,
                        name,
                        request: request
                            .map(|value| serde_json::from_str(&value))
                            .transpose()?,
                        result: result
                            .map(|value| serde_json::from_str(&value))
                            .transpose()?,
                        state,
                        ordinal,
                        created_at,
                        updated_at,
                        completed_at,
                        error_code,
                    })
                })
                .collect::<Result<Vec<_>, PersistenceError>>()?;
            let mut todo_statement = connection.prepare(
                "SELECT turn_id,ordinal,content,status,priority,created_at,updated_at,completed_at
                 FROM session_turn_todo
                 WHERE turn_id=?
                 ORDER BY ordinal",
            )?;
            let todo_rows = todo_statement.query_map([&turn_id], |row| {
                Ok(SessionTurnTodo {
                    turn_id: row.get(0)?,
                    ordinal: row.get(1)?,
                    content: row.get(2)?,
                    status: row.get(3)?,
                    priority: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                    completed_at: row.get(7)?,
                })
            })?;
            let todos = todo_rows.collect::<Result<Vec<_>, _>>()?;
            result.push(SessionConversationTurn {
                turn_id,
                state,
                created_at,
                messages,
                tool_uses,
                todos,
            });
        }
        Ok(result)
    }

    pub fn session_call_messages(
        &self,
        session_id: &str,
        call_id: &str,
    ) -> Result<Vec<SessionCallMessage>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let mut statement = connection.prepare(
            "SELECT message_id,session_id,turn_id,session_call_id,role,message_json,created_at
             FROM session_message
             WHERE session_id=? AND session_call_id=?
               AND role IN ('user','assistant','thinking')
             ORDER BY created_at,rowid",
        )?;
        let rows = statement.query_map(params![session_id, call_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })?;
        rows.map(|row| {
            let (message_id, session_id, turn_id, session_call_id, role, message, created_at) =
                row?;
            Ok(SessionCallMessage {
                message_id,
                session_id,
                turn_id,
                session_call_id,
                role,
                message: serde_json::from_str(&message)?,
                created_at,
            })
        })
        .collect()
    }

    pub fn session_call_tool_uses(
        &self,
        call_id: &str,
    ) -> Result<Vec<SessionCallToolUse>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let mut statement = connection.prepare(
            "SELECT turn_id,tool_call_id,session_call_id,name,request_json,result_json,state,ordinal,created_at,updated_at,completed_at,error_code
             FROM session_tool_use
             WHERE session_call_id=?
             ORDER BY COALESCE(ordinal,9223372036854775807),created_at,tool_call_id",
        )?;
        let rows = statement.query_map([call_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<i64>>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, Option<String>>(10)?,
                row.get::<_, Option<String>>(11)?,
            ))
        })?;
        rows.map(|row| {
            let (
                turn_id,
                tool_call_id,
                session_call_id,
                name,
                request,
                result,
                state,
                ordinal,
                created_at,
                updated_at,
                completed_at,
                error_code,
            ) = row?;
            Ok(SessionCallToolUse {
                turn_id,
                tool_call_id,
                session_call_id,
                name,
                request: request
                    .map(|value| serde_json::from_str(&value))
                    .transpose()?,
                result: result
                    .map(|value| serde_json::from_str(&value))
                    .transpose()?,
                state,
                ordinal,
                created_at,
                updated_at,
                completed_at,
                error_code,
            })
        })
        .collect()
    }

    pub fn begin_turn(
        &self,
        session_id: &str,
        key: &str,
        input: &str,
        model: &str,
    ) -> Result<TurnAdmission, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let existing: Option<SubmissionRow> = connection.query_row(
            "SELECT state,turn_id,input_json,model_id,response_json,error_json FROM session_turn WHERE session_id=? AND submission_idempotency_key=?",
            params![session_id, key],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
        ).optional()?;
        if let Some((status, turn_id, stored_input, stored_model, response, _error)) = existing {
            let requested = serde_json::to_string(&json!({"input": input}))?;
            if stored_input.as_deref() != Some(requested.as_str())
                || stored_model.as_deref() != Some(model)
            {
                return Err(PersistenceError::Invalid(
                    "idempotency key was reused with different turn input".into(),
                ));
            }
            return Ok(TurnAdmission {
                created: false,
                turn_id,
                status,
                response: response
                    .map(|value| serde_json::from_str(&value))
                    .transpose()?,
            });
        }
        let session = self
            .session_by_id_optional_locked(&connection, session_id)?
            .ok_or_else(|| PersistenceError::Invalid("session not found".into()))?;
        if session.status != "active" {
            return Err(PersistenceError::Invalid("session is archived".into()));
        }
        let turn_id = Uuid::new_v4().to_string();
        let timestamp = now();
        connection.execute("INSERT INTO session_turn (session_id,submission_idempotency_key,state,created_at,updated_at,turn_id,input_json,model_id,admitted_at) VALUES (?,?, 'admitted',?,?,?,?,?,?)", params![session_id, key, timestamp, timestamp, turn_id, serde_json::to_string(&json!({"input": input}))?, model, timestamp])?;
        Ok(TurnAdmission {
            created: true,
            turn_id,
            status: "pending".into(),
            response: None,
        })
    }

    pub fn mark_turn_started(&self, session_id: &str, key: &str) -> Result<(), PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        connection.execute("UPDATE session_turn SET started_at=COALESCE(started_at,?),updated_at=? WHERE session_id=? AND submission_idempotency_key=? AND state NOT IN ('completed','failed','cancelled','interrupted')", params![now(), now(), session_id, key])?;
        Ok(())
    }

    pub fn complete_turn(
        &self,
        session_id: &str,
        key: &str,
        response: &Value,
    ) -> Result<(), PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let timestamp = now();
        connection.execute("UPDATE session_turn SET state='completed',response_json=?,error_json=NULL,completed_at=?,updated_at=? WHERE session_id=? AND submission_idempotency_key=? AND state NOT IN ('completed','failed','cancelled','interrupted')", params![serde_json::to_string(response)?, timestamp, timestamp, session_id, key])?;
        Ok(())
    }

    pub fn fail_turn(
        &self,
        session_id: &str,
        key: &str,
        error: &Value,
    ) -> Result<(), PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let timestamp = now();
        let error_code = error.get("code").and_then(Value::as_str);
        connection.execute(
            "UPDATE session_turn
             SET state='failed',error_json=?,error_code=COALESCE(?,error_code),
                 completed_at=COALESCE(completed_at,?),updated_at=?
             WHERE session_id=? AND submission_idempotency_key=?
               AND state NOT IN ('completed','cancelled','interrupted')",
            params![
                serde_json::to_string(error)?,
                error_code,
                timestamp,
                timestamp,
                session_id,
                key
            ],
        )?;
        Ok(())
    }

    pub fn append_audit(
        &self,
        project_id: Option<&str>,
        session_id: Option<&str>,
        turn_id: Option<&str>,
        event_type: &str,
        payload: &Value,
    ) -> Result<(), PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        connection.execute("INSERT INTO audit_record(project_id,session_id,turn_id,occurred_at,event_type,payload_json) VALUES (?,?,?,?,?,?)", params![project_id, session_id, turn_id, now(), event_type, serde_json::to_string(payload)?])?;
        Ok(())
    }

    pub fn create_approval(
        &self,
        input: ApprovalInput<'_>,
    ) -> Result<ApprovalRecord, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let transaction = connection.unchecked_transaction()?;
        let key = format!(
            "{}:{}:{}:approval",
            input.session_id, input.turn_id, input.tool_call_id
        );
        if let Some(id) = transaction
            .query_row(
                "SELECT approval_id FROM approval_request WHERE idempotency_key=?",
                [&key],
                |row| row.get::<_, String>(0),
            )
            .optional()?
        {
            let approval = approval_by_id(&transaction, &id)?
                .ok_or_else(|| PersistenceError::Invalid("approval disappeared".into()))?;
            transaction.commit()?;
            return Ok(approval);
        }
        let id = Uuid::new_v4().to_string();
        let timestamp = now();
        transaction.execute("INSERT INTO approval_request(approval_id,project_id,session_id,turn_id,tool_call_id,operation,arguments_json,idempotency_key,status,created_at,updated_at) VALUES (?,?,?,?,?,?,?,?, 'pending',?,?)", params![id, input.project_id, input.session_id, input.turn_id, input.tool_call_id, input.operation, serde_json::to_string(input.arguments)?, key, timestamp, timestamp])?;
        transaction.execute("UPDATE session_turn SET recovery_approval_id=?,recovery_snapshot_json=?,recovery_status='pending',recovery_created_at=?,recovery_updated_at=? WHERE turn_id=?", params![id, serde_json::to_string(input.snapshot)?, timestamp, timestamp, input.turn_id])?;
        let approval = approval_by_id(&transaction, &id)?
            .ok_or_else(|| PersistenceError::Invalid("approval creation failed".into()))?;
        transaction.commit()?;
        Ok(approval)
    }

    pub fn approval(&self, id: &str) -> Result<Option<ApprovalRecord>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        approval_by_id(&connection, id)
    }

    pub fn create_question(
        &self,
        request_id: &str,
        turn_id: &str,
        snapshot: &Value,
    ) -> Result<(), PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let timestamp = now();
        let changed = connection.execute(
            "UPDATE session_turn SET recovery_approval_id=?,recovery_snapshot_json=?,recovery_status='pending',recovery_created_at=?,recovery_updated_at=? WHERE turn_id=? AND (recovery_status IS NULL OR recovery_status='resuming')",
            params![request_id, serde_json::to_string(snapshot)?, timestamp, timestamp, turn_id],
        )?;
        if changed == 0 {
            return Err(PersistenceError::Invalid(
                "question recovery is already pending or turn is unavailable".into(),
            ));
        }
        Ok(())
    }

    pub fn pending_question(&self, session_id: &str) -> Result<Option<Value>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let row: Option<(String, String)> = connection
            .query_row(
                "SELECT recovery_approval_id,recovery_snapshot_json FROM session_turn WHERE session_id=? AND recovery_status='pending'",
                [session_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let Some((request_id, snapshot)) = row else {
            return Ok(None);
        };
        let snapshot: Value = serde_json::from_str(&snapshot)?;
        let Some(call) = snapshot
            .get("pending_call")
            .filter(|call| call.get("name").and_then(Value::as_str) == Some("question"))
        else {
            return Ok(None);
        };
        Ok(Some(json!({
            "request_id": request_id,
            "session_id": session_id,
            "turn_id": snapshot.get("turn_id"),
            "tool_call_id": call.get("call_id"),
            "questions": call.get("arguments").and_then(|value| value.get("questions")).cloned().unwrap_or_else(|| json!([])),
        })))
    }

    pub fn question_snapshot(&self, request_id: &str) -> Result<Option<Value>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        connection
            .query_row(
                "SELECT recovery_snapshot_json FROM session_turn WHERE recovery_approval_id=? AND recovery_status='pending'",
                [request_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .map(|value| serde_json::from_str(&value))
            .transpose()
            .map_err(Into::into)
    }

    pub fn resolve_question(
        &self,
        id: &str,
        answers: &[Vec<String>],
        rejected: bool,
    ) -> Result<Option<SuspendedTurn>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let transaction = connection.unchecked_transaction()?;
        let row: Option<(String, String, String, String)> = transaction
            .query_row(
                "SELECT recovery_approval_id,session_id,turn_id,recovery_snapshot_json FROM session_turn WHERE recovery_approval_id=? AND recovery_status='pending'",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()?;
        let Some(row) = row else {
            transaction.rollback()?;
            return Ok(None);
        };
        let mut snapshot: Value = serde_json::from_str(&row.3)?;
        snapshot["question_answers"] = serde_json::to_value(answers)?;
        snapshot["question_rejected"] = json!(rejected);
        transaction.execute(
            "UPDATE session_turn SET recovery_status='resuming',recovery_snapshot_json=?,recovery_updated_at=? WHERE recovery_approval_id=? AND recovery_status='pending'",
            params![serde_json::to_string(&snapshot)?, now(), id],
        )?;
        transaction.commit()?;
        Ok(Some(SuspendedTurn {
            approval_id: row.0,
            session_id: row.1,
            turn_id: row.2,
            snapshot,
            status: "resuming".into(),
        }))
    }

    pub fn resolve_approval(
        &self,
        id: &str,
        decision: &str,
    ) -> Result<Option<SuspendedTurn>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let transaction = connection.unchecked_transaction()?;
        let approved = decision != "deny";
        let changed = transaction.execute("UPDATE approval_request SET status=?,decision=?,decision_source='user',updated_at=? WHERE approval_id=? AND status='pending'", params![if approved {"approved"} else {"denied"}, decision, now(), id])?;
        if changed == 0 {
            transaction.rollback()?;
            return Ok(None);
        }
        let row: (String, String, String, String) = transaction.query_row("SELECT recovery_approval_id,session_id,turn_id,recovery_snapshot_json FROM session_turn JOIN session USING(session_id) WHERE recovery_approval_id=? AND recovery_status='pending'", [id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?)))?;
        if decision == "allow_session" {
            transaction.execute(
                "INSERT INTO configuration(scope,session_id,key,value_json,updated_at) VALUES ('session',?,'full_control','true',?) ON CONFLICT(session_id,key) WHERE scope='session' DO UPDATE SET value_json='true',updated_at=excluded.updated_at",
                params![&row.1, now()],
            )?;
            transaction.execute(
                "INSERT INTO audit_record(project_id,session_id,turn_id,occurred_at,event_type,payload_json) VALUES (?,?,?,?,?,?)",
                params![Option::<String>::None, &row.1, &row.2, now(), "session.full_control.changed", "{\"enabled\":true,\"source\":\"approval\"}"],
            )?;
        }
        transaction.execute(
            "UPDATE session_turn SET recovery_status=?,recovery_updated_at=? WHERE recovery_approval_id=?",
            params![if approved { "resuming" } else { "denied" }, now(), id],
        )?;
        transaction.commit()?;
        Ok(Some(SuspendedTurn {
            approval_id: row.0,
            session_id: row.1,
            turn_id: row.2,
            snapshot: serde_json::from_str(&row.3)?,
            status: if approved {
                "resuming".into()
            } else {
                "denied".into()
            },
        }))
    }

    pub fn finish_suspended(&self, id: &str, status: &str) -> Result<(), PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let terminal = matches!(status, "completed" | "denied" | "failed");
        connection.execute(
            "UPDATE session_turn
             SET recovery_status=?,recovery_updated_at=?,recovery_snapshot_json=CASE WHEN ? THEN '{}' ELSE recovery_snapshot_json END
             WHERE recovery_approval_id=?",
            params![status, now(), terminal, id],
        )?;
        Ok(())
    }

    pub fn ensure_manifest(
        &self,
        session_id: &str,
        turn_id: &str,
    ) -> Result<CheckpointManifest, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        if let Some(value) = manifest_by_turn(&connection, turn_id)? {
            return Ok(value);
        }
        let id = Uuid::new_v4().to_string();
        let timestamp = now();
        let expires =
            (Utc::now() + Duration::days(30)).to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        connection.execute("INSERT INTO checkpoint_manifest(manifest_id,session_id,turn_id,status,created_at,updated_at,expires_at) VALUES (?,?,?,'available',?,?,?)", params![id, session_id, turn_id, timestamp, timestamp, expires])?;
        manifest_by_turn(&connection, turn_id)?
            .ok_or_else(|| PersistenceError::Invalid("checkpoint manifest creation failed".into()))
    }

    pub fn manifests(&self, session_id: &str) -> Result<Vec<CheckpointManifest>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        connection.execute("UPDATE checkpoint_manifest SET status='expired',updated_at=? WHERE session_id=? AND status='available' AND expires_at<=?", params![now(), session_id, now()])?;
        let mut statement = connection.prepare("SELECT manifest_id,session_id,turn_id,status,created_at,updated_at,expires_at,restored_at FROM checkpoint_manifest WHERE session_id=? ORDER BY created_at DESC")?;
        let rows = statement.query_map([session_id], manifest_from_row)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn manifest(&self, id: &str) -> Result<Option<CheckpointManifest>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        connection.query_row("SELECT manifest_id,session_id,turn_id,status,created_at,updated_at,expires_at,restored_at FROM checkpoint_manifest WHERE manifest_id=?", [id], manifest_from_row).optional().map_err(Into::into)
    }

    pub fn checkpoint_items(
        &self,
        manifest_id: &str,
    ) -> Result<Vec<CheckpointItem>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let mut statement = connection.prepare("SELECT checkpoint_id,manifest_id,session_id,turn_id,tool_call_id,relative_path,status,created_at,restored_at,invalidated_at,ordinal FROM checkpoint WHERE manifest_id=? ORDER BY ordinal DESC")?;
        let rows = statement.query_map([manifest_id], checkpoint_from_row)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn set_manifest_status(&self, id: &str, status: &str) -> Result<(), PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        connection.execute("UPDATE checkpoint_manifest SET status=?,updated_at=?,restored_at=CASE WHEN ?='restored' THEN ? ELSE restored_at END WHERE manifest_id=?", params![status, now(), status, now(), id])?;
        Ok(())
    }

    pub fn settings(
        &self,
        project_id: Option<&str>,
        session_id: Option<&str>,
    ) -> Result<Vec<SettingRecord>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let mut records = Vec::new();
        let mut statement = connection.prepare(
            "SELECT scope,COALESCE(project_id,session_id,'global'),key,value_json
             FROM configuration
             WHERE scope='global'
                OR (scope='project' AND project_id=?1)
                OR (scope='session' AND session_id=?2)
             ORDER BY CASE scope WHEN 'global' THEN 0 WHEN 'project' THEN 1 ELSE 2 END,key",
        )?;
        let rows = statement.query_map(params![project_id, session_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;
        for row in rows {
            let (scope, scope_id, key, value) = row?;
            records.retain(|record: &SettingRecord| record.key != key);
            records.push(SettingRecord {
                key,
                value: serde_json::from_str(&value)?,
                scope,
                scope_id,
            });
        }
        Ok(records)
    }

    pub fn session_full_control(&self, session_id: &str) -> Result<bool, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let value = connection
            .query_row(
                "SELECT value_json FROM configuration WHERE scope='session' AND session_id=? AND key='full_control'",
                [session_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        match value {
            None => Ok(false),
            Some(value) => serde_json::from_str::<Value>(&value)?
                .as_bool()
                .ok_or_else(|| {
                    PersistenceError::Invalid(
                        "session full_control configuration must be a boolean".into(),
                    )
                }),
        }
    }

    pub fn project_default_model(
        &self,
        project_id: &str,
    ) -> Result<Option<String>, PersistenceError> {
        let Some(value) = self
            .settings(Some(project_id), None)?
            .into_iter()
            .find(|record| record.key == "default_model")
            .map(|record| record.value)
        else {
            return Ok(None);
        };
        let model = value.as_str().ok_or_else(|| {
            PersistenceError::Invalid("project default_model must be a string".into())
        })?;
        if model.trim().is_empty() {
            return Ok(None);
        }
        Ok(Some(model.trim().to_string()))
    }

    pub fn project_tool_call_limit(
        &self,
        project_id: &str,
    ) -> Result<Option<u32>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let value = connection
            .query_row(
                "SELECT value_json FROM configuration
                 WHERE scope='project' AND project_id=? AND key='tool_call_limit'",
                [project_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        let Some(value) = value else {
            return Ok(None);
        };
        let value = serde_json::from_str::<Value>(&value)?;
        let limit = value.as_u64().ok_or_else(|| {
            PersistenceError::Invalid(
                "project tool_call_limit must be an integer between 1 and 256".into(),
            )
        })?;
        if !(1..=256).contains(&limit) {
            return Err(PersistenceError::Invalid(
                "project tool_call_limit must be an integer between 1 and 256".into(),
            ));
        }
        Ok(Some(limit as u32))
    }

    pub fn llm_model_providers(
        &self,
        enabled_only: bool,
    ) -> Result<Vec<LlmModelProviderRecord>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let sql = if enabled_only {
            "SELECT provider_id,display_name,endpoint,adapter_type,(api_key IS NOT NULL AND length(api_key) > 0),enabled,sort_order,created_at,updated_at
             FROM llm_model_provider WHERE enabled=1 ORDER BY sort_order,provider_id"
        } else {
            "SELECT provider_id,display_name,endpoint,adapter_type,(api_key IS NOT NULL AND length(api_key) > 0),enabled,sort_order,created_at,updated_at
             FROM llm_model_provider ORDER BY sort_order,provider_id"
        };
        let mut statement = connection.prepare(sql)?;
        let rows = statement.query_map([], llm_model_provider_from_row)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn upsert_llm_model_provider(
        &self,
        input: LlmModelProviderInput<'_>,
    ) -> Result<(), PersistenceError> {
        let provider_id = input.provider_id.trim();
        let display_name = input.display_name.trim();
        let endpoint = input.endpoint.trim().trim_end_matches('/');
        let adapter_type = input.adapter_type.trim();
        if provider_id.is_empty()
            || display_name.is_empty()
            || endpoint.is_empty()
            || adapter_type != "openai"
            || input.sort_order < 0
        {
            return Err(PersistenceError::Invalid(
                "model provider has invalid fields".into(),
            ));
        }
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let timestamp = now();
        connection.execute(
            "INSERT INTO llm_model_provider(provider_id,display_name,endpoint,adapter_type,api_key,enabled,sort_order,created_at,updated_at)
             VALUES (?,?,?,?,?,?,?,?,?)
             ON CONFLICT(provider_id) DO UPDATE SET display_name=excluded.display_name,endpoint=excluded.endpoint,adapter_type=excluded.adapter_type,enabled=excluded.enabled,sort_order=excluded.sort_order,updated_at=excluded.updated_at",
            params![
                provider_id,
                display_name,
                endpoint,
                adapter_type,
                Option::<String>::None,
                input.enabled,
                input.sort_order,
                timestamp,
                timestamp,
            ],
        )?;
        Ok(())
    }

    pub fn llm_models(&self, enabled_only: bool) -> Result<Vec<LlmModelRecord>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let sql = if enabled_only {
            "SELECT model_id,provider_id,display_name,request_model,context_tokens,auto_compact_tokens,max_output_tokens,supports_streaming,supports_tool_use,supports_vision,supports_structured_output,supports_cancellation,supports_reasoning_effort,enabled,sort_order,created_at,updated_at
             FROM llm_model WHERE enabled=1 ORDER BY sort_order,model_id"
        } else {
            "SELECT model_id,provider_id,display_name,request_model,context_tokens,auto_compact_tokens,max_output_tokens,supports_streaming,supports_tool_use,supports_vision,supports_structured_output,supports_cancellation,supports_reasoning_effort,enabled,sort_order,created_at,updated_at
             FROM llm_model ORDER BY sort_order,model_id"
        };
        let mut statement = connection.prepare(sql)?;
        let rows = statement.query_map([], llm_model_from_row)?;
        rows.map(|row| row.map_err(PersistenceError::from))
            .collect()
    }

    pub fn upsert_llm_model(&self, input: LlmModelInput<'_>) -> Result<(), PersistenceError> {
        let model_id = input.model_id.trim();
        let provider_id = input.provider_id.trim();
        let display_name = input.display_name.trim();
        let request_model = input.request_model.trim();
        if model_id.is_empty()
            || provider_id.is_empty()
            || display_name.is_empty()
            || request_model.is_empty()
            || input.context_tokens < 16_000
            || input.auto_compact_tokens < 1_000
            || input.auto_compact_tokens >= input.context_tokens
            || input.max_output_tokens == Some(0)
            || input.sort_order < 0
        {
            return Err(PersistenceError::Invalid("model has invalid fields".into()));
        }
        let context_tokens = i64::try_from(input.context_tokens)
            .map_err(|_| PersistenceError::Invalid("context_tokens exceeds SQLite".into()))?;
        let auto_compact_tokens = i64::try_from(input.auto_compact_tokens)
            .map_err(|_| PersistenceError::Invalid("auto_compact_tokens exceeds SQLite".into()))?;
        let max_output_tokens = input
            .max_output_tokens
            .map(|value| {
                i64::try_from(value).map_err(|_| {
                    PersistenceError::Invalid("max_output_tokens exceeds SQLite".into())
                })
            })
            .transpose()?;
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let timestamp = now();
        connection.execute(
            "INSERT INTO llm_model(model_id,provider_id,display_name,request_model,context_tokens,auto_compact_tokens,max_output_tokens,supports_streaming,supports_tool_use,supports_vision,supports_structured_output,supports_cancellation,supports_reasoning_effort,enabled,sort_order,created_at,updated_at)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)
             ON CONFLICT(model_id) DO UPDATE SET provider_id=excluded.provider_id,display_name=excluded.display_name,request_model=excluded.request_model,context_tokens=excluded.context_tokens,auto_compact_tokens=excluded.auto_compact_tokens,max_output_tokens=excluded.max_output_tokens,supports_streaming=excluded.supports_streaming,supports_tool_use=excluded.supports_tool_use,supports_vision=excluded.supports_vision,supports_structured_output=excluded.supports_structured_output,supports_cancellation=excluded.supports_cancellation,supports_reasoning_effort=excluded.supports_reasoning_effort,enabled=excluded.enabled,sort_order=excluded.sort_order,updated_at=excluded.updated_at",
            params![
                model_id,
                provider_id,
                display_name,
                request_model,
                context_tokens,
                auto_compact_tokens,
                max_output_tokens,
                input.supports_streaming,
                input.supports_tool_use,
                input.supports_vision,
                input.supports_structured_output,
                input.supports_cancellation,
                input.supports_reasoning_effort,
                input.enabled,
                input.sort_order,
                timestamp,
                timestamp,
            ],
        )?;
        Ok(())
    }

    pub fn llm_provider_api_key(
        &self,
        provider_id: &str,
    ) -> Result<Option<String>, PersistenceError> {
        let provider_id = provider_id.trim();
        if provider_id.is_empty() {
            return Err(PersistenceError::Invalid(
                "model provider id is required".into(),
            ));
        }
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        Ok(connection
            .query_row(
                "SELECT api_key FROM llm_model_provider WHERE provider_id=? AND enabled=1",
                [provider_id],
                |row| row.get(0),
            )
            .optional()?
            .flatten())
    }

    pub fn set_llm_provider_api_key(
        &self,
        provider_id: &str,
        value: &str,
    ) -> Result<(), PersistenceError> {
        let provider_id = provider_id.trim();
        let value = value.trim();
        if provider_id.is_empty() || value.is_empty() {
            return Err(PersistenceError::Invalid(
                "model provider id and credential are required".into(),
            ));
        }
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let changed = connection.execute(
            "UPDATE llm_model_provider SET api_key=?,updated_at=? WHERE provider_id=?",
            params![value, now(), provider_id],
        )?;
        if changed == 0 {
            return Err(PersistenceError::Invalid(
                "model provider does not exist".into(),
            ));
        }
        Ok(())
    }

    pub fn delete_llm_provider_api_key(&self, provider_id: &str) -> Result<(), PersistenceError> {
        let provider_id = provider_id.trim();
        if provider_id.is_empty() {
            return Err(PersistenceError::Invalid(
                "model provider id is required".into(),
            ));
        }
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let changed = connection.execute(
            "UPDATE llm_model_provider SET api_key=NULL,updated_at=? WHERE provider_id=?",
            params![now(), provider_id],
        )?;
        if changed == 0 {
            return Err(PersistenceError::Invalid(
                "model provider does not exist".into(),
            ));
        }
        Ok(())
    }

    pub fn set_setting(
        &self,
        scope: &str,
        scope_id: &str,
        key: &str,
        value: &Value,
    ) -> Result<(), PersistenceError> {
        if !matches!(scope, "global" | "project" | "session") || key.trim().is_empty() {
            return Err(PersistenceError::Invalid(
                "setting scope, scope id, and key are required".into(),
            ));
        }
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let key = key.trim();
        let value_json = serde_json::to_string(value)?;
        match scope {
            "global" => {
                connection.execute(
                    "INSERT INTO configuration(scope,key,value_json,updated_at) VALUES ('global',?,?,?) ON CONFLICT(key) WHERE scope='global' DO UPDATE SET value_json=excluded.value_json,updated_at=excluded.updated_at",
                    params![key, value_json, now()],
                )?;
            }
            "project" => {
                let project_id = scope_id.trim();
                if project_id.is_empty() {
                    return Err(PersistenceError::Invalid("project id is required".into()));
                }
                connection.execute(
                    "INSERT INTO configuration(scope,project_id,key,value_json,updated_at) VALUES ('project',?,?,?,?) ON CONFLICT(project_id,key) WHERE scope='project' DO UPDATE SET value_json=excluded.value_json,updated_at=excluded.updated_at",
                    params![project_id, key, value_json, now()],
                )?;
            }
            "session" => {
                let session_id = scope_id.trim();
                if session_id.is_empty() {
                    return Err(PersistenceError::Invalid("session id is required".into()));
                }
                connection.execute(
                    "INSERT INTO configuration(scope,session_id,key,value_json,updated_at) VALUES ('session',?,?,?,?) ON CONFLICT(session_id,key) WHERE scope='session' DO UPDATE SET value_json=excluded.value_json,updated_at=excluded.updated_at",
                    params![session_id, key, value_json, now()],
                )?;
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    pub fn recover_startup(&self) -> Result<Vec<SessionEvent>, PersistenceError> {
        let rows = {
            let connection = self
                .connection
                .lock()
                .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
            let mut statement = connection.prepare("SELECT turn_id,session_id,submission_idempotency_key,model_id FROM session_turn WHERE state NOT IN ('completed','failed','cancelled','interrupted') AND (recovery_status IS NULL OR recovery_status NOT IN ('pending','resuming'))")?;
            let values = statement
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            values
        };
        let mut events = Vec::new();
        for (turn_id, session_id, key, model) in rows {
            let event = self.append_content(
                &session_id,
                "turn.state",
                &json!({"turn_id":turn_id,"state":"interrupted","reason":"runtime_restarted","submission_idempotency_key":key,"model_id":model}),
            )?;
            if let Some(key) = key {
                self.fail_turn(
                    &session_id,
                    &key,
                    &json!({"code":"runtime_restarted","message":"Runtime restarted during the turn"}),
                )?;
            }
            events.push(event);
        }
        Ok(events)
    }

    pub fn resuming_turns(&self) -> Result<Vec<SuspendedTurn>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let mut statement = connection.prepare("SELECT recovery_approval_id,session_id,turn_id,recovery_snapshot_json,recovery_status FROM session_turn WHERE recovery_status='resuming'")?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?;
        rows.map(|row| {
            let value = row?;
            Ok(SuspendedTurn {
                approval_id: value.0,
                session_id: value.1,
                turn_id: value.2,
                snapshot: serde_json::from_str(&value.3)?,
                status: value.4,
            })
        })
        .collect()
    }

    pub fn project(
        &self,
        root: &str,
        display_name: &str,
    ) -> Result<ProjectRecord, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let timestamp = now();
        let existing: Option<String> = connection
            .query_row(
                "SELECT project_id FROM project WHERE canonical_root=?",
                [root],
                |row| row.get(0),
            )
            .optional()?;
        let id = existing.unwrap_or_else(|| Uuid::new_v4().to_string());
        connection.execute("INSERT INTO project (project_id,canonical_root,display_name,created_at,updated_at,last_opened_at,archived_at) VALUES (?,?,?,?,?,?,NULL) ON CONFLICT(project_id) DO UPDATE SET display_name=excluded.display_name,updated_at=excluded.updated_at,last_opened_at=excluded.last_opened_at,archived_at=NULL", params![id, root, display_name, timestamp, timestamp, timestamp])?;
        self.project_by_id_locked(&connection, &id)
    }

    pub fn projects(&self, include_archived: bool) -> Result<Vec<ProjectRecord>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let sql = if include_archived {
            "SELECT project_id,canonical_root,display_name,created_at,updated_at,last_opened_at,archived_at FROM project ORDER BY last_opened_at DESC"
        } else {
            "SELECT project_id,canonical_root,display_name,created_at,updated_at,last_opened_at,archived_at FROM project WHERE archived_at IS NULL ORDER BY last_opened_at DESC"
        };
        let mut statement = connection.prepare(sql)?;
        let rows = statement.query_map([], project_from_row)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn project_by_id(&self, id: &str) -> Result<Option<ProjectRecord>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        self.project_by_id_optional_locked(&connection, id)
    }

    pub fn add_project_dependency(
        &self,
        project_id: &str,
        canonical_root: &str,
        display_name: &str,
    ) -> Result<ProjectDependencyRecord, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let project = self
            .project_by_id_optional_locked(&connection, project_id)?
            .ok_or_else(|| PersistenceError::Invalid("project not found".into()))?;
        if project.canonical_root == canonical_root {
            return Err(PersistenceError::Invalid(
                "project cannot depend on its own root".into(),
            ));
        }
        let existing: Option<String> = connection
            .query_row(
                "SELECT dependency_id FROM project_dependency WHERE project_id=? AND canonical_root=?",
                params![project_id, canonical_root],
                |row| row.get(0),
            )
            .optional()?;
        let dependency_id = existing.unwrap_or_else(|| Uuid::new_v4().to_string());
        connection.execute(
            "INSERT INTO project_dependency(dependency_id,project_id,canonical_root,display_name,created_at) VALUES (?,?,?,?,?) ON CONFLICT(project_id,canonical_root) DO UPDATE SET display_name=excluded.display_name",
            params![dependency_id, project_id, canonical_root, display_name, now()],
        )?;
        self.project_dependency_by_id_locked(&connection, project_id, &dependency_id)?
            .ok_or_else(|| PersistenceError::Invalid("dependency was not stored".into()))
    }

    pub fn project_dependencies(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectDependencyRecord>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let mut statement = connection.prepare(
            "SELECT dependency_id,project_id,canonical_root,display_name,created_at FROM project_dependency WHERE project_id=? ORDER BY display_name COLLATE NOCASE,dependency_id",
        )?;
        let rows = statement.query_map([project_id], project_dependency_from_row)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn project_dependency_by_id(
        &self,
        project_id: &str,
        dependency_id: &str,
    ) -> Result<Option<ProjectDependencyRecord>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        self.project_dependency_by_id_locked(&connection, project_id, dependency_id)
    }

    pub fn remove_project_dependency(
        &self,
        project_id: &str,
        dependency_id: &str,
    ) -> Result<bool, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        Ok(connection.execute(
            "DELETE FROM project_dependency WHERE project_id=? AND dependency_id=?",
            params![project_id, dependency_id],
        )? == 1)
    }

    pub fn create_session(
        &self,
        project_id: &str,
        title: Option<&str>,
        model_id: Option<&str>,
    ) -> Result<SessionRecord, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let exists: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM project WHERE project_id=? AND archived_at IS NULL)",
            [project_id],
            |row| row.get(0),
        )?;
        if !exists {
            return Err(PersistenceError::Invalid(
                "project not found or archived".into(),
            ));
        }
        let id = Uuid::new_v4().to_string();
        let timestamp = now();
        connection.execute("INSERT INTO session (session_id,project_id,title,model_id,status,created_at,updated_at,last_activity_at,pin_at,archived_at) VALUES (?,?,?,?,?,?,?,?,NULL,NULL)", params![id, project_id, title, model_id, "active", timestamp, timestamp, timestamp])?;
        self.session_by_id_locked(&connection, &id)
    }

    pub fn session_by_id(&self, id: &str) -> Result<Option<SessionRecord>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        self.session_by_id_optional_locked(&connection, id)
    }

    pub fn sessions_for_project(
        &self,
        project_id: &str,
        include_archived: bool,
    ) -> Result<Vec<SessionRecord>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let sql = if include_archived {
            "SELECT session_id,project_id,title,model_id,status,created_at,updated_at,last_activity_at,pin_at,archived_at FROM session WHERE project_id=? ORDER BY (pin_at IS NOT NULL) DESC,pin_at DESC,last_activity_at DESC,session_id"
        } else {
            "SELECT session_id,project_id,title,model_id,status,created_at,updated_at,last_activity_at,pin_at,archived_at FROM session WHERE project_id=? AND status='active' ORDER BY (pin_at IS NOT NULL) DESC,pin_at DESC,last_activity_at DESC,session_id"
        };
        let mut statement = connection.prepare(sql)?;
        let rows = statement.query_map([project_id], session_from_row)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn set_session_archived(
        &self,
        id: &str,
        archived: bool,
    ) -> Result<SessionRecord, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let timestamp = now();
        connection.execute(
            "UPDATE session SET status=?,pin_at=NULL,archived_at=?,updated_at=? WHERE session_id=?",
            params![
                if archived { "archived" } else { "active" },
                if archived {
                    Some(timestamp.clone())
                } else {
                    None
                },
                timestamp,
                id
            ],
        )?;
        self.session_by_id_locked(&connection, id)
    }

    pub fn set_session_pinned(
        &self,
        id: &str,
        pinned: bool,
    ) -> Result<SessionRecord, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let status: String = connection
            .query_row(
                "SELECT status FROM session WHERE session_id=?",
                [id],
                |row| row.get(0),
            )
            .optional()?
            .ok_or_else(|| PersistenceError::Invalid("session not found".into()))?;
        if status != "active" && pinned {
            return Err(PersistenceError::Invalid(
                "archived sessions cannot be pinned".into(),
            ));
        }
        connection.execute(
            "UPDATE session SET pin_at=? WHERE session_id=?",
            params![if pinned { Some(now()) } else { None }, id],
        )?;
        self.session_by_id_locked(&connection, id)
    }

    pub fn rename_session(&self, id: &str, title: &str) -> Result<SessionRecord, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        connection.execute(
            "UPDATE session SET title=?,updated_at=? WHERE session_id=?",
            params![title, now(), id],
        )?;
        self.session_by_id_locked(&connection, id)
    }

    fn project_by_id_locked(
        &self,
        connection: &Connection,
        id: &str,
    ) -> Result<ProjectRecord, PersistenceError> {
        self.project_by_id_optional_locked(connection, id)?
            .ok_or_else(|| PersistenceError::Invalid("project not found".into()))
    }
    fn project_by_id_optional_locked(
        &self,
        connection: &Connection,
        id: &str,
    ) -> Result<Option<ProjectRecord>, PersistenceError> {
        connection.query_row("SELECT project_id,canonical_root,display_name,created_at,updated_at,last_opened_at,archived_at FROM project WHERE project_id=?", [id], project_from_row).optional().map_err(Into::into)
    }
    fn project_dependency_by_id_locked(
        &self,
        connection: &Connection,
        project_id: &str,
        dependency_id: &str,
    ) -> Result<Option<ProjectDependencyRecord>, PersistenceError> {
        connection.query_row(
            "SELECT dependency_id,project_id,canonical_root,display_name,created_at FROM project_dependency WHERE project_id=? AND dependency_id=?",
            params![project_id, dependency_id],
            project_dependency_from_row,
        ).optional().map_err(Into::into)
    }
    fn session_by_id_locked(
        &self,
        connection: &Connection,
        id: &str,
    ) -> Result<SessionRecord, PersistenceError> {
        self.session_by_id_optional_locked(connection, id)?
            .ok_or_else(|| PersistenceError::Invalid("session not found".into()))
    }
    fn session_by_id_optional_locked(
        &self,
        connection: &Connection,
        id: &str,
    ) -> Result<Option<SessionRecord>, PersistenceError> {
        connection.query_row("SELECT session_id,project_id,title,model_id,status,created_at,updated_at,last_activity_at,pin_at,archived_at FROM session WHERE session_id=?", [id], session_from_row).optional().map_err(Into::into)
    }
}

fn apply_projection(
    transaction: &Transaction<'_>,
    session_id: &str,
    occurred_at: &str,
    event_type: &str,
    payload: &Value,
) -> Result<(), PersistenceError> {
    if matches!(
        event_type,
        "message.user" | "message.assistant" | "message.thinking"
    ) {
        if let Some(message) = payload.get("message") {
            let role = message.get("role").and_then(Value::as_str);
            if matches!(role, Some("user" | "assistant" | "thinking")) {
                let message_id = payload
                    .get("message_id")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("event:{session_id}:{}", Uuid::new_v4()));
                transaction.execute("INSERT OR IGNORE INTO session_message (message_id,session_id,turn_id,session_call_id,role,message_json,created_at) VALUES (?,?,?,?,?,?,?)", params![message_id, session_id, payload.get("turn_id").and_then(Value::as_str), payload.get("call_id").or_else(|| payload.get("exchange_id")).and_then(Value::as_str), role, serde_json::to_string(message)?, occurred_at])?;
            }
        }
    }
    if event_type == "turn.state" {
        if let (Some(turn_id), Some(state)) = (
            payload.get("turn_id").and_then(Value::as_str),
            payload.get("state").and_then(Value::as_str),
        ) {
            let terminal = matches!(state, "completed" | "failed" | "cancelled" | "interrupted");
            transaction.execute("INSERT INTO session_turn(turn_id,session_id,submission_idempotency_key,state,model_id,created_at,updated_at,completed_at,error_code) VALUES (?,?,?,?,?,?,?,?,?) ON CONFLICT(turn_id) DO UPDATE SET state=excluded.state,updated_at=excluded.updated_at,completed_at=excluded.completed_at,error_code=excluded.error_code", params![turn_id, session_id, payload.get("submission_idempotency_key").and_then(Value::as_str), state, payload.get("model_id").and_then(Value::as_str), occurred_at, occurred_at, if terminal {Some(occurred_at)} else {None}, payload.get("reason").and_then(Value::as_str)])?;
        }
    }
    if event_type == "usage.updated" {
        let turn_id = payload
            .get("turn_id")
            .and_then(Value::as_str)
            .ok_or_else(|| PersistenceError::Invalid("usage event is missing turn_id".into()))?;
        let usage = payload
            .get("usage")
            .ok_or_else(|| PersistenceError::Invalid("usage event is missing usage".into()))?;
        let input_tokens = usage_to_sql(usage, "input_tokens")?;
        let output_tokens = usage_to_sql(usage, "output_tokens")?;
        let total_tokens = usage_to_sql(usage, "total_tokens")?;
        let changed = transaction.execute(
            "UPDATE session_turn
             SET input_tokens=?,output_tokens=?,total_tokens=?,updated_at=?
             WHERE turn_id=? AND session_id=?",
            params![
                input_tokens,
                output_tokens,
                total_tokens,
                occurred_at,
                turn_id,
                session_id
            ],
        )?;
        if changed == 0 {
            return Err(PersistenceError::Invalid(
                "usage event references an unknown turn".into(),
            ));
        }
    }
    if event_type == "provider.exchange.started" {
        let exchange_id = payload
            .get("exchange_id")
            .and_then(Value::as_str)
            .ok_or_else(|| PersistenceError::Invalid("provider exchange is missing id".into()))?;
        let turn_id = payload
            .get("turn_id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                PersistenceError::Invalid("provider exchange is missing turn_id".into())
            })?;
        let provider = payload
            .get("provider")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                PersistenceError::Invalid("provider exchange is missing provider".into())
            })?;
        let model_id = payload
            .get("model_id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                PersistenceError::Invalid("provider exchange is missing model_id".into())
            })?;
        let wire_model = payload
            .get("wire_model")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                PersistenceError::Invalid("provider exchange is missing wire_model".into())
            })?;
        let iteration = payload
            .get("iteration")
            .and_then(Value::as_i64)
            .unwrap_or(1);
        let input_messages = payload
            .get("input_messages")
            .cloned()
            .unwrap_or_else(|| json!([]));
        transaction.execute(
            "INSERT INTO session_call(call_id,session_id,turn_id,provider,model_id,wire_model,state,iteration,started_at,input_messages_json,tool_calls_json)
             VALUES (?,?,?,?,?,?, 'started',?,?,?,?)
             ON CONFLICT(call_id) DO UPDATE SET state='started',input_messages_json=excluded.input_messages_json",
            params![
                exchange_id,
                session_id,
                turn_id,
                provider,
                model_id,
                wire_model,
                iteration,
                occurred_at,
                serde_json::to_string(&input_messages)?,
                "[]",
            ],
        )?;
    }
    if event_type == "provider.exchange.completed" {
        let exchange_id = payload
            .get("exchange_id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                PersistenceError::Invalid("provider exchange completion is missing id".into())
            })?;
        let output_message = payload
            .get("output_message")
            .map(serde_json::to_string)
            .transpose()?;
        let tool_calls = payload
            .get("tool_calls")
            .cloned()
            .unwrap_or_else(|| json!([]));
        let usage = payload
            .get("usage")
            .map(serde_json::to_string)
            .transpose()?;
        transaction.execute(
            "UPDATE session_call
             SET state='completed',completed_at=?,output_message_json=?,tool_calls_json=?,usage_json=?,provider_request_id=?,provider_response_id=?,finish_reason=?,error_json=NULL
             WHERE session_id=? AND call_id=?",
            params![
                occurred_at,
                output_message,
                serde_json::to_string(&tool_calls)?,
                usage,
                payload.get("provider_request_id").and_then(Value::as_str),
                payload.get("provider_response_id").and_then(Value::as_str),
                payload.get("finish_reason").and_then(Value::as_str),
                session_id,
                exchange_id,
            ],
        )?;
    }
    if event_type == "provider.exchange.failed" {
        let exchange_id = payload
            .get("exchange_id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                PersistenceError::Invalid("provider exchange failure is missing id".into())
            })?;
        transaction.execute(
            "UPDATE session_call
             SET state='failed',completed_at=?,provider_request_id=?,finish_reason='error',error_json=?
             WHERE session_id=? AND call_id=?",
            params![
                occurred_at,
                payload.get("provider_request_id").and_then(Value::as_str),
                payload
                    .get("error")
                    .map(serde_json::to_string)
                    .transpose()?,
                session_id,
                exchange_id,
            ],
        )?;
    }
    if event_type == "todo.updated" {
        let turn_id = payload
            .get("turn_id")
            .and_then(Value::as_str)
            .ok_or_else(|| PersistenceError::Invalid("todo event is missing turn_id".into()))?;
        let todos = payload
            .get("todos")
            .and_then(Value::as_array)
            .ok_or_else(|| PersistenceError::Invalid("todo event is missing todos".into()))?;
        if todos.len() > 100 {
            return Err(PersistenceError::Invalid(
                "todo event contains more than 100 items".into(),
            ));
        }
        let mut in_progress = 0;
        transaction.execute("DELETE FROM session_turn_todo WHERE turn_id=?", [turn_id])?;
        for (ordinal, todo) in todos.iter().enumerate() {
            let object = todo.as_object().ok_or_else(|| {
                PersistenceError::Invalid("todo event contains a non-object item".into())
            })?;
            let content = object
                .get("content")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty() && value.chars().count() <= 500)
                .ok_or_else(|| PersistenceError::Invalid("todo content is required".into()))?;
            let status = object
                .get("status")
                .and_then(Value::as_str)
                .filter(|value| {
                    matches!(
                        *value,
                        "pending" | "in_progress" | "completed" | "cancelled"
                    )
                })
                .ok_or_else(|| PersistenceError::Invalid("todo status is invalid".into()))?;
            if status == "in_progress" {
                in_progress += 1;
                if in_progress > 1 {
                    return Err(PersistenceError::Invalid(
                        "todo event contains multiple in_progress items".into(),
                    ));
                }
            }
            let priority = object
                .get("priority")
                .and_then(Value::as_str)
                .filter(|value| matches!(*value, "high" | "medium" | "low"))
                .ok_or_else(|| PersistenceError::Invalid("todo priority is invalid".into()))?;
            transaction.execute(
                "INSERT INTO session_turn_todo(turn_id,ordinal,content,status,priority,created_at,updated_at,completed_at)
                 VALUES (?,?,?,?,?,?,?,?)",
                params![
                    turn_id,
                    ordinal as i64,
                    content,
                    status,
                    priority,
                    occurred_at,
                    occurred_at,
                    if matches!(status, "completed" | "cancelled") {
                        Some(occurred_at)
                    } else {
                        None
                    },
                ],
            )?;
        }
    }
    if event_type == "tool.requested" {
        if let (Some(turn_id), Some(call_id), Some(name), Some(arguments)) = (
            payload.get("turn_id").and_then(Value::as_str),
            payload.get("tool_call_id").and_then(Value::as_str),
            payload.get("name").and_then(Value::as_str),
            payload.get("arguments"),
        ) {
            transaction.execute(
                "INSERT INTO session_tool_use(turn_id,tool_call_id,session_call_id,name,request_json,state,ordinal,created_at,updated_at) VALUES (?,?,?,?,?,'requested',?,?,?) ON CONFLICT(turn_id,tool_call_id) DO UPDATE SET session_call_id=COALESCE(excluded.session_call_id,session_tool_use.session_call_id),request_json=excluded.request_json,name=excluded.name,ordinal=COALESCE(excluded.ordinal,session_tool_use.ordinal),updated_at=excluded.updated_at",
                params![turn_id, call_id, payload.get("call_id").and_then(Value::as_str), name, serde_json::to_string(arguments)?, payload.get("ordinal").and_then(Value::as_i64), occurred_at, occurred_at],
            )?;
        }
    }
    if event_type == "tool.state" {
        if let (Some(turn_id), Some(call_id), Some(name), Some(state)) = (
            payload.get("turn_id").and_then(Value::as_str),
            payload.get("tool_call_id").and_then(Value::as_str),
            payload.get("name").and_then(Value::as_str),
            payload.get("state").and_then(Value::as_str),
        ) {
            let terminal = matches!(state, "denied" | "succeeded" | "failed" | "timed_out");
            transaction.execute("INSERT INTO session_tool_use(turn_id,tool_call_id,session_call_id,name,request_json,state,ordinal,created_at,updated_at,completed_at,error_code) VALUES (?,?,?,?,?,?,?,?,?,?,?) ON CONFLICT(turn_id,tool_call_id) DO UPDATE SET session_call_id=COALESCE(excluded.session_call_id,session_tool_use.session_call_id),name=excluded.name,state=excluded.state,ordinal=COALESCE(excluded.ordinal,session_tool_use.ordinal),updated_at=excluded.updated_at,completed_at=excluded.completed_at,error_code=excluded.error_code", params![turn_id,call_id,payload.get("call_id").and_then(Value::as_str),name,Option::<String>::None,state,payload.get("ordinal").and_then(Value::as_i64),occurred_at,occurred_at,if terminal {Some(occurred_at)} else {None},payload.get("reason").and_then(Value::as_str)])?;
        }
    }
    if event_type == "tool.result" {
        if let (Some(turn_id), Some(call_id), Some(result)) = (
            payload.get("turn_id").and_then(Value::as_str),
            payload.get("tool_call_id").and_then(Value::as_str),
            payload.get("result"),
        ) {
            transaction.execute(
                "UPDATE session_tool_use SET result_json=?,updated_at=? WHERE turn_id=? AND tool_call_id=?",
                params![serde_json::to_string(result)?, occurred_at, turn_id, call_id],
            )?;
        }
    }
    if event_type == "checkpoint.captured" {
        if let Some(checkpoint_id) = payload.get("checkpoint_id").and_then(Value::as_str) {
            transaction.execute("INSERT OR IGNORE INTO checkpoint(checkpoint_id,manifest_id,session_id,turn_id,tool_call_id,relative_path,status,created_at,ordinal) VALUES (?,?,?,?,?,?,'available',?,?)", params![checkpoint_id,payload.get("manifest_id").and_then(Value::as_str),session_id,payload.get("turn_id").and_then(Value::as_str),payload.get("tool_call_id").and_then(Value::as_str),payload.get("path").and_then(Value::as_str),occurred_at,payload.get("ordinal").and_then(Value::as_i64)])?;
        }
    }
    if event_type == "checkpoint.item_restored" {
        if let Some(checkpoint_id) = payload.get("checkpoint_id").and_then(Value::as_str) {
            transaction.execute("UPDATE checkpoint SET status='restored',restored_at=? WHERE checkpoint_id=? AND session_id=?", params![occurred_at,checkpoint_id,session_id])?;
        }
    }
    Ok(())
}

fn approval_by_id(
    connection: &Connection,
    id: &str,
) -> Result<Option<ApprovalRecord>, PersistenceError> {
    let row: Option<ApprovalRow> = connection.query_row("SELECT approval_id,project_id,session_id,turn_id,tool_call_id,operation,arguments_json,status,decision,decision_source,created_at,updated_at FROM approval_request WHERE approval_id=?", [id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?,row.get(6)?,row.get(7)?,row.get(8)?,row.get(9)?,row.get(10)?,row.get(11)?))).optional()?;
    row.map(|value| {
        Ok(ApprovalRecord {
            approval_id: value.0,
            project_id: value.1,
            session_id: value.2,
            turn_id: value.3,
            tool_call_id: value.4,
            operation: value.5,
            arguments: serde_json::from_str(&value.6)?,
            status: value.7,
            decision: value.8,
            decision_source: value.9,
            created_at: value.10,
            updated_at: value.11,
        })
    })
    .transpose()
}
fn manifest_by_turn(
    connection: &Connection,
    turn_id: &str,
) -> Result<Option<CheckpointManifest>, PersistenceError> {
    connection.query_row("SELECT manifest_id,session_id,turn_id,status,created_at,updated_at,expires_at,restored_at FROM checkpoint_manifest WHERE turn_id=?", [turn_id], manifest_from_row).optional().map_err(Into::into)
}
fn manifest_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<CheckpointManifest> {
    Ok(CheckpointManifest {
        manifest_id: row.get(0)?,
        session_id: row.get(1)?,
        turn_id: row.get(2)?,
        status: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
        expires_at: row.get(6)?,
        restored_at: row.get(7)?,
    })
}
fn checkpoint_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<CheckpointItem> {
    Ok(CheckpointItem {
        checkpoint_id: row.get(0)?,
        manifest_id: row.get(1)?,
        session_id: row.get(2)?,
        turn_id: row.get(3)?,
        tool_call_id: row.get(4)?,
        relative_path: row.get(5)?,
        status: row.get(6)?,
        created_at: row.get(7)?,
        restored_at: row.get(8)?,
        invalidated_at: row.get(9)?,
        ordinal: row.get(10)?,
    })
}

fn project_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectRecord> {
    Ok(ProjectRecord {
        project_id: row.get(0)?,
        canonical_root: row.get(1)?,
        display_name: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
        last_opened_at: row.get(5)?,
        archived_at: row.get(6)?,
    })
}

fn project_dependency_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<ProjectDependencyRecord> {
    Ok(ProjectDependencyRecord {
        dependency_id: row.get(0)?,
        project_id: row.get(1)?,
        canonical_root: row.get(2)?,
        display_name: row.get(3)?,
        created_at: row.get(4)?,
    })
}
fn session_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SessionRecord> {
    Ok(SessionRecord {
        session_id: row.get(0)?,
        project_id: row.get(1)?,
        title: row.get(2)?,
        model_id: row.get(3)?,
        status: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
        last_activity_at: row.get(7)?,
        pin_at: row.get(8)?,
        archived_at: row.get(9)?,
    })
}
fn provider_exchange_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProviderExchange> {
    let input_messages: String = row.get(12)?;
    let output_message: Option<String> = row.get(13)?;
    let tool_calls: String = row.get(14)?;
    let usage: Option<String> = row.get(15)?;
    let error: Option<String> = row.get(17)?;
    Ok(ProviderExchange {
        exchange_id: row.get(0)?,
        session_id: row.get(1)?,
        turn_id: row.get(2)?,
        provider: row.get(3)?,
        model_id: row.get(4)?,
        wire_model: row.get(5)?,
        provider_request_id: row.get(6)?,
        provider_response_id: row.get(7)?,
        state: row.get(8)?,
        iteration: row.get(9)?,
        started_at: row.get(10)?,
        completed_at: row.get(11)?,
        input_messages: serde_json::from_str(&input_messages).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                12,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        output_message: output_message
            .map(|value| {
                serde_json::from_str(&value).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        13,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })
            })
            .transpose()?,
        tool_calls: serde_json::from_str(&tool_calls).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                14,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        usage: usage
            .map(|value| {
                serde_json::from_str(&value).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        15,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })
            })
            .transpose()?,
        finish_reason: row.get(16)?,
        error: error
            .map(|value| {
                serde_json::from_str(&value).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        17,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })
            })
            .transpose()?,
    })
}
fn now() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn initialize(connection: &mut Connection) -> Result<(), PersistenceError> {
    let transaction = connection.transaction()?;
    schema::apply(&transaction)?;
    let actual_tables = schema::table_names(&transaction)?;
    if actual_tables.iter().map(String::as_str).collect::<Vec<_>>() != schema::TABLE_NAMES {
        return Err(PersistenceError::Invalid(format!(
            "database tables do not match the current schema: {actual_tables:?}"
        )));
    }
    if !schema::session_message_excludes_tool_role(&transaction)? {
        return Err(PersistenceError::Invalid(
            "session_message schema still permits the retired tool role".into(),
        ));
    }
    if !schema::session_message_excludes_usage_column(&transaction)? {
        return Err(PersistenceError::Invalid(
            "session_message schema still contains the retired usage_json column".into(),
        ));
    }
    if !schema::session_call_includes_provider_ids(&transaction)? {
        return Err(PersistenceError::Invalid(
            "session_call schema is missing provider request/response identifiers".into(),
        ));
    }
    data::apply(&transaction)?;
    transaction.commit()?;
    Ok(())
}

fn usage_to_sql(usage: &Value, field: &str) -> Result<i64, PersistenceError> {
    let value = usage
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| PersistenceError::Invalid(format!("usage event has invalid {field}")))?;
    i64::try_from(value)
        .map_err(|_| PersistenceError::Invalid(format!("usage event {field} exceeds SQLite")))
}

fn usage_from_sql(value: i64) -> Result<u64, PersistenceError> {
    u64::try_from(value)
        .map_err(|_| PersistenceError::Invalid("stored token usage is negative".into()))
}

fn session_trace_turn_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SessionTraceTurn> {
    Ok(SessionTraceTurn {
        turn_id: row.get(0)?,
        session_id: row.get(1)?,
        state: row.get(2)?,
        model_id: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
        started_at: row.get(6)?,
        completed_at: row.get(7)?,
        error_code: row.get(8)?,
        input_tokens: nonnegative_u64_from_row(row, 9)?,
        output_tokens: nonnegative_u64_from_row(row, 10)?,
        total_tokens: nonnegative_u64_from_row(row, 11)?,
    })
}

fn nonnegative_u64_from_row(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<u64> {
    let value: i64 = row.get(index)?;
    u64::try_from(value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            index,
            rusqlite::types::Type::Integer,
            Box::new(error),
        )
    })
}

fn llm_model_provider_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<LlmModelProviderRecord> {
    Ok(LlmModelProviderRecord {
        provider_id: row.get(0)?,
        display_name: row.get(1)?,
        endpoint: row.get(2)?,
        adapter_type: row.get(3)?,
        api_key_configured: row.get(4)?,
        enabled: row.get(5)?,
        sort_order: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn llm_model_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<LlmModelRecord> {
    let context_tokens: i64 = row.get(4)?;
    let auto_compact_tokens: i64 = row.get(5)?;
    let max_output_tokens: Option<i64> = row.get(6)?;
    Ok(LlmModelRecord {
        model_id: row.get(0)?,
        provider_id: row.get(1)?,
        display_name: row.get(2)?,
        request_model: row.get(3)?,
        context_tokens: u64::try_from(context_tokens).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                4,
                rusqlite::types::Type::Integer,
                Box::new(error),
            )
        })?,
        auto_compact_tokens: u64::try_from(auto_compact_tokens).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                5,
                rusqlite::types::Type::Integer,
                Box::new(error),
            )
        })?,
        max_output_tokens: max_output_tokens
            .map(|value| {
                u64::try_from(value).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Integer,
                        Box::new(error),
                    )
                })
            })
            .transpose()?,
        supports_streaming: row.get(7)?,
        supports_tool_use: row.get(8)?,
        supports_vision: row.get(9)?,
        supports_structured_output: row.get(10)?,
        supports_cancellation: row.get(11)?,
        supports_reasoning_effort: row.get(12)?,
        enabled: row.get(13)?,
        sort_order: row.get(14)?,
        created_at: row.get(15)?,
        updated_at: row.get(16)?,
    })
}

fn repair_incomplete_tool_exchanges(messages: Vec<Message>) -> Vec<Message> {
    let mut repaired = Vec::with_capacity(messages.len());
    let mut index = 0;
    while index < messages.len() {
        let message = &messages[index];
        if message.role == "assistant" && !message.tool_calls.is_empty() {
            let expected = message
                .tool_calls
                .iter()
                .map(|call| call.call_id.as_str())
                .collect::<Vec<_>>();
            let end = index + 1 + expected.len();
            if messages.len() < end {
                break;
            }
            let has_all_tool_results =
                messages[index + 1..end]
                    .iter()
                    .zip(expected.iter())
                    .all(|(tool, call_id)| {
                        tool.role == "tool" && tool.tool_call_id.as_deref() == Some(*call_id)
                    });
            if !has_all_tool_results {
                break;
            }
            repaired.push(message.clone());
            repaired.extend_from_slice(&messages[index + 1..end]);
            index = end;
            continue;
        }
        if message.role == "tool" {
            break;
        }
        repaired.push(message.clone());
        index += 1;
    }
    repaired
}

#[cfg(test)]
mod tests {
    use super::*;

    fn object_names(connection: &Connection, object_type: &str) -> Vec<String> {
        let mut statement = connection
            .prepare(
                "SELECT name FROM sqlite_schema
                 WHERE type=? AND name NOT LIKE 'sqlite_%' AND sql IS NOT NULL
                 ORDER BY name",
            )
            .unwrap();
        statement
            .query_map([object_type], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    fn session(store: &Store) -> String {
        let project = store.project("/tmp/suncode-test", "Test").unwrap();
        store
            .create_session(&project.project_id, None, Some("deepseek-v4-flash"))
            .unwrap()
            .session_id
    }

    #[test]
    fn session_pinning_is_project_local_and_archiving_clears_it() {
        let store = Store::open_memory().unwrap();
        let project = store.project("/tmp/suncode-pinning", "Pinning").unwrap();
        let first = store
            .create_session(
                &project.project_id,
                Some("First"),
                Some("deepseek-v4-flash"),
            )
            .unwrap();
        let second = store
            .create_session(
                &project.project_id,
                Some("Second"),
                Some("deepseek-v4-flash"),
            )
            .unwrap();

        assert!(first.pin_at.is_none());
        store.set_session_pinned(&second.session_id, true).unwrap();
        let sessions = store
            .sessions_for_project(&project.project_id, true)
            .unwrap();
        assert_eq!(sessions[0].session_id, second.session_id);
        assert!(sessions[0].pin_at.is_some());
        assert!(sessions[1].pin_at.is_none());

        store
            .set_session_archived(&second.session_id, true)
            .unwrap();
        let archived = store.session_by_id(&second.session_id).unwrap().unwrap();
        assert!(archived.pin_at.is_none());
        assert!(store.set_session_pinned(&second.session_id, true).is_err());
    }

    #[test]
    fn context_messages_keep_completed_tool_exchange() {
        let store = Store::open_memory().unwrap();
        let session_id = session(&store);
        let assistant = Message {
            role: "assistant".into(),
            content: Vec::new(),
            tool_calls: vec![ToolCall {
                call_id: "call-1".into(),
                name: "read".into(),
                arguments: json!({"path":"README.md"}),
            }],
            tool_call_id: None,
        };
        store
            .append_content(
                &session_id,
                "turn.state",
                &json!({"turn_id":"turn-1","state":"resolving_calls"}),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "message.user",
                &json!({"message_id":"user-1","turn_id":"turn-1","message":Message::text("user","read")}),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "message.assistant",
                &json!({"message_id":"assistant-1","turn_id":"turn-1","message":assistant}),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "tool.requested",
                &json!({"turn_id":"turn-1","tool_call_id":"call-1","name":"read","arguments":{"path":"README.md"},"ordinal":0}),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "tool.state",
                &json!({"turn_id":"turn-1","tool_call_id":"call-1","name":"read","state":"succeeded","ordinal":0}),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "tool.result",
                &json!({"turn_id":"turn-1","tool_call_id":"call-1","result":{"content":"hello"}}),
            )
            .unwrap();

        let messages = store.context_messages(&session_id).unwrap();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[1].tool_calls[0].call_id, "call-1");
        assert_eq!(messages[2].role, "tool");
        assert_eq!(messages[2].tool_call_id.as_deref(), Some("call-1"));
        assert_eq!(messages[2].text_content(), "{\"content\":\"hello\"}");
    }

    #[test]
    fn question_recovery_snapshot_is_pending_and_resumable() {
        let store = Store::open_memory().unwrap();
        let project = store.project("/tmp/suncode-question", "Question").unwrap();
        let session = store
            .create_session(&project.project_id, None, Some("deepseek-v4-flash"))
            .unwrap();
        let admission = store
            .begin_turn(
                &session.session_id,
                "question-key",
                "clarify",
                "deepseek-v4-flash",
            )
            .unwrap();
        store
            .append_content(
                &session.session_id,
                "tool.requested",
                &json!({"turn_id":admission.turn_id,"tool_call_id":"question-call","name":"question","arguments":{"questions":[]}}),
            )
            .unwrap();
        store
            .append_content(
                &session.session_id,
                "tool.state",
                &json!({"turn_id":admission.turn_id,"tool_call_id":"question-call","name":"question","state":"awaiting_question","reason":"user_input_required"}),
            )
            .unwrap();
        let snapshot = json!({
            "session_id": session.session_id,
            "turn_id": admission.turn_id,
            "pending_call": {"call_id":"question-call","name":"question","arguments":{"questions":[{"question":"Choose","header":"Mode","options":[{"label":"Fast","description":"Quick"}]}]}},
        });
        store
            .create_question("que_test", &admission.turn_id, &snapshot)
            .unwrap();
        let pending = store
            .pending_question(&session.session_id)
            .unwrap()
            .unwrap();
        assert_eq!(pending["request_id"], "que_test");
        let resumed = store
            .resolve_question("que_test", &[vec!["Fast".into()]], false)
            .unwrap()
            .unwrap();
        assert_eq!(resumed.status, "resuming");
        assert_eq!(resumed.snapshot["question_answers"][0][0], "Fast");
        assert!(store
            .pending_question(&session.session_id)
            .unwrap()
            .is_none());
    }

    #[test]
    fn conversation_turns_group_messages_and_tools() {
        let store = Store::open_memory().unwrap();
        let session_id = session(&store);
        store
            .append_content(
                &session_id,
                "turn.state",
                &json!({"turn_id":"turn-1","state":"resolving_calls"}),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "message.user",
                &json!({"message_id":"user-1","turn_id":"turn-1","message":Message::text("user","inspect")}),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "message.assistant",
                &json!({"message_id":"assistant-1","turn_id":"turn-1","message":Message::text("assistant","reading")}),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "tool.requested",
                &json!({"turn_id":"turn-1","tool_call_id":"tool-1","name":"read","arguments":{"path":"README.md"},"ordinal":0}),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "tool.state",
                &json!({"turn_id":"turn-1","tool_call_id":"tool-1","name":"read","state":"succeeded"}),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "tool.result",
                &json!({"turn_id":"turn-1","tool_call_id":"tool-1","result":{"content":"hello"}}),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "todo.updated",
                &json!({
                    "turn_id":"turn-1",
                    "todos":[
                        {"content":"Implement tool","status":"completed","priority":"high"},
                        {"content":"Run tests","status":"in_progress","priority":"medium"}
                    ]
                }),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "message.assistant",
                &json!({"message_id":"assistant-2","turn_id":"turn-1","message":Message::text("assistant","done")}),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "turn.state",
                &json!({"turn_id":"turn-1","state":"completed"}),
            )
            .unwrap();

        let turns = store.session_conversation_turns(&session_id).unwrap();
        let turn = turns.first().unwrap();
        assert_eq!(turn.turn_id, "turn-1");
        assert_eq!(turn.state, "completed");
        assert_eq!(turn.messages.len(), 3);
        assert_eq!(turn.messages[0].message_id, "user-1");
        assert_eq!(turn.messages[2].message_id, "assistant-2");
        assert_eq!(turn.tool_uses.len(), 1);
        assert_eq!(turn.tool_uses[0].tool_call_id, "tool-1");
        assert_eq!(turn.tool_uses[0].state, "succeeded");
        assert_eq!(
            turn.tool_uses[0].result.as_ref().unwrap()["content"],
            "hello"
        );
        assert_eq!(turn.todos.len(), 2);
        assert_eq!(turn.todos[0].content, "Implement tool");
        assert_eq!(turn.todos[0].status, "completed");
        assert_eq!(turn.todos[1].status, "in_progress");

        store
            .append_content(
                &session_id,
                "todo.updated",
                &json!({
                    "turn_id":"turn-1",
                    "todos":[
                        {"content":"Run tests","status":"completed","priority":"medium"}
                    ]
                }),
            )
            .unwrap();
        let updated_turn = store
            .session_conversation_turns(&session_id)
            .unwrap()
            .remove(0);
        assert_eq!(updated_turn.todos.len(), 1);
        assert_eq!(updated_turn.todos[0].content, "Run tests");
        assert_eq!(updated_turn.todos[0].status, "completed");
    }

    #[test]
    fn thinking_messages_are_persisted_and_read_in_timestamp_order() {
        let store = Store::open_memory().unwrap();
        let session_id = session(&store);

        store
            .append_content(
                &session_id,
                "message.thinking",
                &json!({
                    "message_id": "thinking-1",
                    "turn_id": "turn-1",
                    "message": Message::text("thinking", "inspect the project")
                }),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "message.user",
                &json!({
                    "message_id": "user-1",
                    "turn_id": "turn-1",
                    "message": Message::text("user", "please inspect")
                }),
            )
            .unwrap();

        let messages = store.messages(&session_id).unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "thinking");
        assert_eq!(messages[1].role, "user");
    }

    #[test]
    fn session_message_rejects_tool_role() {
        let store = Store::open_memory().unwrap();
        let session_id = session(&store);
        let connection = store.connection.lock().unwrap();

        let result = connection.execute(
            "INSERT INTO session_message(message_id,session_id,role,message_json,created_at)
             VALUES (?,?,?,?,?)",
            params![
                "tool-message-1",
                session_id,
                "tool",
                serde_json::to_string(&Message::text("tool", "result")).unwrap(),
                now(),
            ],
        );

        assert!(result.is_err());
    }

    #[test]
    fn provider_api_keys_are_stored_on_provider_rows() {
        let store = Store::open_memory().unwrap();
        store
            .set_llm_provider_api_key("deepseek", "  test-key  ")
            .unwrap();

        assert_eq!(
            store.llm_provider_api_key("deepseek").unwrap().as_deref(),
            Some("test-key")
        );

        {
            let connection = store.connection.lock().unwrap();
            let (api_key,): (String,) = connection
                .query_row(
                    "SELECT api_key FROM llm_model_provider WHERE provider_id='deepseek'",
                    [],
                    |row| Ok((row.get(0)?,)),
                )
                .unwrap();
            assert_eq!(api_key, "test-key");
        }

        store.delete_llm_provider_api_key("deepseek").unwrap();
        assert!(store.llm_provider_api_key("deepseek").unwrap().is_none());
    }

    #[test]
    fn custom_provider_and_model_round_trip_without_overwriting_key() {
        let store = Store::open_memory().unwrap();
        store
            .upsert_llm_model_provider(LlmModelProviderInput {
                provider_id: "enterprise-gateway",
                display_name: "Enterprise Gateway",
                endpoint: "https://llm.example.test/v1/",
                adapter_type: "openai",
                enabled: true,
                sort_order: 5,
            })
            .unwrap();
        store
            .set_llm_provider_api_key("enterprise-gateway", "enterprise-key")
            .unwrap();
        store
            .upsert_llm_model_provider(LlmModelProviderInput {
                provider_id: "enterprise-gateway",
                display_name: "Updated Gateway",
                endpoint: "https://llm.example.test/v2",
                adapter_type: "openai",
                enabled: true,
                sort_order: 6,
            })
            .unwrap();
        store
            .upsert_llm_model(LlmModelInput {
                model_id: "enterprise-code",
                provider_id: "enterprise-gateway",
                display_name: "Enterprise Code",
                request_model: "vendor-code-v1",
                context_tokens: 32_768,
                auto_compact_tokens: 24_576,
                max_output_tokens: Some(8_192),
                supports_streaming: true,
                supports_tool_use: true,
                supports_vision: false,
                supports_structured_output: true,
                supports_cancellation: true,
                supports_reasoning_effort: true,
                enabled: true,
                sort_order: 1,
            })
            .unwrap();

        let provider = store
            .llm_model_providers(false)
            .unwrap()
            .into_iter()
            .find(|provider| provider.provider_id == "enterprise-gateway")
            .unwrap();
        assert_eq!(provider.display_name, "Updated Gateway");
        assert_eq!(provider.endpoint, "https://llm.example.test/v2");
        assert_eq!(provider.adapter_type, "openai");
        assert!(provider.api_key_configured);

        let model = store
            .llm_models(false)
            .unwrap()
            .into_iter()
            .find(|model| model.model_id == "enterprise-code")
            .unwrap();
        assert_eq!(model.provider_id, "enterprise-gateway");
        assert_eq!(model.request_model, "vendor-code-v1");
        assert_eq!(model.context_tokens, 32_768);
        assert_eq!(model.auto_compact_tokens, 24_576);
        assert!(model.supports_structured_output);
        assert!(model.supports_reasoning_effort);
    }

    #[test]
    fn disabled_provider_and_model_are_excluded_from_enabled_lists() {
        let store = Store::open_memory().unwrap();
        store
            .upsert_llm_model_provider(LlmModelProviderInput {
                provider_id: "disabled-provider",
                display_name: "Disabled Provider",
                endpoint: "https://llm.example.test",
                adapter_type: "openai",
                enabled: false,
                sort_order: 999,
            })
            .unwrap();
        store
            .upsert_llm_model(LlmModelInput {
                model_id: "disabled-model",
                provider_id: "disabled-provider",
                display_name: "Disabled Model",
                request_model: "disabled-v1",
                context_tokens: 16_000,
                auto_compact_tokens: 12_000,
                max_output_tokens: None,
                supports_streaming: true,
                supports_tool_use: false,
                supports_vision: false,
                supports_structured_output: false,
                supports_cancellation: true,
                supports_reasoning_effort: false,
                enabled: false,
                sort_order: 999,
            })
            .unwrap();

        assert!(!store
            .llm_model_providers(true)
            .unwrap()
            .iter()
            .any(|provider| provider.provider_id == "disabled-provider"));
        assert!(!store
            .llm_models(true)
            .unwrap()
            .iter()
            .any(|model| model.model_id == "disabled-model"));
        assert!(store
            .llm_models(false)
            .unwrap()
            .iter()
            .any(|model| model.model_id == "disabled-model"));
    }

    #[test]
    fn custom_provider_requires_a_known_adapter() {
        let store = Store::open_memory().unwrap();
        let error = store
            .upsert_llm_model_provider(LlmModelProviderInput {
                provider_id: "unknown-adapter",
                display_name: "Unknown Adapter",
                endpoint: "https://llm.example.test",
                adapter_type: "custom-wire-format",
                enabled: true,
                sort_order: 999,
            })
            .unwrap_err();
        assert!(error.to_string().contains("invalid fields"));
    }

    #[test]
    fn invalid_model_limits_are_rejected_before_insert() {
        let store = Store::open_memory().unwrap();
        let input = |context_tokens, auto_compact_tokens| LlmModelInput {
            model_id: "invalid-model",
            provider_id: "deepseek",
            display_name: "Invalid Model",
            request_model: "invalid-v1",
            context_tokens,
            auto_compact_tokens,
            max_output_tokens: None,
            supports_streaming: true,
            supports_tool_use: true,
            supports_vision: false,
            supports_structured_output: false,
            supports_cancellation: true,
            supports_reasoning_effort: false,
            enabled: true,
            sort_order: 0,
        };

        assert!(store.upsert_llm_model(input(15_999, 12_000)).is_err());
        assert!(store.upsert_llm_model(input(32_768, 999)).is_err());
        assert!(store.upsert_llm_model(input(32_768, 32_768)).is_err());
        assert!(store
            .upsert_llm_model(LlmModelInput {
                max_output_tokens: Some(0),
                ..input(32_768, 24_576)
            })
            .is_err());
        assert!(!store
            .llm_models(false)
            .unwrap()
            .iter()
            .any(|model| model.model_id == "invalid-model"));
    }

    #[test]
    fn provider_list_is_redacted_but_key_lookup_still_works() {
        let store = Store::open_memory().unwrap();
        store
            .set_llm_provider_api_key("deepseek", "debug-secret")
            .unwrap();
        let provider = store
            .llm_model_providers(false)
            .unwrap()
            .into_iter()
            .find(|provider| provider.provider_id == "deepseek")
            .unwrap();
        assert!(provider.api_key_configured);
        assert!(!format!("{provider:?}").contains("debug-secret"));
        assert_eq!(
            store.llm_provider_api_key("deepseek").unwrap().as_deref(),
            Some("debug-secret")
        );
    }

    #[test]
    fn configuration_resolves_global_then_project_then_session() {
        let store = Store::open_memory().unwrap();
        let project = store
            .project("/tmp/suncode-project-settings", "Settings")
            .unwrap();
        let session_id = store
            .create_session(&project.project_id, None, Some("deepseek-v4-flash"))
            .unwrap()
            .session_id;

        store
            .set_setting("global", "global", "default_model", &json!("gpt-5.5"))
            .unwrap();
        assert_eq!(
            store
                .project_default_model(&project.project_id)
                .unwrap()
                .as_deref(),
            Some("gpt-5.5")
        );
        store
            .set_setting(
                "project",
                &project.project_id,
                "default_model",
                &json!("glm-5.2"),
            )
            .unwrap();
        assert_eq!(
            store
                .project_default_model(&project.project_id)
                .unwrap()
                .as_deref(),
            Some("glm-5.2")
        );

        let effective = store
            .settings(Some(&project.project_id), Some(&session_id))
            .unwrap();
        let default_model = effective
            .iter()
            .find(|record| record.key == "default_model")
            .unwrap();
        assert_eq!(default_model.value, json!("glm-5.2"));
        assert_eq!(default_model.scope, "project");

        store
            .set_setting("session", &session_id, "default_model", &json!("kimi-k3"))
            .unwrap();
        let effective = store
            .settings(Some(&project.project_id), Some(&session_id))
            .unwrap();
        let default_model = effective
            .iter()
            .find(|record| record.key == "default_model")
            .unwrap();
        assert_eq!(default_model.value, json!("kimi-k3"));
        assert_eq!(default_model.scope, "session");

        let connection = store.connection.lock().unwrap();
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM configuration", [], |row| row
                    .get::<_, i64>(0),)
                .unwrap(),
            7
        );
    }

    #[test]
    fn project_tool_call_limit_is_typed_and_bounded() {
        let store = Store::open_memory().unwrap();
        let project = store
            .project("/tmp/suncode-tool-call-limit", "Tool limit")
            .unwrap();

        assert_eq!(
            store.project_tool_call_limit(&project.project_id).unwrap(),
            None
        );
        store
            .set_setting(
                "project",
                &project.project_id,
                "tool_call_limit",
                &json!(64),
            )
            .unwrap();
        assert_eq!(
            store.project_tool_call_limit(&project.project_id).unwrap(),
            Some(64)
        );

        for invalid in [json!(0), json!(257), json!(1.5), json!("64")] {
            store
                .set_setting("project", &project.project_id, "tool_call_limit", &invalid)
                .unwrap();
            assert!(store.project_tool_call_limit(&project.project_id).is_err());
        }
    }

    #[test]
    fn fail_turn_enriches_an_already_failed_projection() {
        let store = Store::open_memory().unwrap();
        let session_id = session(&store);
        let admission = store
            .begin_turn(
                &session_id,
                "failure-1",
                "run too many tools",
                "deepseek-v4-flash",
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "turn.state",
                &json!({
                    "turn_id": admission.turn_id,
                    "submission_idempotency_key": "failure-1",
                    "state": "failed",
                    "reason": "tool_budget_exceeded"
                }),
            )
            .unwrap();

        let error = json!({
            "code": "tool_budget_exceeded",
            "message": "Turn exceeded its tool-call budget",
            "details": {"limit": 64}
        });
        store.fail_turn(&session_id, "failure-1", &error).unwrap();

        let connection = store.connection.lock().unwrap();
        let (state, error_code, error_json): (String, Option<String>, Option<String>) = connection
            .query_row(
                "SELECT state,error_code,error_json FROM session_turn WHERE turn_id=?",
                [&admission.turn_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(state, "failed");
        assert_eq!(error_code.as_deref(), Some("tool_budget_exceeded"));
        assert_eq!(
            serde_json::from_str::<Value>(&error_json.unwrap()).unwrap(),
            error
        );
    }

    #[test]
    fn fail_turn_does_not_overwrite_other_terminal_states() {
        let store = Store::open_memory().unwrap();
        let session_id = session(&store);
        for state in ["completed", "cancelled", "interrupted"] {
            let key = format!("terminal-{state}");
            let admission = store
                .begin_turn(&session_id, &key, "terminal turn", "deepseek-v4-flash")
                .unwrap();
            store
                .append_content(
                    &session_id,
                    "turn.state",
                    &json!({
                        "turn_id": admission.turn_id,
                        "submission_idempotency_key": key,
                        "state": state
                    }),
                )
                .unwrap();
            store
                .fail_turn(
                    &session_id,
                    &key,
                    &json!({"code":"late_failure","message":"must not overwrite"}),
                )
                .unwrap();

            let connection = store.connection.lock().unwrap();
            let (stored_state, error_json): (String, Option<String>) = connection
                .query_row(
                    "SELECT state,error_json FROM session_turn WHERE turn_id=?",
                    [&admission.turn_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .unwrap();
            assert_eq!(stored_state, state);
            assert!(error_json.is_none());
        }
    }

    #[test]
    fn session_full_control_defaults_false_and_requires_a_boolean() {
        let store = Store::open_memory().unwrap();
        let project = store
            .project("/tmp/suncode-session-full-control", "Full control")
            .unwrap();
        let session_id = store
            .create_session(&project.project_id, None, Some("deepseek-v4-flash"))
            .unwrap()
            .session_id;

        assert!(!store.session_full_control(&session_id).unwrap());
        store
            .set_setting("session", &session_id, "full_control", &json!(true))
            .unwrap();
        assert!(store.session_full_control(&session_id).unwrap());
        store
            .set_setting("session", &session_id, "full_control", &json!(false))
            .unwrap();
        assert!(!store.session_full_control(&session_id).unwrap());
        store
            .set_setting("session", &session_id, "full_control", &json!("yes"))
            .unwrap();
        assert!(store.session_full_control(&session_id).is_err());
    }

    #[test]
    fn allow_session_atomically_enables_full_control_for_a_pending_approval() {
        let store = Store::open_memory().unwrap();
        let project = store
            .project("/tmp/suncode-session-approval", "Approval")
            .unwrap();
        let session = store
            .create_session(&project.project_id, None, Some("deepseek-v4-flash"))
            .unwrap();
        let admission = store
            .begin_turn(
                &session.session_id,
                "turn-key",
                "write",
                "deepseek-v4-flash",
            )
            .unwrap();
        store
            .append_content(
                &session.session_id,
                "tool.requested",
                &json!({
                    "turn_id": admission.turn_id,
                    "tool_call_id": "write-call",
                    "name": "write",
                    "arguments": {"path":"README.md","content":"updated"}
                }),
            )
            .unwrap();
        let approval = store
            .create_approval(ApprovalInput {
                project_id: Some(&project.project_id),
                session_id: &session.session_id,
                turn_id: &admission.turn_id,
                tool_call_id: "write-call",
                operation: "write",
                arguments: &json!({"path":"README.md","content":"updated"}),
                snapshot: &json!({}),
            })
            .unwrap();

        assert!(!store.session_full_control(&session.session_id).unwrap());
        assert!(store
            .resolve_approval(&approval.approval_id, "allow_session")
            .unwrap()
            .is_some());
        assert!(store.session_full_control(&session.session_id).unwrap());

        store
            .set_setting(
                "session",
                &session.session_id,
                "full_control",
                &json!(false),
            )
            .unwrap();
        assert!(store
            .resolve_approval(&approval.approval_id, "allow_session")
            .unwrap()
            .is_none());
        assert!(!store.session_full_control(&session.session_id).unwrap());
    }

    #[test]
    fn global_logging_configuration_has_typed_defaults() {
        let store = Store::open_memory().unwrap();
        let settings = store.settings(None, None).unwrap();
        let value = |key: &str| {
            settings
                .iter()
                .find(|record| record.key == key)
                .map(|record| record.value.clone())
                .unwrap()
        };
        assert_eq!(value("log_level"), json!("INFO"));
        assert_eq!(value("log_directory"), json!(""));
        assert_eq!(value("log_max_bytes"), json!(10 * 1024 * 1024));
        assert_eq!(value("log_retention"), json!(5));
    }

    #[test]
    fn project_default_model_requires_a_string_value() {
        let store = Store::open_memory().unwrap();
        let project = store
            .project("/tmp/suncode-project-setting-type", "Settings")
            .unwrap();
        store
            .set_setting(
                "project",
                &project.project_id,
                "default_model",
                &json!({"model":"gpt-5.5"}),
            )
            .unwrap();
        assert!(store.project_default_model(&project.project_id).is_err());
    }

    #[test]
    fn project_dependencies_are_unique_scoped_and_removable() {
        let store = Store::open_memory().unwrap();
        let project = store.project("/tmp/main-project", "Main").unwrap();
        let first = store
            .add_project_dependency(&project.project_id, "/tmp/shared-source", "Shared")
            .unwrap();
        let repeated = store
            .add_project_dependency(&project.project_id, "/tmp/shared-source", "Renamed")
            .unwrap();
        assert_eq!(first.dependency_id, repeated.dependency_id);
        assert_eq!(
            store.project_dependencies(&project.project_id).unwrap()[0].display_name,
            "Renamed"
        );
        assert!(store
            .add_project_dependency(&project.project_id, "/tmp/main-project", "Self")
            .is_err());
        assert!(store
            .remove_project_dependency(&project.project_id, &first.dependency_id)
            .unwrap());
        assert!(store
            .project_dependencies(&project.project_id)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn fresh_schema_is_complete_and_consistent() {
        let store = Store::open_memory().unwrap();
        let health = store.health().unwrap();
        assert_eq!(health["ok"], true);
        assert!(health.get("schema_version").is_none());

        let connection = store.connection.lock().unwrap();
        assert_eq!(
            schema::table_names(&connection).unwrap(),
            schema::TABLE_NAMES
        );
        assert_eq!(
            object_names(&connection, "index"),
            [
                "approval_request_session_status_idx",
                "audit_record_occurred_idx",
                "audit_record_project_time_idx",
                "audit_record_session_time_idx",
                "audit_record_turn_time_idx",
                "checkpoint_manifest_expiry_idx",
                "checkpoint_manifest_ordinal_idx",
                "checkpoint_manifest_session_status_idx",
                "checkpoint_manifest_turn_idx",
                "configuration_global_key_idx",
                "configuration_project_key_idx",
                "configuration_session_key_idx",
                "llm_model_by_provider_enabled_order_idx",
                "llm_model_enabled_order_idx",
                "llm_model_provider_enabled_order_idx",
                "project_dependency_project_name_idx",
                "project_last_opened_idx",
                "session_call_session_started_idx",
                "session_call_started_idx",
                "session_call_turn_idx",
                "session_message_call_idx",
                "session_message_session_created_idx",
                "session_project_activity_idx",
                "session_tool_use_call_idx",
                "session_tool_use_turn_state_idx",
                "session_turn_recovery_idx",
                "session_turn_resuming_idx",
                "session_turn_session_created_idx",
                "session_turn_todo_turn_status_idx",
            ]
        );
        assert_eq!(
            object_names(&connection, "trigger"),
            ["audit_record_no_delete", "audit_record_no_update"]
        );

        schema::apply(&connection).unwrap();
        data::apply(&connection).unwrap();
        assert_eq!(
            connection
                .query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
                .unwrap(),
            "ok"
        );
        let foreign_key_errors: i64 = connection
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(foreign_key_errors, 0);
    }

    #[test]
    fn incompatible_database_is_rejected_without_conversion() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("state.db");
        let connection = Connection::open(&database).unwrap();
        connection
            .execute_batch("CREATE TABLE legacy_state (id INTEGER PRIMARY KEY);")
            .unwrap();
        drop(connection);

        let error = Store::open(&database).err().unwrap();
        assert!(error
            .to_string()
            .contains("do not match the current schema"));

        let connection = Connection::open(&database).unwrap();
        assert_eq!(
            schema::table_names(&connection).unwrap(),
            vec!["legacy_state".to_string()]
        );
    }

    #[test]
    fn current_database_receives_the_additive_dependency_table() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("state.db");
        let connection = Connection::open(&database).unwrap();
        schema::apply(&connection).unwrap();
        data::apply(&connection).unwrap();
        connection
            .execute_batch("DROP TABLE project_dependency;")
            .unwrap();
        assert_eq!(schema::table_names(&connection).unwrap().len(), 14);
        drop(connection);

        let store = Store::open(&database).unwrap();
        let connection = store.connection.lock().unwrap();
        assert_eq!(
            schema::table_names(&connection).unwrap(),
            schema::TABLE_NAMES
        );
    }

    #[test]
    fn former_projects_table_is_rejected_without_being_renamed() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("state.db");
        let connection = Connection::open(&database).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE projects (
                    project_id TEXT PRIMARY KEY,
                    canonical_root TEXT NOT NULL,
                    display_name TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    last_opened_at TEXT NOT NULL,
                    archived_at TEXT
                );",
            )
            .unwrap();
        drop(connection);

        let error = Store::open(&database).err().unwrap();
        assert!(error
            .to_string()
            .contains("do not match the current schema"));

        let connection = Connection::open(&database).unwrap();
        assert_eq!(
            schema::table_names(&connection).unwrap(),
            vec!["projects".to_string()]
        );
    }

    #[test]
    fn database_whose_message_schema_permits_tool_role_is_rejected() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("state.db");
        let connection = Connection::open(&database).unwrap();
        schema::apply(&connection).unwrap();
        data::apply(&connection).unwrap();
        connection
            .execute_batch(
                "PRAGMA foreign_keys=OFF;
                 ALTER TABLE session_message RENAME TO session_message_current;
                 CREATE TABLE session_message (
                    message_id TEXT PRIMARY KEY,
                    session_id TEXT NOT NULL,
                    turn_id TEXT,
                    session_call_id TEXT,
                    role TEXT NOT NULL CHECK(role IN ('user','assistant','thinking','tool')),
                    message_json TEXT NOT NULL CHECK(json_valid(message_json)),
                    usage_json TEXT CHECK(usage_json IS NULL OR json_valid(usage_json)),
                    created_at TEXT NOT NULL
                 );
                 DROP TABLE session_message_current;",
            )
            .unwrap();
        drop(connection);

        let error = Store::open(&database).err().unwrap();

        assert!(error
            .to_string()
            .contains("session_message schema still permits the retired tool role"));
    }

    #[test]
    fn database_whose_message_schema_contains_usage_column_is_rejected() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("state.db");
        let connection = Connection::open(&database).unwrap();
        schema::apply(&connection).unwrap();
        data::apply(&connection).unwrap();
        connection
            .execute_batch("ALTER TABLE session_message ADD COLUMN usage_json TEXT;")
            .unwrap();
        drop(connection);

        let error = Store::open(&database).err().unwrap();

        assert!(error
            .to_string()
            .contains("session_message schema still contains the retired usage_json column"));
    }

    #[test]
    fn database_whose_call_schema_lacks_provider_ids_is_rejected() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("state.db");
        let connection = Connection::open(&database).unwrap();
        schema::apply(&connection).unwrap();
        data::apply(&connection).unwrap();
        connection
            .execute_batch(
                "ALTER TABLE session_call DROP COLUMN provider_request_id;
                 ALTER TABLE session_call DROP COLUMN provider_response_id;",
            )
            .unwrap();
        drop(connection);

        let error = Store::open(&database).err().unwrap();

        assert!(error
            .to_string()
            .contains("session_call schema is missing provider request/response identifiers"));
    }

    #[test]
    fn terminal_turns_release_recovery_snapshots() {
        let store = Store::open_memory().unwrap();
        let session_id = session(&store);
        store
            .append_content(
                &session_id,
                "turn.state",
                &json!({"turn_id":"turn-1","state":"resolving_calls"}),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "tool.state",
                &json!({
                    "turn_id":"turn-1",
                    "tool_call_id":"call-1",
                    "name":"write",
                    "state":"awaiting_approval"
                }),
            )
            .unwrap();
        let approval = store
            .create_approval(ApprovalInput {
                project_id: None,
                session_id: &session_id,
                turn_id: "turn-1",
                tool_call_id: "call-1",
                operation: "write",
                arguments: &json!({"path":"README.md"}),
                snapshot: &json!({"messages":[{"role":"user","content":"sensitive"}]}),
            })
            .unwrap();

        store
            .finish_suspended(&approval.approval_id, "resuming")
            .unwrap();
        {
            let connection = store.connection.lock().unwrap();
            let snapshot: String = connection
                .query_row(
                    "SELECT recovery_snapshot_json FROM session_turn WHERE recovery_approval_id=?",
                    [&approval.approval_id],
                    |row| row.get(0),
                )
                .unwrap();
            assert_ne!(snapshot, "{}");
        }

        store
            .finish_suspended(&approval.approval_id, "completed")
            .unwrap();
        let connection = store.connection.lock().unwrap();
        let snapshot: String = connection
            .query_row(
                "SELECT recovery_snapshot_json FROM session_turn WHERE recovery_approval_id=?",
                [&approval.approval_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(snapshot, "{}");
    }

    #[test]
    fn session_usage_replaces_turn_totals_and_sums_across_turns() {
        let store = Store::open_memory().unwrap();
        let session_id = session(&store);
        for turn_id in ["turn-1", "turn-2"] {
            store
                .append_content(
                    &session_id,
                    "turn.state",
                    &json!({"turn_id":turn_id,"state":"admitted","model_id":"deepseek-v4-flash"}),
                )
                .unwrap();
        }
        store
            .append_content(
                &session_id,
                "usage.updated",
                &json!({"turn_id":"turn-1","usage":{"input_tokens":100,"output_tokens":20,"total_tokens":120}}),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "usage.updated",
                &json!({"turn_id":"turn-1","usage":{"input_tokens":180,"output_tokens":30,"total_tokens":210}}),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "usage.updated",
                &json!({"turn_id":"turn-2","usage":{"input_tokens":40,"output_tokens":10,"total_tokens":50}}),
            )
            .unwrap();

        assert_eq!(
            store.session_usage(&session_id).unwrap(),
            Usage {
                input_tokens: 220,
                output_tokens: 40,
                total_tokens: 260,
            }
        );
    }

    #[test]
    fn provider_exchange_events_project_to_queryable_traces() {
        let store = Store::open_memory().unwrap();
        let session_id = session(&store);
        store
            .append_content(
                &session_id,
                "turn.state",
                &json!({"turn_id":"turn-1","state":"calling_model","model_id":"deepseek-v4-flash"}),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "turn.state",
                &json!({"turn_id":"turn-without-call","state":"completed","model_id":"deepseek-v4-flash"}),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "provider.exchange.started",
                &json!({
                    "exchange_id":"exchange-1",
                    "turn_id":"turn-1",
                    "provider":"deepseek",
                    "model_id":"deepseek-v4-flash",
                    "wire_model":"deepseek-chat",
                    "iteration":1,
                    "input_messages":[{"role":"user","content":[{"type":"text","text":"hello"}]}]
                }),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "provider.exchange.completed",
                &json!({
                    "exchange_id":"exchange-1",
                    "turn_id":"turn-1",
                    "provider_request_id":"request-1",
                    "provider_response_id":"chatcmpl-response-1",
                    "output_message":{"role":"assistant","content":[{"type":"text","text":"hi"}]},
                    "tool_calls":[],
                    "usage":{"input_tokens":4,"output_tokens":2,"total_tokens":6,"cache_read_tokens":3,"cache_miss_tokens":1,"cache_write_tokens":null,"reasoning_tokens":2},
                    "finish_reason":"stop"
                }),
            )
            .unwrap();

        let exchanges = store.provider_exchanges(&session_id).unwrap();
        assert_eq!(exchanges.len(), 1);
        assert_eq!(exchanges[0].exchange_id, "exchange-1");
        assert_eq!(exchanges[0].usage.as_ref().unwrap()["cache_read_tokens"], 3);
        assert_eq!(exchanges[0].usage.as_ref().unwrap()["cache_miss_tokens"], 1);
        assert_eq!(exchanges[0].usage.as_ref().unwrap()["reasoning_tokens"], 2);
        assert_eq!(exchanges[0].state, "completed");
        assert_eq!(exchanges[0].provider, "deepseek");
        assert_eq!(
            exchanges[0].provider_request_id.as_deref(),
            Some("request-1")
        );
        assert_eq!(
            exchanges[0].provider_response_id.as_deref(),
            Some("chatcmpl-response-1")
        );
        assert_eq!(exchanges[0].usage.as_ref().unwrap()["total_tokens"], 6);
        let turns = store.session_trace_turns(&session_id).unwrap();
        assert_eq!(turns.len(), 2);
        assert!(turns.iter().any(|turn| turn.turn_id == "turn-without-call"));
        assert_eq!(
            store
                .provider_exchange(&session_id, "exchange-1")
                .unwrap()
                .unwrap()
                .finish_reason
                .as_deref(),
            Some("stop")
        );
    }

    #[test]
    fn tool_use_retains_call_correlation_and_result_without_tool_message_row() {
        let store = Store::open_memory().unwrap();
        let session_id = session(&store);
        store
            .append_content(
                &session_id,
                "turn.state",
                &json!({"turn_id":"turn-1","state":"resolving_calls"}),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "provider.exchange.started",
                &json!({
                    "exchange_id":"call-1",
                    "turn_id":"turn-1",
                    "provider":"deepseek",
                    "model_id":"deepseek-v4-flash",
                    "wire_model":"deepseek-chat",
                    "input_messages":[]
                }),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "tool.requested",
                &json!({
                    "turn_id":"turn-1",
                    "call_id":"call-1",
                    "tool_call_id":"tool-1",
                    "name":"read",
                    "arguments":{"path":"README.md"}
                }),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "tool.result",
                &json!({
                    "turn_id":"turn-1",
                    "call_id":"call-1",
                    "tool_call_id":"tool-1",
                    "result":{"content":"hello"}
                }),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "message.tool",
                &json!({"turn_id":"turn-1","call_id":"call-1","tool_call_id":"tool-1","message":{"role":"tool","content":[{"type":"text","text":"hello"}]}}),
            )
            .unwrap();

        let connection = store.connection.lock().unwrap();
        let tool_row: (String, String, String) = connection
            .query_row(
                "SELECT session_call_id,request_json,result_json FROM session_tool_use WHERE turn_id=? AND tool_call_id=?",
                params!["turn-1", "tool-1"],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(tool_row.0, "call-1");
        assert_eq!(
            serde_json::from_str::<Value>(&tool_row.1).unwrap()["path"],
            "README.md"
        );
        assert_eq!(
            serde_json::from_str::<Value>(&tool_row.2).unwrap()["content"],
            "hello"
        );
        let tool_message_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM session_message WHERE role='tool'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(tool_message_count, 0);
        drop(connection);

        let messages = store.session_call_messages(&session_id, "call-1").unwrap();
        assert!(messages.is_empty());
        let tools = store.session_call_tool_uses("call-1").unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "read");
        assert_eq!(tools[0].request.as_ref().unwrap()["path"], "README.md");
        assert_eq!(tools[0].result.as_ref().unwrap()["content"], "hello");
    }

    #[test]
    fn ephemeral_deltas_do_not_create_message_rows() {
        let store = Store::open_memory().unwrap();
        let session_id = session(&store);
        store
            .append_content(
                &session_id,
                "assistant.delta",
                &json!({"turn_id":"turn-1","text":"partial"}),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "message.assistant",
                &json!({"message_id":"assistant-1","turn_id":"turn-1","message":Message::text("assistant","complete")}),
            )
            .unwrap();

        assert!(store.messages(&session_id).unwrap().iter().any(|message| {
            message.role == "assistant" && message.text_content() == "complete"
        }));
    }

    #[test]
    fn context_messages_drop_incomplete_tool_exchange_tail() {
        let store = Store::open_memory().unwrap();
        let session_id = session(&store);
        let assistant = Message {
            role: "assistant".into(),
            content: Vec::new(),
            tool_calls: vec![ToolCall {
                call_id: "call-1".into(),
                name: "write".into(),
                arguments: json!({"path":"README.md","content":"updated"}),
            }],
            tool_call_id: None,
        };

        store
            .append_content(
                &session_id,
                "message.user",
                &json!({"message_id":"user-1","turn_id":"turn-1","message":Message::text("user","write")}),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "message.assistant",
                &json!({"message_id":"assistant-1","turn_id":"turn-1","message":assistant}),
            )
            .unwrap();

        let messages = store.context_messages(&session_id).unwrap();
        assert_eq!(messages, vec![Message::text("user", "write")]);
    }

    #[test]
    fn context_messages_drop_partial_multi_tool_exchange_tail() {
        let store = Store::open_memory().unwrap();
        let session_id = session(&store);
        let assistant = Message {
            role: "assistant".into(),
            content: Vec::new(),
            tool_calls: vec![
                ToolCall {
                    call_id: "call-1".into(),
                    name: "read".into(),
                    arguments: json!({"path":"README.md"}),
                },
                ToolCall {
                    call_id: "call-2".into(),
                    name: "read".into(),
                    arguments: json!({"path":"Cargo.toml"}),
                },
            ],
            tool_call_id: None,
        };
        let mut tool = Message::text("tool", "{\"content\":\"hello\"}");
        tool.tool_call_id = Some("call-1".into());

        store
            .append_content(
                &session_id,
                "turn.state",
                &json!({"turn_id":"turn-1","state":"resolving_calls"}),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "message.user",
                &json!({"message_id":"user-1","turn_id":"turn-1","message":Message::text("user","read two")}),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "message.assistant",
                &json!({"message_id":"assistant-1","turn_id":"turn-1","message":assistant}),
            )
            .unwrap();
        store
            .append_content(
                &session_id,
                "message.tool",
                &json!({"turn_id":"turn-1","tool_call_id":"call-1","message":tool}),
            )
            .unwrap();

        let messages = store.context_messages(&session_id).unwrap();
        assert_eq!(messages, vec![Message::text("user", "read two")]);
    }
}
