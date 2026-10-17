//! Agent integration for MCP. The runtime lives in `suncode-mcp`; this module
//! supplies it with server configuration and network settings from the store
//! and exposes the public `Agent` API.

use super::*;
use suncode_data::{McpServerRecord, McpTransportConfig, McpWorkingDirectory};
use suncode_mcp::{ServerSource, ServerSpec, ServerTransport, TlsConfig, WorkingDirectory};

pub use suncode_mcp::{McpLoadProgress, McpRuntimeState, McpRuntimeStatus};

/// Desired MCP configuration read from the store.
pub(super) struct StoreServerSource {
    pub(super) store: Store,
    pub(super) application_data: PathBuf,
}

impl ServerSource for StoreServerSource {
    fn servers(&self) -> Result<Vec<ServerSpec>, BusinessError> {
        Ok(self.store.mcp_servers()?.iter().map(server_spec).collect())
    }

    fn server(&self, server_id: &str) -> Result<Option<ServerSpec>, BusinessError> {
        Ok(self
            .store
            .mcp_server_by_id(server_id)?
            .as_ref()
            .map(server_spec))
    }

    fn application_data(&self) -> PathBuf {
        self.application_data.clone()
    }

    fn network(&self) -> Result<TlsConfig, BusinessError> {
        let settings = self.store.global_settings()?;
        Ok(TlsConfig {
            verify_certificates: settings.tls.verify_https_certificates,
            use_system_certificates: settings.tls.use_system_certificates,
            certificate_path: settings.tls.certificate_path,
            proxy: settings.proxy,
        })
    }
}

pub(super) fn server_spec(record: &McpServerRecord) -> ServerSpec {
    ServerSpec {
        server_id: record.mcp_server_id.clone(),
        display_name: record.display_name.clone(),
        tool_prefix: record.tool_prefix.clone(),
        enabled: record.enabled,
        revision: record.revision,
        transport: match &record.transport {
            McpTransportConfig::Stdio {
                command,
                arguments,
                working_directory,
                environment,
                startup_timeout_seconds,
                request_timeout_seconds,
                ..
            } => ServerTransport::Stdio {
                command: command.clone(),
                arguments: arguments.clone(),
                working_directory: match working_directory {
                    McpWorkingDirectory::Project => WorkingDirectory::Project,
                    McpWorkingDirectory::ApplicationData => WorkingDirectory::ApplicationData,
                },
                environment: environment.clone(),
                startup_timeout: Duration::from_secs(*startup_timeout_seconds),
                request_timeout: Duration::from_secs(*request_timeout_seconds),
            },
            McpTransportConfig::StreamableHttp {
                url,
                headers,
                startup_timeout_seconds,
                request_timeout_seconds,
                ..
            } => ServerTransport::StreamableHttp {
                url: url.clone(),
                headers: headers.clone(),
                startup_timeout: Duration::from_secs(*startup_timeout_seconds),
                request_timeout: Duration::from_secs(*request_timeout_seconds),
            },
        },
    }
}

impl AgentRuntime {
    pub async fn activate_mcp_project(
        &self,
        project_id: &str,
        project_root: &Path,
    ) -> Result<(), BusinessError> {
        self.mcp.activate_project(project_id, project_root).await
    }

    pub async fn reconcile_mcp_server(&self, server_id: &str) -> Result<(), BusinessError> {
        self.mcp.reconcile_server(server_id).await
    }

    pub async fn reconcile_mcp_network_configuration(&self) -> Result<(), BusinessError> {
        self.mcp.reconcile_remote_servers().await
    }

    pub async fn retry_mcp_server(
        &self,
        project_id: &str,
        server_id: &str,
    ) -> Result<(), BusinessError> {
        self.mcp.retry(project_id, server_id).await
    }

    pub async fn mcp_runtime_status(
        &self,
        project_id: Option<&str>,
        server: &McpServerRecord,
    ) -> McpRuntimeStatus {
        self.mcp.status(project_id, &server_spec(server)).await
    }

    pub async fn mcp_load_progress(&self, project_id: &str) -> McpLoadProgress {
        self.mcp.progress(project_id).await
    }
}
