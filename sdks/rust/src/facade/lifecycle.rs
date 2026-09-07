use super::*;

impl AgentSdk {
    pub fn open_default() -> SdkResult<Self> {
        Self::open_default_with_providers(|_| Ok(()))
    }

    /// Opens the agent after extending the built-in registry with trusted providers.
    pub fn open_default_with_providers<F>(configure_providers: F) -> SdkResult<Self>
    where
        F: FnOnce(&mut ModelProviderRegistry) -> Result<(), BusinessError>,
    {
        let config = Config::load().map_err(BusinessError::invalid)?;
        let lock = AgentLock::acquire(&config.data_dir).map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                BusinessError::new("agent_already_active", error.to_string())
            } else {
                BusinessError::unavailable(format!("agent lock unavailable: {error}"))
            }
        })?;
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("suncode-sdk")
            .build()
            .map_err(|error| {
                BusinessError::unavailable(format!("tokio runtime unavailable: {error}"))
            })?;
        let state = runtime.block_on(build_state(&config, configure_providers))?;
        logging::write(Level::Info, "agent", "open completed");
        Ok(Self {
            _lock: Some(lock),
            data_dir: config.data_dir,
            runtime,
            state,
        })
    }

    pub fn health(&self) -> SdkResult<HealthResult> {
        Ok(HealthResult {
            ok: true,
            agent: "ready",
            database: self.state.store.health()?,
        })
    }

    pub fn diagnostics(&self) -> SdkResult<DiagnosticsResult> {
        Ok(DiagnosticsResult {
            health: self.health()?,
            recovery: RecoveryStatus {
                status: "ready",
                pending_operations: 0,
            },
            credentials: credential_states(&self.state.store)?,
            active_project_id: self
                .state
                .active_project
                .lock()
                .ok()
                .and_then(|value| value.clone()),
        })
    }

    pub fn list_models(&self) -> SdkResult<ModelsResult> {
        let mut models = self.state.providers.models();
        let credentials = credential_states(&self.state.store)?;
        for model in &mut models {
            model.availability = if credentials
                .iter()
                .any(|state| state.provider == model.provider && state.configured)
            {
                "configured".into()
            } else {
                "unconfigured".into()
            };
        }
        Ok(ModelsResult { models })
    }

    pub fn list_credentials(&self) -> SdkResult<CredentialsResult> {
        Ok(CredentialsResult {
            credentials: credential_states(&self.state.store)?,
        })
    }

    pub fn set_credential(&self, provider: &str, api_key: &str) -> SdkResult<CredentialUpdate> {
        if !provider_exists(&self.state.store, provider)? {
            return Err(BusinessError::invalid("provider is not supported"));
        }
        let provider = provider.to_string();
        self.state
            .store
            .set_llm_provider_api_key(&provider, api_key)
            .map_err(|error| BusinessError::new("credential_unavailable", error.to_string()))?;
        Ok(CredentialUpdate {
            provider,
            configured: true,
        })
    }

    pub fn remove_credential(&self, provider: &str) -> SdkResult<CredentialUpdate> {
        if !provider_exists(&self.state.store, provider)? {
            return Err(BusinessError::invalid("provider is not supported"));
        }
        let provider = provider.to_string();
        self.state
            .store
            .delete_llm_provider_api_key(&provider)
            .map_err(|error| BusinessError::new("credential_unavailable", error.to_string()))?;
        Ok(CredentialUpdate {
            provider,
            configured: false,
        })
    }

    pub fn set_provider_endpoint(
        &self,
        provider_id: &str,
        endpoint: &str,
    ) -> SdkResult<ProviderEndpointUpdate> {
        let provider_id = provider_id.trim();
        let endpoint = normalize_provider_endpoint(endpoint)?;
        let mut provider = self
            .state
            .store
            .llm_model_providers(false)?
            .into_iter()
            .find(|provider| provider.provider_id == provider_id)
            .ok_or_else(|| BusinessError::new("provider_not_found", "provider is not supported"))?;
        provider.endpoint = endpoint.clone();
        let models = provider_models(&self.state.store, &provider)?;
        if models.is_empty() {
            return Err(BusinessError::new(
                "provider_registration_failed",
                "provider has no enabled models",
            ));
        }
        let adapter = openai_provider(
            &provider,
            Arc::new(SqliteApiKeyResolver {
                store: self.state.store.clone(),
            }),
            self.state.verify_https_certificates.clone(),
            self.state.use_system_certificates.clone(),
            self.state.certificate_path.clone(),
        )?;
        self.state
            .store
            .upsert_llm_model_provider(LlmModelProviderInput {
                provider_id: &provider.provider_id,
                display_name: &provider.display_name,
                endpoint: &provider.endpoint,
                default_endpoint: &provider.default_endpoint,
                adapter_type: &provider.adapter_type,
                enabled: provider.enabled,
                sort_order: provider.sort_order,
            })?;
        self.state
            .providers
            .replace(provider.provider_id.clone(), adapter, models)?;
        Ok(ProviderEndpointUpdate {
            provider_id: provider.provider_id,
            endpoint,
        })
    }
}
