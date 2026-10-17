//! Agent integration for language servers. The runtime lives in
//! `suncode-lsp`; this module supplies it with server configuration from the
//! store and exposes the public `Agent` API.

use super::*;
use suncode_data::LanguageServerRecord;
use suncode_lsp::{ServerSource, ServerSpec};

pub use suncode_lsp::{LanguageServerRuntimeState, LanguageServerRuntimeStatus};

/// Desired language-server configuration read from the store.
pub(super) struct StoreServerSource {
    pub(super) store: Store,
}

impl ServerSource for StoreServerSource {
    fn servers(&self) -> Result<Vec<ServerSpec>, BusinessError> {
        Ok(self
            .store
            .language_servers()?
            .iter()
            .map(server_spec)
            .collect())
    }

    fn server(&self, server_id: &str) -> Result<Option<ServerSpec>, BusinessError> {
        Ok(self
            .store
            .language_server_by_id(server_id)?
            .as_ref()
            .map(server_spec))
    }
}

fn server_spec(record: &LanguageServerRecord) -> ServerSpec {
    let config = &record.config;
    ServerSpec {
        server_id: record.language_server_id.clone(),
        enabled: record.enabled,
        revision: record.revision,
        command: config.command.clone(),
        arguments: config.arguments.clone(),
        language_ids: config.language_ids.clone(),
        root_markers: config.root_markers.clone(),
        initialization_options: config.initialization_options.clone(),
        environment: config.environment.clone(),
        startup_timeout: Duration::from_secs(config.startup_timeout_seconds),
        request_timeout: Duration::from_secs(config.request_timeout_seconds),
    }
}

impl AgentRuntime {
    pub async fn activate_language_server_project(
        &self,
        project_id: &str,
        project_root: &Path,
    ) -> Result<(), BusinessError> {
        self.lsp.activate_project(project_id, project_root).await
    }

    pub async fn reconcile_language_server(
        &self,
        language_server_id: &str,
    ) -> Result<(), BusinessError> {
        self.lsp.reconcile_all(language_server_id).await
    }

    pub async fn retry_language_server(
        &self,
        project_id: &str,
        language_server_id: &str,
    ) -> Result<(), BusinessError> {
        self.lsp.retry(project_id, language_server_id).await
    }

    pub async fn language_server_runtime_status(
        &self,
        project_id: Option<&str>,
        server: &LanguageServerRecord,
    ) -> LanguageServerRuntimeStatus {
        self.lsp.status(project_id, &server_spec(server)).await
    }
}
