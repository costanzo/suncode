use super::*;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use suncode_data::{McpServerInput, McpServerRecord, McpTransportConfig};

impl AgentSdk {
    pub fn list_mcp_servers(&self, project_id: Option<&str>) -> SdkResult<McpServersResult> {
        self.validate_mcp_project(project_id)?;
        let servers = self
            .state
            .store
            .mcp_servers()?
            .into_iter()
            .map(|server| self.mcp_server_dto(project_id, server))
            .collect::<SdkResult<Vec<_>>>()?;
        Ok(McpServersResult { servers })
    }

    pub fn create_mcp_server(
        &self,
        project_id: Option<&str>,
        idempotency_key: &str,
        request: McpServerWriteRequest,
    ) -> SdkResult<McpServerDto> {
        self.validate_mcp_project(project_id)?;
        require_idempotency_key(idempotency_key)?;
        let server_id = deterministic_server_id(idempotency_key);
        let input = write_input(request, None)?;
        let server = self.state.store.create_mcp_server(&server_id, &input)?;
        self.runtime
            .block_on(self.state.agent.reconcile_mcp_server(&server_id))?;
        self.mcp_server_dto(project_id, server)
    }

    pub fn update_mcp_server(
        &self,
        project_id: Option<&str>,
        server_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
        request: McpServerWriteRequest,
    ) -> SdkResult<McpServerDto> {
        self.validate_mcp_project(project_id)?;
        require_idempotency_key(idempotency_key)?;
        let current = self
            .state
            .store
            .mcp_server_by_id(server_id)?
            .ok_or_else(|| BusinessError::missing("mcp_server"))?;
        let input = write_input(request, Some(&current.transport))?;
        let server = self
            .state
            .store
            .update_mcp_server(server_id, expected_revision, &input)?;
        self.runtime
            .block_on(self.state.agent.reconcile_mcp_server(server_id))?;
        self.mcp_server_dto(project_id, server)
    }

    pub fn set_mcp_server_enabled(
        &self,
        project_id: Option<&str>,
        server_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
        enabled: bool,
    ) -> SdkResult<McpServerDto> {
        self.validate_mcp_project(project_id)?;
        require_idempotency_key(idempotency_key)?;
        let server =
            self.state
                .store
                .set_mcp_server_enabled(server_id, expected_revision, enabled)?;
        self.runtime
            .block_on(self.state.agent.reconcile_mcp_server(server_id))?;
        self.mcp_server_dto(project_id, server)
    }

    pub fn delete_mcp_server(
        &self,
        server_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> SdkResult<McpServerDeleteResult> {
        require_idempotency_key(idempotency_key)?;
        let removed = self
            .state
            .store
            .delete_mcp_server(server_id, expected_revision)?;
        self.runtime
            .block_on(self.state.agent.reconcile_mcp_server(server_id))?;
        Ok(McpServerDeleteResult {
            mcp_server_id: server_id.to_string(),
            removed,
        })
    }

    pub fn retry_mcp_server(&self, project_id: &str, server_id: &str) -> SdkResult<McpServerDto> {
        self.runtime
            .block_on(self.state.agent.retry_mcp_server(project_id, server_id))?;
        let server = self
            .state
            .store
            .mcp_server_by_id(server_id)?
            .ok_or_else(|| BusinessError::missing("mcp_server"))?;
        self.mcp_server_dto(Some(project_id), server)
    }

    fn mcp_server_dto(
        &self,
        project_id: Option<&str>,
        server: McpServerRecord,
    ) -> SdkResult<McpServerDto> {
        let runtime = self
            .runtime
            .block_on(self.state.agent.mcp_runtime_status(project_id, &server));
        let (
            transport_type,
            command,
            arguments,
            working_directory,
            url,
            environment_keys,
            header_keys,
            startup_timeout_seconds,
            request_timeout_seconds,
        ) = match &server.transport {
            McpTransportConfig::Stdio {
                command,
                arguments,
                working_directory,
                environment,
                startup_timeout_seconds,
                request_timeout_seconds,
                ..
            } => (
                "stdio".to_string(),
                Some(command.clone()),
                arguments.clone(),
                Some(working_directory.clone()),
                None,
                environment.keys().cloned().collect(),
                Vec::new(),
                *startup_timeout_seconds,
                *request_timeout_seconds,
            ),
            McpTransportConfig::StreamableHttp {
                url,
                headers,
                startup_timeout_seconds,
                request_timeout_seconds,
                ..
            } => (
                "streamable_http".to_string(),
                None,
                Vec::new(),
                None,
                Some(url.clone()),
                Vec::new(),
                headers.keys().cloned().collect(),
                *startup_timeout_seconds,
                *request_timeout_seconds,
            ),
        };
        Ok(McpServerDto {
            mcp_server_id: server.mcp_server_id,
            display_name: server.display_name,
            tool_prefix: server.tool_prefix,
            transport_type,
            command,
            arguments,
            working_directory,
            url,
            environment_keys,
            header_keys,
            startup_timeout_seconds,
            request_timeout_seconds,
            enabled: server.enabled,
            sort_order: server.sort_order,
            revision: server.revision,
            runtime_status: runtime.state,
            tool_count: runtime.tool_count,
            error: runtime.error,
            created_at: server.created_at,
            updated_at: server.updated_at,
        })
    }

    fn validate_mcp_project(&self, project_id: Option<&str>) -> SdkResult<()> {
        if let Some(project_id) = project_id {
            if self.state.store.project_by_id(project_id)?.is_none() {
                return Err(BusinessError::missing("project"));
            }
        }
        Ok(())
    }
}

fn write_input(
    request: McpServerWriteRequest,
    existing: Option<&McpTransportConfig>,
) -> SdkResult<McpServerInput> {
    let transport = match request.transport {
        McpTransportRequest::Stdio {
            command,
            arguments,
            working_directory,
            environment,
            startup_timeout_seconds,
            request_timeout_seconds,
        } => {
            let current = match existing {
                Some(McpTransportConfig::Stdio { environment, .. }) => environment.clone(),
                _ => BTreeMap::new(),
            };
            McpTransportConfig::Stdio {
                version: 1,
                command,
                arguments,
                working_directory,
                environment: apply_secret_changes(current, environment)?,
                startup_timeout_seconds,
                request_timeout_seconds,
            }
        }
        McpTransportRequest::StreamableHttp {
            url,
            headers,
            startup_timeout_seconds,
            request_timeout_seconds,
        } => {
            let current = match existing {
                Some(McpTransportConfig::StreamableHttp { headers, .. }) => headers.clone(),
                _ => BTreeMap::new(),
            };
            McpTransportConfig::StreamableHttp {
                version: 1,
                url,
                headers: apply_secret_changes(current, headers)?,
                startup_timeout_seconds,
                request_timeout_seconds,
            }
        }
    };
    Ok(McpServerInput {
        display_name: request.display_name,
        transport,
        enabled: request.enabled,
        sort_order: request.sort_order,
    })
}

fn apply_secret_changes(
    mut values: BTreeMap<String, String>,
    changes: Option<McpSecretChanges>,
) -> SdkResult<BTreeMap<String, String>> {
    let Some(changes) = changes else {
        return Ok(values);
    };
    for key in changes.remove {
        values.remove(key.trim());
    }
    for (key, value) in changes.set {
        let key = key.trim();
        if key.is_empty() || value.is_empty() {
            return Err(BusinessError::invalid(
                "MCP secret keys and values must not be empty",
            ));
        }
        values.insert(key.to_string(), value);
    }
    Ok(values)
}

fn deterministic_server_id(idempotency_key: &str) -> String {
    let digest = Sha256::digest(idempotency_key.as_bytes());
    format!(
        "mcp_{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        digest[0], digest[1], digest[2], digest[3], digest[4], digest[5], digest[6], digest[7],
        digest[8], digest[9], digest[10], digest[11], digest[12], digest[13], digest[14], digest[15]
    )
}

fn require_idempotency_key(value: &str) -> SdkResult<()> {
    if value.trim().is_empty() || value.chars().count() > 256 {
        Err(BusinessError::invalid("idempotency_key is required"))
    } else {
        Ok(())
    }
}
