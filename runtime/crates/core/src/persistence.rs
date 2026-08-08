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

pub(crate) struct ApprovalInput<'a> {
    pub(crate) project_id: Option<&'a str>,
    pub(crate) session_id: &'a str,
    pub(crate) turn_id: &'a str,
    pub(crate) tool_call_id: &'a str,
    pub(crate) operation: &'a str,
    pub(crate) arguments: &'a Value,
    pub(crate) snapshot: &'a Value,
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
        let transaction = connection.transaction()?;
        transaction.execute_batch(include_str!("schema.sql"))?;
        mark_schema_version(&transaction)?;
        transaction.commit()?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    #[cfg(test)]
    pub fn open_memory() -> Result<Self, PersistenceError> {
        let mut connection = Connection::open_in_memory()?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        let transaction = connection.transaction()?;
        transaction.execute_batch(include_str!("schema.sql"))?;
        mark_schema_version(&transaction)?;
        transaction.commit()?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub fn health(&self) -> Result<Value, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let version: i64 = connection.query_row(
            "SELECT COALESCE(MAX(version),0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )?;
        Ok(json!({"ok": true, "schema_version": version, "journal_mode": "wal"}))
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
        transaction.execute(
            "INSERT OR IGNORE INTO session_sequences(session_id,next_content_sequence)
             VALUES (?, COALESCE((SELECT MAX(content_sequence)+1 FROM session_content WHERE session_id=?),1))",
            params![session_id, session_id],
        )?;
        let sequence: i64 = transaction.query_row(
            "SELECT next_content_sequence FROM session_sequences WHERE session_id=?",
            [session_id],
            |row| row.get(0),
        )?;
        transaction.execute(
            "UPDATE session_sequences SET next_content_sequence=? WHERE session_id=?",
            params![sequence + 1, session_id],
        )?;
        let payload_json = serde_json::to_string(payload)?;
        transaction.execute("INSERT INTO session_content (session_id,content_sequence,occurred_at,event_type,payload_json) VALUES (?,?,?,?,?)", params![session_id, sequence, now, event_type, payload_json])?;
        apply_projection(
            &transaction,
            session_id,
            sequence,
            &now,
            event_type,
            payload,
        )?;
        transaction.execute(
            "UPDATE sessions SET updated_at = ?, last_activity_at = ? WHERE session_id = ?",
            params![now, now, session_id],
        )?;
        transaction.commit()?;
        Ok(SessionEvent {
            session_id: session_id.to_string(),
            content_sequence: sequence,
            occurred_at: now,
            event_type: event_type.to_string(),
            payload: payload.clone(),
        })
    }

    pub fn events(
        &self,
        session_id: &str,
        after: i64,
    ) -> Result<Vec<SessionEvent>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let mut statement = connection.prepare(
            "SELECT content_sequence,occurred_at,event_type,payload_json
             FROM session_content
             WHERE session_id=?
               AND content_sequence>?
               AND event_type NOT IN ('assistant.delta','reasoning.delta','tool.input.delta')
             ORDER BY content_sequence",
        )?;
        let rows = statement.query_map(params![session_id, after], |row| {
            let payload: String = row.get(3)?;
            Ok(SessionEvent {
                session_id: session_id.to_string(),
                content_sequence: row.get(0)?,
                occurred_at: row.get(1)?,
                event_type: row.get(2)?,
                payload: serde_json::from_str(&payload).unwrap_or(Value::Null),
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn messages(&self, session_id: &str) -> Result<Vec<Message>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let mut statement = connection.prepare("SELECT message_json FROM session_messages WHERE session_id=? ORDER BY content_sequence")?;
        let rows = statement.query_map([session_id], |row| row.get::<_, String>(0))?;
        rows.map(|row| Ok(serde_json::from_str(&row?)?)).collect()
    }

    pub fn context_messages(&self, session_id: &str) -> Result<Vec<Message>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let mut statement = connection.prepare(
            "SELECT message_json FROM session_messages WHERE session_id=? ORDER BY content_sequence",
        )?;
        let rows = statement.query_map([session_id], |row| row.get::<_, String>(0))?;
        let mut messages: Vec<Message> = Vec::new();
        for row in rows {
            messages.push(serde_json::from_str(&row?)?);
        }
        Ok(repair_incomplete_tool_exchanges(messages))
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
            "SELECT status,turn_id,input_json,model_id,response_json,error_json FROM turn_submissions WHERE session_id=? AND idempotency_key=?",
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
        connection.execute("INSERT INTO turn_submissions (session_id,idempotency_key,status,created_at,updated_at,turn_id,input_json,model_id,admitted_at) VALUES (?,?, 'pending',?,?,?,?,?,?)", params![session_id, key, timestamp, timestamp, turn_id, serde_json::to_string(&json!({"input": input}))?, model, timestamp])?;
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
        connection.execute("UPDATE turn_submissions SET started_at=COALESCE(started_at,?),updated_at=? WHERE session_id=? AND idempotency_key=? AND status='pending'", params![now(), now(), session_id, key])?;
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
        connection.execute("UPDATE turn_submissions SET status='completed',response_json=?,error_json=NULL,completed_at=?,updated_at=? WHERE session_id=? AND idempotency_key=? AND status='pending'", params![serde_json::to_string(response)?, timestamp, timestamp, session_id, key])?;
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
        connection.execute("UPDATE turn_submissions SET status='failed',error_json=?,completed_at=?,updated_at=? WHERE session_id=? AND idempotency_key=? AND status='pending'", params![serde_json::to_string(error)?, timestamp, timestamp, session_id, key])?;
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
        connection.execute("INSERT INTO audit_records(project_id,session_id,turn_id,occurred_at,event_type,payload_json) VALUES (?,?,?,?,?,?)", params![project_id, session_id, turn_id, now(), event_type, serde_json::to_string(payload)?])?;
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
                "SELECT approval_id FROM approval_requests WHERE idempotency_key=?",
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
        transaction.execute("INSERT INTO approval_requests(approval_id,project_id,session_id,turn_id,tool_call_id,operation,arguments_json,idempotency_key,status,created_at,updated_at) VALUES (?,?,?,?,?,?,?,?, 'pending',?,?)", params![id, input.project_id, input.session_id, input.turn_id, input.tool_call_id, input.operation, serde_json::to_string(input.arguments)?, key, timestamp, timestamp])?;
        transaction.execute("INSERT INTO suspended_turns(approval_id,session_id,turn_id,snapshot_json,status,created_at,updated_at) VALUES (?,?,?,?, 'pending',?,?)", params![id, input.session_id, input.turn_id, serde_json::to_string(input.snapshot)?, timestamp, timestamp])?;
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
        let changed = transaction.execute("UPDATE approval_requests SET status=?,decision=?,decision_source='user',updated_at=? WHERE approval_id=? AND status='pending'", params![if approved {"approved"} else {"denied"}, decision, now(), id])?;
        if changed == 0 {
            transaction.rollback()?;
            return Ok(None);
        }
        let row: (String, String, String, String) = transaction.query_row("SELECT approval_id,session_id,turn_id,snapshot_json FROM suspended_turns WHERE approval_id=? AND status='pending'", [id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?)))?;
        transaction.execute(
            "UPDATE suspended_turns SET status=?,updated_at=? WHERE approval_id=?",
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
        connection.execute(
            "UPDATE suspended_turns SET status=?,updated_at=? WHERE approval_id=?",
            params![status, now(), id],
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
        connection.execute("INSERT INTO checkpoint_manifests(manifest_id,session_id,turn_id,status,created_at,updated_at,expires_at) VALUES (?,?,?,'available',?,?,?)", params![id, session_id, turn_id, timestamp, timestamp, expires])?;
        manifest_by_turn(&connection, turn_id)?
            .ok_or_else(|| PersistenceError::Invalid("checkpoint manifest creation failed".into()))
    }

    pub fn manifests(&self, session_id: &str) -> Result<Vec<CheckpointManifest>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        connection.execute("UPDATE checkpoint_manifests SET status='expired',updated_at=? WHERE session_id=? AND status='available' AND expires_at<=?", params![now(), session_id, now()])?;
        let mut statement = connection.prepare("SELECT manifest_id,session_id,turn_id,status,created_at,updated_at,expires_at,restored_at FROM checkpoint_manifests WHERE session_id=? ORDER BY created_at DESC")?;
        let rows = statement.query_map([session_id], manifest_from_row)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn manifest(&self, id: &str) -> Result<Option<CheckpointManifest>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        connection.query_row("SELECT manifest_id,session_id,turn_id,status,created_at,updated_at,expires_at,restored_at FROM checkpoint_manifests WHERE manifest_id=?", [id], manifest_from_row).optional().map_err(Into::into)
    }

    pub fn checkpoint_items(
        &self,
        manifest_id: &str,
    ) -> Result<Vec<CheckpointItem>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let mut statement = connection.prepare("SELECT checkpoint_id,manifest_id,session_id,turn_id,tool_call_id,relative_path,status,created_at,restored_at,invalidated_at,ordinal FROM checkpoints WHERE manifest_id=? ORDER BY ordinal DESC")?;
        let rows = statement.query_map([manifest_id], checkpoint_from_row)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn set_manifest_status(&self, id: &str, status: &str) -> Result<(), PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        connection.execute("UPDATE checkpoint_manifests SET status=?,updated_at=?,restored_at=CASE WHEN ?='restored' THEN ? ELSE restored_at END WHERE manifest_id=?", params![status, now(), status, now(), id])?;
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
        for (scope, scope_id) in [
            ("user", Some("default")),
            ("project", project_id),
            ("session", session_id),
        ] {
            let Some(scope_id) = scope_id else { continue };
            let mut statement = connection.prepare(
                "SELECT key,value_json FROM setting_records WHERE scope=? AND scope_id=? ORDER BY key",
            )?;
            let rows = statement.query_map(params![scope, scope_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            for row in rows {
                let (key, value) = row?;
                records.retain(|record: &SettingRecord| record.key != key);
                records.push(SettingRecord {
                    key,
                    value: serde_json::from_str(&value)?,
                    scope: scope.into(),
                    scope_id: scope_id.into(),
                });
            }
        }
        Ok(records)
    }

    pub fn set_setting(
        &self,
        scope: &str,
        scope_id: &str,
        key: &str,
        value: &Value,
    ) -> Result<(), PersistenceError> {
        if !matches!(scope, "user" | "project" | "session") || scope_id.is_empty() || key.is_empty()
        {
            return Err(PersistenceError::Invalid(
                "setting scope, scope id, and key are required".into(),
            ));
        }
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        connection.execute("INSERT INTO setting_records(scope,scope_id,key,value_json,updated_at) VALUES (?,?,?,?,?) ON CONFLICT(scope,scope_id,key) DO UPDATE SET value_json=excluded.value_json,updated_at=excluded.updated_at",params![scope,scope_id,key,serde_json::to_string(value)?,now()])?;
        Ok(())
    }

    pub fn recover_startup(&self) -> Result<Vec<SessionEvent>, PersistenceError> {
        let rows = {
            let connection = self
                .connection
                .lock()
                .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
            let mut statement = connection.prepare("SELECT turn_id,session_id,submission_idempotency_key,model_id FROM turns WHERE state NOT IN ('completed','failed','cancelled','interrupted') AND turn_id NOT IN (SELECT turn_id FROM suspended_turns WHERE status IN ('pending','resuming'))")?;
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
        let mut statement = connection.prepare("SELECT approval_id,session_id,turn_id,snapshot_json,status FROM suspended_turns WHERE status='resuming'")?;
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
                "SELECT project_id FROM projects WHERE canonical_root=?",
                [root],
                |row| row.get(0),
            )
            .optional()?;
        let id = existing.unwrap_or_else(|| Uuid::new_v4().to_string());
        connection.execute("INSERT INTO projects (project_id,canonical_root,display_name,created_at,updated_at,last_opened_at,archived_at) VALUES (?,?,?,?,?,?,NULL) ON CONFLICT(project_id) DO UPDATE SET display_name=excluded.display_name,updated_at=excluded.updated_at,last_opened_at=excluded.last_opened_at,archived_at=NULL", params![id, root, display_name, timestamp, timestamp, timestamp])?;
        self.project_by_id_locked(&connection, &id)
    }

    pub fn projects(&self, include_archived: bool) -> Result<Vec<ProjectRecord>, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        let sql = if include_archived {
            "SELECT project_id,canonical_root,display_name,created_at,updated_at,last_opened_at,archived_at FROM projects ORDER BY last_opened_at DESC"
        } else {
            "SELECT project_id,canonical_root,display_name,created_at,updated_at,last_opened_at,archived_at FROM projects WHERE archived_at IS NULL ORDER BY last_opened_at DESC"
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
            "SELECT EXISTS(SELECT 1 FROM projects WHERE project_id=? AND archived_at IS NULL)",
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
        connection.execute("INSERT INTO sessions (session_id,project_id,title,model_id,status,created_at,updated_at,last_activity_at,archived_at) VALUES (?,?,?,?,?,?,?,?,NULL)", params![id, project_id, title, model_id, "active", timestamp, timestamp, timestamp])?;
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
            "SELECT session_id,project_id,title,model_id,status,created_at,updated_at,last_activity_at,archived_at FROM sessions WHERE project_id=? ORDER BY last_activity_at DESC,session_id"
        } else {
            "SELECT session_id,project_id,title,model_id,status,created_at,updated_at,last_activity_at,archived_at FROM sessions WHERE project_id=? AND status='active' ORDER BY last_activity_at DESC,session_id"
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
            "UPDATE sessions SET status=?,archived_at=?,updated_at=? WHERE session_id=?",
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

    pub fn rename_session(&self, id: &str, title: &str) -> Result<SessionRecord, PersistenceError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| PersistenceError::Invalid("database lock poisoned".into()))?;
        connection.execute(
            "UPDATE sessions SET title=?,updated_at=? WHERE session_id=?",
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
        connection.query_row("SELECT project_id,canonical_root,display_name,created_at,updated_at,last_opened_at,archived_at FROM projects WHERE project_id=?", [id], project_from_row).optional().map_err(Into::into)
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
        connection.query_row("SELECT session_id,project_id,title,model_id,status,created_at,updated_at,last_activity_at,archived_at FROM sessions WHERE session_id=?", [id], session_from_row).optional().map_err(Into::into)
    }
}

fn apply_projection(
    transaction: &Transaction<'_>,
    session_id: &str,
    sequence: i64,
    occurred_at: &str,
    event_type: &str,
    payload: &Value,
) -> Result<(), PersistenceError> {
    if matches!(
        event_type,
        "message.user" | "message.assistant" | "message.tool"
    ) {
        if let Some(message) = payload.get("message") {
            let role = message.get("role").and_then(Value::as_str);
            if matches!(role, Some("user" | "assistant" | "tool")) {
                let message_id = payload
                    .get("message_id")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("event:{session_id}:{sequence}"));
                transaction.execute("INSERT OR IGNORE INTO session_messages (message_id,session_id,turn_id,content_sequence,role,message_json,usage_json,created_at) VALUES (?,?,?,?,?,?,?,?)", params![message_id, session_id, payload.get("turn_id").and_then(Value::as_str), sequence, role, serde_json::to_string(message)?, payload.get("usage").map(serde_json::to_string).transpose()?, occurred_at])?;
            }
        }
    }
    if event_type == "turn.state" {
        if let (Some(turn_id), Some(state)) = (
            payload.get("turn_id").and_then(Value::as_str),
            payload.get("state").and_then(Value::as_str),
        ) {
            let terminal = matches!(state, "completed" | "failed" | "cancelled" | "interrupted");
            transaction.execute("INSERT INTO turns(turn_id,session_id,submission_idempotency_key,state,model_id,created_at,updated_at,completed_at,error_code) VALUES (?,?,?,?,?,?,?,?,?) ON CONFLICT(turn_id) DO UPDATE SET state=excluded.state,updated_at=excluded.updated_at,completed_at=excluded.completed_at,error_code=excluded.error_code", params![turn_id, session_id, payload.get("submission_idempotency_key").and_then(Value::as_str), state, payload.get("model_id").and_then(Value::as_str), occurred_at, occurred_at, if terminal {Some(occurred_at)} else {None}, payload.get("reason").and_then(Value::as_str)])?;
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
            transaction.execute("INSERT INTO tool_calls(turn_id,tool_call_id,name,state,ordinal,created_at,updated_at,completed_at,error_code) VALUES (?,?,?,?,?,?,?,?,?) ON CONFLICT(turn_id,tool_call_id) DO UPDATE SET name=excluded.name,state=excluded.state,ordinal=COALESCE(excluded.ordinal,tool_calls.ordinal),updated_at=excluded.updated_at,completed_at=excluded.completed_at,error_code=excluded.error_code", params![turn_id,call_id,name,state,payload.get("ordinal").and_then(Value::as_i64),occurred_at,occurred_at,if terminal {Some(occurred_at)} else {None},payload.get("reason").and_then(Value::as_str)])?;
        }
    }
    if event_type == "checkpoint.captured" {
        if let Some(checkpoint_id) = payload.get("checkpoint_id").and_then(Value::as_str) {
            transaction.execute("INSERT OR IGNORE INTO checkpoints(checkpoint_id,manifest_id,session_id,turn_id,tool_call_id,relative_path,status,created_at,ordinal) VALUES (?,?,?,?,?,?,'available',?,?)", params![checkpoint_id,payload.get("manifest_id").and_then(Value::as_str),session_id,payload.get("turn_id").and_then(Value::as_str),payload.get("tool_call_id").and_then(Value::as_str),payload.get("path").and_then(Value::as_str),occurred_at,payload.get("ordinal").and_then(Value::as_i64)])?;
        }
    }
    if event_type == "checkpoint.item_restored" {
        if let Some(checkpoint_id) = payload.get("checkpoint_id").and_then(Value::as_str) {
            transaction.execute("UPDATE checkpoints SET status='restored',restored_at=? WHERE checkpoint_id=? AND session_id=?", params![occurred_at,checkpoint_id,session_id])?;
        }
    }
    Ok(())
}

fn approval_by_id(
    connection: &Connection,
    id: &str,
) -> Result<Option<ApprovalRecord>, PersistenceError> {
    let row: Option<ApprovalRow> = connection.query_row("SELECT approval_id,project_id,session_id,turn_id,tool_call_id,operation,arguments_json,status,decision,decision_source,created_at,updated_at FROM approval_requests WHERE approval_id=?", [id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?,row.get(6)?,row.get(7)?,row.get(8)?,row.get(9)?,row.get(10)?,row.get(11)?))).optional()?;
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
    connection.query_row("SELECT manifest_id,session_id,turn_id,status,created_at,updated_at,expires_at,restored_at FROM checkpoint_manifests WHERE turn_id=?", [turn_id], manifest_from_row).optional().map_err(Into::into)
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
        archived_at: row.get(8)?,
    })
}
fn now() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
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

fn mark_schema_version(connection: &Connection) -> Result<(), PersistenceError> {
    let version: i64 = connection.query_row(
        "SELECT COALESCE(MAX(version),0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;
    if version > 10 {
        return Err(PersistenceError::Invalid(format!(
            "database schema version {version} is newer than supported version 10"
        )));
    }
    if version < 10 {
        connection.execute(
            "INSERT INTO session_sequences(session_id,next_content_sequence)
             SELECT session_id, COALESCE(MAX(content_sequence),0)+1
             FROM session_content
             GROUP BY session_id
             ON CONFLICT(session_id) DO UPDATE SET
                next_content_sequence=max(session_sequences.next_content_sequence,excluded.next_content_sequence)",
            [],
        )?;
        connection.execute(
            "DELETE FROM session_content WHERE event_type IN ('assistant.delta', 'reasoning.delta', 'tool.input.delta')",
            [],
        )?;
        connection.execute_batch(
            "DROP INDEX IF EXISTS session_messages_session_role_seq_idx;
             CREATE TABLE IF NOT EXISTS session_messages_v10 (
                message_id TEXT PRIMARY KEY CHECK(length(message_id)>0),
                session_id TEXT NOT NULL REFERENCES sessions(session_id) ON DELETE CASCADE,
                turn_id TEXT,
                content_sequence INTEGER NOT NULL CHECK(content_sequence>0),
                role TEXT NOT NULL CHECK(role IN ('user','assistant','tool')),
                message_json TEXT NOT NULL CHECK(json_valid(message_json)),
                usage_json TEXT CHECK(usage_json IS NULL OR json_valid(usage_json)),
                created_at TEXT NOT NULL,
                UNIQUE(session_id,content_sequence)
             );
             INSERT OR IGNORE INTO session_messages_v10 (message_id,session_id,turn_id,content_sequence,role,message_json,usage_json,created_at)
                SELECT message_id,session_id,turn_id,content_sequence,role,message_json,usage_json,created_at
                FROM session_messages;
             DROP TABLE session_messages;
             ALTER TABLE session_messages_v10 RENAME TO session_messages;
             CREATE INDEX IF NOT EXISTS session_messages_session_role_seq_idx ON session_messages(session_id,role,content_sequence);",
        )?;
        backfill_message_projection(connection)?;
    }
    for next in (version + 1)..=10 {
        connection.execute(
            "INSERT INTO schema_migrations(version,applied_at) VALUES (?,?)",
            params![next, now()],
        )?;
    }
    Ok(())
}

fn backfill_message_projection(connection: &Connection) -> Result<(), PersistenceError> {
    let mut statement = connection.prepare(
        "SELECT session_id,content_sequence,occurred_at,payload_json
         FROM session_content
         WHERE event_type IN ('message.user','message.assistant','message.tool')
         ORDER BY session_id,content_sequence",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    for row in rows {
        let (session_id, sequence, occurred_at, payload_json) = row?;
        let payload: Value = serde_json::from_str(&payload_json)?;
        if let Some(message) = payload.get("message") {
            if let Some(role @ ("user" | "assistant" | "tool")) =
                message.get("role").and_then(Value::as_str)
            {
                let message_id = payload
                    .get("message_id")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("event:{session_id}:{sequence}"));
                connection.execute("INSERT OR IGNORE INTO session_messages (message_id,session_id,turn_id,content_sequence,role,message_json,usage_json,created_at) VALUES (?,?,?,?,?,?,?,?)", params![message_id, session_id, payload.get("turn_id").and_then(Value::as_str), sequence, role, serde_json::to_string(message)?, payload.get("usage").map(serde_json::to_string).transpose()?, occurred_at])?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(store: &Store) -> String {
        let project = store.project("/tmp/suncode-test", "Test").unwrap();
        store
            .create_session(&project.project_id, None, Some("deepseek-v4-flash"))
            .unwrap()
            .session_id
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
                name: "fs_read".into(),
                arguments: json!({"path":"README.md"}),
            }],
            tool_call_id: None,
        };
        let mut tool = Message::text("tool", "{\"content\":\"hello\"}");
        tool.tool_call_id = Some("call-1".into());

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
                "message.tool",
                &json!({"turn_id":"turn-1","tool_call_id":"call-1","message":tool}),
            )
            .unwrap();

        let messages = store.context_messages(&session_id).unwrap();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[1].tool_calls[0].call_id, "call-1");
        assert_eq!(messages[2].role, "tool");
        assert_eq!(messages[2].tool_call_id.as_deref(), Some("call-1"));
    }

    #[test]
    fn persisted_events_do_not_return_ephemeral_deltas() {
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

        let events = store.events(&session_id, 0).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "message.assistant");
        assert!(events[0].content_sequence > 1);
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
                name: "fs_write".into(),
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
                    name: "fs_read".into(),
                    arguments: json!({"path":"README.md"}),
                },
                ToolCall {
                    call_id: "call-2".into(),
                    name: "fs_read".into(),
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
