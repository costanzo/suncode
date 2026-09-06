//! Operations for scoped `configuration`.

use crate::{
    domain::SettingRecord,
    store::{lock, now, Store},
};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Nullable, Text};
use serde_json::Value;
use suncode_common::BusinessError;

#[derive(QueryableByName)]
struct Row {
    #[diesel(sql_type = Text)]
    scope: String,
    #[diesel(sql_type = Text)]
    scope_id: String,
    #[diesel(sql_type = Text)]
    key: String,
    #[diesel(sql_type = Text)]
    value_json: String,
}

#[derive(QueryableByName)]
struct ValueRow {
    #[diesel(sql_type = Text)]
    value: String,
}

impl Store {
    pub fn settings(
        &self,
        project_id: Option<&str>,
        session_id: Option<&str>,
    ) -> Result<Vec<SettingRecord>, BusinessError> {
        let mut connection = lock(&self.connection)?;
        let rows = sql_query("SELECT scope,COALESCE(project_id,session_id,'global') AS scope_id,key,value_json FROM configuration WHERE scope='global' OR (scope='project' AND project_id=?) OR (scope='session' AND session_id=?) ORDER BY CASE scope WHEN 'global' THEN 0 WHEN 'project' THEN 1 ELSE 2 END,key")
            .bind::<Nullable<Text>, _>(project_id).bind::<Nullable<Text>, _>(session_id)
            .load::<Row>(&mut *connection).map_err(crate::database_error)?;
        let mut settings = Vec::new();
        for row in rows {
            settings.retain(|value: &SettingRecord| value.key != row.key);
            settings.push(SettingRecord {
                key: row.key,
                value: serde_json::from_str(&row.value_json)?,
                scope: row.scope,
                scope_id: row.scope_id,
            });
        }
        Ok(settings)
    }

    pub fn session_full_control(&self, session_id: &str) -> Result<bool, BusinessError> {
        let mut connection = lock(&self.connection)?;
        match sql_query("SELECT value_json AS value FROM configuration WHERE scope='session' AND session_id=? AND key='full_control'")
            .bind::<Text, _>(session_id).get_result::<ValueRow>(&mut *connection).optional().map_err(crate::database_error)? {
            None => Ok(false),
            Some(value) => serde_json::from_str::<Value>(&value.value)?.as_bool()
                .ok_or_else(|| BusinessError::invalid("session full_control configuration must be a boolean")),
        }
    }

    pub fn project_default_model(&self, project_id: &str) -> Result<Option<String>, BusinessError> {
        let value = self
            .settings(Some(project_id), None)?
            .into_iter()
            .find(|record| record.key == "default_model")
            .map(|record| record.value);
        let Some(value) = value else { return Ok(None) };
        let model = value
            .as_str()
            .ok_or_else(|| BusinessError::invalid("project default_model must be a string"))?;
        if model.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(model.trim().into()))
        }
    }

    pub fn project_tool_call_limit(&self, project_id: &str) -> Result<Option<u32>, BusinessError> {
        let mut connection = lock(&self.connection)?;
        let Some(value) = sql_query("SELECT value_json AS value FROM configuration WHERE scope='project' AND project_id=? AND key='tool_call_limit'")
            .bind::<Text, _>(project_id).get_result::<ValueRow>(&mut *connection).optional().map_err(crate::database_error)? else { return Ok(None) };
        let limit = serde_json::from_str::<Value>(&value.value)?
            .as_u64()
            .ok_or_else(|| {
                BusinessError::invalid(
                    "project tool_call_limit must be an integer between 1 and 256",
                )
            })?;
        if !(1..=256).contains(&limit) {
            return Err(BusinessError::invalid(
                "project tool_call_limit must be an integer between 1 and 256",
            ));
        }
        Ok(Some(limit as u32))
    }

    pub fn set_setting(
        &self,
        scope: &str,
        scope_id: &str,
        key: &str,
        value: &Value,
    ) -> Result<(), BusinessError> {
        if !matches!(scope, "global" | "project" | "session") || key.trim().is_empty() {
            return Err(BusinessError::invalid(
                "setting scope, scope id, and key are required",
            ));
        }
        let mut connection = lock(&self.connection)?;
        let encoded = serde_json::to_string(value)?;
        let timestamp = now();
        match scope {
            "global" => sql_query("INSERT INTO configuration(scope,key,value_json,updated_at) VALUES ('global',?,?,?) ON CONFLICT(key) WHERE scope='global' DO UPDATE SET value_json=excluded.value_json,updated_at=excluded.updated_at")
                .bind::<Text, _>(key.trim()).bind::<Text, _>(&encoded).bind::<Text, _>(&timestamp).execute(&mut *connection),
            "project" => sql_query("INSERT INTO configuration(scope,project_id,key,value_json,updated_at) VALUES ('project',?,?,?,?) ON CONFLICT(project_id,key) WHERE scope='project' DO UPDATE SET value_json=excluded.value_json,updated_at=excluded.updated_at")
                .bind::<Text, _>(scope_id).bind::<Text, _>(key.trim()).bind::<Text, _>(&encoded).bind::<Text, _>(&timestamp).execute(&mut *connection),
            "session" => sql_query("INSERT INTO configuration(scope,session_id,key,value_json,updated_at) VALUES ('session',?,?,?,?) ON CONFLICT(session_id,key) WHERE scope='session' DO UPDATE SET value_json=excluded.value_json,updated_at=excluded.updated_at")
                .bind::<Text, _>(scope_id).bind::<Text, _>(key.trim()).bind::<Text, _>(&encoded).bind::<Text, _>(&timestamp).execute(&mut *connection),
            _ => unreachable!(),
        }.map_err(crate::database_error)?;
        Ok(())
    }
}
