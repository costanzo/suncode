use super::*;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use suncode_data::{LanguageServerConfig, LanguageServerInput, LanguageServerRecord};

impl AgentSdk {
    pub fn start_language_server_project(
        &self,
        project_id: &str,
    ) -> SdkResult<LanguageServerProjectResult> {
        let project = self.project_for_user(project_id)?;
        self.runtime.block_on(
            self.state
                .agent
                .activate_language_server_project(project_id, Path::new(&project.canonical_root)),
        )?;
        Ok(LanguageServerProjectResult {
            project_id: project_id.to_string(),
            started: true,
        })
    }

    pub fn list_language_servers(
        &self,
        project_id: Option<&str>,
    ) -> SdkResult<LanguageServersResult> {
        self.validate_language_server_project(project_id)?;
        let servers = self
            .state
            .store
            .language_servers()?
            .into_iter()
            .map(|server| self.language_server_dto(project_id, server))
            .collect::<SdkResult<Vec<_>>>()?;
        Ok(LanguageServersResult { servers })
    }

    pub fn create_language_server(
        &self,
        project_id: Option<&str>,
        idempotency_key: &str,
        request: LanguageServerWriteRequest,
    ) -> SdkResult<LanguageServerDto> {
        self.validate_language_server_project(project_id)?;
        require_idempotency_key(idempotency_key)?;
        let language_server_id = deterministic_language_server_id(idempotency_key);
        let input = language_server_input(request, None)?;
        let server = self
            .state
            .store
            .create_language_server(&language_server_id, &input)?;
        self.runtime.block_on(
            self.state
                .agent
                .reconcile_language_server(&language_server_id),
        )?;
        self.language_server_dto(project_id, server)
    }

    pub fn update_language_server(
        &self,
        project_id: Option<&str>,
        language_server_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
        request: LanguageServerWriteRequest,
    ) -> SdkResult<LanguageServerDto> {
        self.validate_language_server_project(project_id)?;
        require_idempotency_key(idempotency_key)?;
        let current = self
            .state
            .store
            .language_server_by_id(language_server_id)?
            .ok_or_else(|| BusinessError::missing("language_server"))?;
        let input = language_server_input(request, Some(&current.config))?;
        let server = self.state.store.update_language_server(
            language_server_id,
            expected_revision,
            &input,
        )?;
        self.runtime.block_on(
            self.state
                .agent
                .reconcile_language_server(language_server_id),
        )?;
        self.language_server_dto(project_id, server)
    }

    pub fn set_language_server_enabled(
        &self,
        project_id: Option<&str>,
        language_server_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
        enabled: bool,
    ) -> SdkResult<LanguageServerDto> {
        self.validate_language_server_project(project_id)?;
        require_idempotency_key(idempotency_key)?;
        let server = self.state.store.set_language_server_enabled(
            language_server_id,
            expected_revision,
            enabled,
        )?;
        self.runtime.block_on(
            self.state
                .agent
                .reconcile_language_server(language_server_id),
        )?;
        self.language_server_dto(project_id, server)
    }

    pub fn delete_language_server(
        &self,
        language_server_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> SdkResult<LanguageServerDeleteResult> {
        require_idempotency_key(idempotency_key)?;
        let removed = self
            .state
            .store
            .delete_language_server(language_server_id, expected_revision)?;
        self.runtime.block_on(
            self.state
                .agent
                .reconcile_language_server(language_server_id),
        )?;
        Ok(LanguageServerDeleteResult {
            language_server_id: language_server_id.to_string(),
            removed,
        })
    }

    pub fn retry_language_server(
        &self,
        project_id: &str,
        language_server_id: &str,
    ) -> SdkResult<LanguageServerDto> {
        self.project_for_user(project_id)?;
        self.runtime.block_on(
            self.state
                .agent
                .retry_language_server(project_id, language_server_id),
        )?;
        let server = self
            .state
            .store
            .language_server_by_id(language_server_id)?
            .ok_or_else(|| BusinessError::missing("language_server"))?;
        self.language_server_dto(Some(project_id), server)
    }

    fn language_server_dto(
        &self,
        project_id: Option<&str>,
        server: LanguageServerRecord,
    ) -> SdkResult<LanguageServerDto> {
        let runtime = self.runtime.block_on(
            self.state
                .agent
                .language_server_runtime_status(project_id, &server),
        );
        Ok(LanguageServerDto {
            language_server_id: server.language_server_id,
            display_name: server.display_name,
            command: server.config.command,
            arguments: server.config.arguments,
            language_ids: server.config.language_ids,
            root_markers: server.config.root_markers,
            initialization_options: server.config.initialization_options,
            environment_keys: server.config.environment.keys().cloned().collect(),
            startup_timeout_seconds: server.config.startup_timeout_seconds,
            request_timeout_seconds: server.config.request_timeout_seconds,
            enabled: server.enabled,
            sort_order: server.sort_order,
            revision: server.revision,
            runtime_status: runtime.state,
            capability_count: runtime.capability_count,
            error: runtime.error,
            created_at: server.created_at,
            updated_at: server.updated_at,
        })
    }

    fn validate_language_server_project(&self, project_id: Option<&str>) -> SdkResult<()> {
        if let Some(project_id) = project_id {
            self.project_for_user(project_id)?;
        }
        Ok(())
    }
}

fn language_server_input(
    request: LanguageServerWriteRequest,
    existing: Option<&LanguageServerConfig>,
) -> SdkResult<LanguageServerInput> {
    let environment = apply_environment_changes(
        existing
            .map(|config| config.environment.clone())
            .unwrap_or_default(),
        request.environment,
    )?;
    Ok(LanguageServerInput {
        display_name: request.display_name,
        config: LanguageServerConfig {
            version: 1,
            command: request.command,
            arguments: request.arguments,
            language_ids: request.language_ids,
            root_markers: request.root_markers,
            initialization_options: request.initialization_options,
            environment,
            startup_timeout_seconds: request.startup_timeout_seconds,
            request_timeout_seconds: request.request_timeout_seconds,
        },
        enabled: request.enabled,
        sort_order: request.sort_order,
    })
}

fn apply_environment_changes(
    mut current: BTreeMap<String, String>,
    changes: Option<LanguageServerEnvironmentChanges>,
) -> SdkResult<BTreeMap<String, String>> {
    let Some(changes) = changes else {
        return Ok(current);
    };
    let mut seen = HashSet::new();
    for key in changes.set.keys().chain(changes.remove.iter()) {
        let normalized = key.to_ascii_uppercase();
        if !seen.insert(normalized) {
            return Err(BusinessError::invalid(
                "Language server environment changes contain duplicate keys",
            ));
        }
    }
    for key in changes.remove {
        if let Some(existing) = current
            .keys()
            .find(|existing| existing.eq_ignore_ascii_case(&key))
            .cloned()
        {
            current.remove(&existing);
        }
    }
    for (key, value) in changes.set {
        if let Some(existing) = current
            .keys()
            .find(|existing| existing.eq_ignore_ascii_case(&key))
            .cloned()
        {
            current.remove(&existing);
        }
        current.insert(key, value);
    }
    Ok(current)
}

fn deterministic_language_server_id(idempotency_key: &str) -> String {
    let digest = Sha256::digest(format!("language-server:{idempotency_key}").as_bytes());
    format!(
        "lsp_{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        digest[0], digest[1], digest[2], digest[3], digest[4], digest[5], digest[6], digest[7]
    )
}

fn require_idempotency_key(value: &str) -> SdkResult<()> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 256 {
        return Err(BusinessError::invalid(
            "idempotency key is required and must not exceed 256 characters",
        ));
    }
    Ok(())
}
