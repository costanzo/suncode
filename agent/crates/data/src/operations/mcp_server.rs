//! Operations for global MCP server desired configuration.

use crate::{
    domain::{McpServerInput, McpServerRecord, McpTransportConfig},
    store::{business_transaction, lock, now, Store},
};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Integer, Text};
use diesel::sqlite::SqliteConnection;
use serde_json::json;
use std::{collections::BTreeMap, net::IpAddr};
use suncode_common::BusinessError;

#[derive(QueryableByName)]
struct Row {
    #[diesel(sql_type = Text)]
    mcp_server_id: String,
    #[diesel(sql_type = Text)]
    display_name: String,
    #[diesel(sql_type = Text)]
    tool_prefix: String,
    #[diesel(sql_type = Text)]
    transport_type: String,
    #[diesel(sql_type = Text)]
    transport_config_json: String,
    #[diesel(sql_type = Integer)]
    enabled: i32,
    #[diesel(sql_type = Integer)]
    sort_order: i32,
    #[diesel(sql_type = Integer)]
    revision: i32,
    #[diesel(sql_type = Text)]
    created_at: String,
    #[diesel(sql_type = Text)]
    updated_at: String,
}

const SELECT: &str = "SELECT mcp_server_id,display_name,tool_prefix,transport_type,transport_config_json,enabled,sort_order,revision,created_at,updated_at FROM mcp_server";

fn by_id(
    connection: &mut SqliteConnection,
    mcp_server_id: &str,
) -> Result<Option<McpServerRecord>, BusinessError> {
    sql_query(format!("{SELECT} WHERE mcp_server_id=?"))
        .bind::<Text, _>(mcp_server_id)
        .get_result::<Row>(connection)
        .optional()
        .map_err(crate::database_error)?
        .map(to_record)
        .transpose()
}

fn to_record(row: Row) -> Result<McpServerRecord, BusinessError> {
    let transport: McpTransportConfig = serde_json::from_str(&row.transport_config_json)
        .map_err(|_| BusinessError::database("stored MCP transport configuration is invalid"))?;
    if transport.transport_type() != row.transport_type {
        return Err(BusinessError::database(
            "stored MCP transport type does not match its configuration",
        ));
    }
    Ok(McpServerRecord {
        mcp_server_id: row.mcp_server_id,
        display_name: row.display_name,
        tool_prefix: row.tool_prefix,
        transport,
        enabled: row.enabled != 0,
        sort_order: i64::from(row.sort_order),
        revision: u64::try_from(row.revision)
            .map_err(|_| BusinessError::database("stored MCP revision is invalid"))?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

fn validate_input(input: &McpServerInput) -> Result<(), BusinessError> {
    let display_name = input.display_name.trim();
    if display_name.is_empty() || display_name.chars().count() > 80 {
        return Err(BusinessError::invalid(
            "MCP server name must contain 1 to 80 characters",
        ));
    }
    if input.sort_order < 0 || input.sort_order > i64::from(i32::MAX) {
        return Err(BusinessError::invalid("MCP sort order is invalid"));
    }
    if !(1..=120).contains(&input.transport.startup_timeout_seconds()) {
        return Err(BusinessError::invalid(
            "MCP startup timeout must be between 1 and 120 seconds",
        ));
    }
    if !(1..=600).contains(&input.transport.request_timeout_seconds()) {
        return Err(BusinessError::invalid(
            "MCP request timeout must be between 1 and 600 seconds",
        ));
    }
    match &input.transport {
        McpTransportConfig::Stdio {
            version,
            command,
            arguments,
            environment,
            ..
        } => {
            if *version != 1 {
                return Err(BusinessError::invalid(
                    "unsupported MCP transport configuration version",
                ));
            }
            if command.trim().is_empty() || command.chars().count() > 2048 {
                return Err(BusinessError::invalid("MCP executable is invalid"));
            }
            if arguments.len() > 128 || arguments.iter().any(|arg| arg.chars().count() > 8192) {
                return Err(BusinessError::invalid("MCP arguments are too large"));
            }
            validate_entries(environment, "environment")?;
        }
        McpTransportConfig::StreamableHttp {
            version,
            url,
            headers,
            ..
        } => {
            if *version != 1 {
                return Err(BusinessError::invalid(
                    "unsupported MCP transport configuration version",
                ));
            }
            validate_remote_url(url)?;
            validate_entries(headers, "header")?;
        }
    }
    Ok(())
}

fn validate_entries(entries: &BTreeMap<String, String>, label: &str) -> Result<(), BusinessError> {
    if entries.len() > 128 {
        return Err(BusinessError::invalid(format!(
            "MCP {label} entries exceed the limit"
        )));
    }
    for (key, value) in entries {
        if key.trim().is_empty()
            || key.chars().count() > 128
            || key.chars().any(char::is_control)
            || value.chars().count() > 8192
        {
            return Err(BusinessError::invalid(format!(
                "MCP {label} entry is invalid"
            )));
        }
    }
    Ok(())
}

fn validate_remote_url(value: &str) -> Result<(), BusinessError> {
    let url = url::Url::parse(value.trim())
        .map_err(|_| BusinessError::invalid("MCP server URL is invalid"))?;
    if !url.username().is_empty() || url.password().is_some() || url.fragment().is_some() {
        return Err(BusinessError::invalid(
            "MCP server URL must not contain credentials or a fragment",
        ));
    }
    let loopback = url.host_str().is_some_and(|host| {
        host.eq_ignore_ascii_case("localhost")
            || host
                .parse::<IpAddr>()
                .map(|address| address.is_loopback())
                .unwrap_or(false)
    });
    if url.scheme() != "https" && !(url.scheme() == "http" && loopback) {
        return Err(BusinessError::invalid(
            "MCP server URL must use HTTPS; loopback HTTP is allowed",
        ));
    }
    Ok(())
}

fn generated_prefix(display_name: &str) -> String {
    let mut prefix = String::new();
    let mut separator = false;
    for character in display_name.trim().chars() {
        if character.is_ascii_alphanumeric() {
            if separator && !prefix.is_empty() {
                prefix.push('_');
            }
            separator = false;
            prefix.push(character.to_ascii_lowercase());
        } else {
            separator = true;
        }
        if prefix.len() >= 32 {
            break;
        }
    }
    while prefix.ends_with('_') {
        prefix.pop();
    }
    if prefix.is_empty() {
        "server".to_string()
    } else {
        prefix
    }
}

fn revision_conflict(expected: u64, actual: u64) -> BusinessError {
    BusinessError::new(
        "mcp_server_revision_conflict",
        "MCP server was changed by another operation",
    )
    .details(json!({"expectedRevision": expected, "actualRevision": actual}))
}

impl Store {
    pub fn mcp_servers(&self) -> Result<Vec<McpServerRecord>, BusinessError> {
        let mut connection = lock(&self.connection)?;
        sql_query(format!(
            "{SELECT} ORDER BY sort_order,display_name COLLATE NOCASE,mcp_server_id"
        ))
        .load::<Row>(&mut *connection)
        .map_err(crate::database_error)?
        .into_iter()
        .map(to_record)
        .collect()
    }

    pub fn mcp_server_by_id(
        &self,
        mcp_server_id: &str,
    ) -> Result<Option<McpServerRecord>, BusinessError> {
        let mut connection = lock(&self.connection)?;
        by_id(&mut connection, mcp_server_id.trim())
    }

    pub fn create_mcp_server(
        &self,
        mcp_server_id: &str,
        input: &McpServerInput,
    ) -> Result<McpServerRecord, BusinessError> {
        validate_input(input)?;
        let mcp_server_id = mcp_server_id.trim();
        if mcp_server_id.is_empty() || mcp_server_id.chars().count() > 128 {
            return Err(BusinessError::invalid("MCP server id is invalid"));
        }
        let mut connection = lock(&self.connection)?;
        business_transaction(&mut connection, |connection| {
            if let Some(existing) = by_id(connection, mcp_server_id)? {
                if existing.display_name == input.display_name.trim()
                    && existing.transport == input.transport
                    && existing.enabled == input.enabled
                    && existing.sort_order == input.sort_order
                {
                    return Ok(existing);
                }
                return Err(BusinessError::new(
                    "idempotency_conflict",
                    "MCP create id was already used with different input",
                ));
            }
            let display_name = input.display_name.trim();
            let tool_prefix = generated_prefix(display_name);
            let timestamp = now();
            let config = serde_json::to_string(&input.transport)?;
            sql_query("INSERT INTO mcp_server(mcp_server_id,display_name,tool_prefix,transport_type,transport_config_json,enabled,sort_order,revision,created_at,updated_at) VALUES (?,?,?,?,?,?,?,?,?,?)")
                .bind::<Text, _>(mcp_server_id)
                .bind::<Text, _>(display_name)
                .bind::<Text, _>(&tool_prefix)
                .bind::<Text, _>(input.transport.transport_type())
                .bind::<Text, _>(&config)
                .bind::<Integer, _>(input.enabled as i32)
                .bind::<Integer, _>(input.sort_order as i32)
                .bind::<Integer, _>(1)
                .bind::<Text, _>(&timestamp)
                .bind::<Text, _>(&timestamp)
                .execute(connection)
                .map_err(|error| match error {
                    diesel::result::Error::DatabaseError(
                        diesel::result::DatabaseErrorKind::UniqueViolation,
                        _,
                    ) => BusinessError::new(
                        "mcp_server_conflict",
                        "MCP server name or generated tool prefix already exists",
                    ),
                    other => crate::database_error(other),
                })?;
            by_id(connection, mcp_server_id)?
                .ok_or_else(|| BusinessError::database("MCP server was not stored"))
        })
    }

    pub fn update_mcp_server(
        &self,
        mcp_server_id: &str,
        expected_revision: u64,
        input: &McpServerInput,
    ) -> Result<McpServerRecord, BusinessError> {
        validate_input(input)?;
        let mut connection = lock(&self.connection)?;
        business_transaction(&mut connection, |connection| {
            let current = by_id(connection, mcp_server_id)?
                .ok_or_else(|| BusinessError::missing("mcp_server"))?;
            if current.revision != expected_revision {
                if current.display_name == input.display_name.trim()
                    && current.transport == input.transport
                    && current.enabled == input.enabled
                    && current.sort_order == input.sort_order
                {
                    return Ok(current);
                }
                return Err(revision_conflict(expected_revision, current.revision));
            }
            let revision = i32::try_from(current.revision + 1)
                .map_err(|_| BusinessError::invalid("MCP server revision overflow"))?;
            let config = serde_json::to_string(&input.transport)?;
            sql_query("UPDATE mcp_server SET display_name=?,transport_type=?,transport_config_json=?,enabled=?,sort_order=?,revision=?,updated_at=? WHERE mcp_server_id=? AND revision=?")
                .bind::<Text, _>(input.display_name.trim())
                .bind::<Text, _>(input.transport.transport_type())
                .bind::<Text, _>(&config)
                .bind::<Integer, _>(input.enabled as i32)
                .bind::<Integer, _>(input.sort_order as i32)
                .bind::<Integer, _>(revision)
                .bind::<Text, _>(&now())
                .bind::<Text, _>(mcp_server_id)
                .bind::<Integer, _>(expected_revision as i32)
                .execute(connection)
                .map_err(|error| match error {
                    diesel::result::Error::DatabaseError(
                        diesel::result::DatabaseErrorKind::UniqueViolation,
                        _,
                    ) => BusinessError::new(
                        "mcp_server_conflict",
                        "MCP server name already exists",
                    ),
                    other => crate::database_error(other),
                })?;
            by_id(connection, mcp_server_id)?
                .ok_or_else(|| BusinessError::database("MCP server update was not stored"))
        })
    }

    pub fn set_mcp_server_enabled(
        &self,
        mcp_server_id: &str,
        expected_revision: u64,
        enabled: bool,
    ) -> Result<McpServerRecord, BusinessError> {
        let current = self
            .mcp_server_by_id(mcp_server_id)?
            .ok_or_else(|| BusinessError::missing("mcp_server"))?;
        let input = McpServerInput {
            display_name: current.display_name,
            transport: current.transport,
            enabled,
            sort_order: current.sort_order,
        };
        self.update_mcp_server(mcp_server_id, expected_revision, &input)
    }

    pub fn delete_mcp_server(
        &self,
        mcp_server_id: &str,
        expected_revision: u64,
    ) -> Result<bool, BusinessError> {
        let mut connection = lock(&self.connection)?;
        business_transaction(&mut connection, |connection| {
            let Some(current) = by_id(connection, mcp_server_id)? else {
                return Ok(false);
            };
            if current.revision != expected_revision {
                return Err(revision_conflict(expected_revision, current.revision));
            }
            let changed = sql_query("DELETE FROM mcp_server WHERE mcp_server_id=? AND revision=?")
                .bind::<Text, _>(mcp_server_id)
                .bind::<Integer, _>(expected_revision as i32)
                .execute(connection)
                .map_err(crate::database_error)?;
            Ok(changed == 1)
        })
    }
}
