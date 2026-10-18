use super::*;

impl AsyncAgentSdk {
    pub fn version() -> VersionResult {
        VersionResult {
            version: suncode_core::version(),
        }
    }

    pub async fn open_default(user_id: &str) -> SdkResult<Self> {
        Self::open_with_options(
            user_id,
            SdkOpenOptions {
                environment_source: suncode_config::EnvironmentSource::Process,
                ..SdkOpenOptions::default()
            },
        )
        .await
    }

    pub async fn open_with_options(user_id: &str, options: SdkOpenOptions) -> SdkResult<Self> {
        Self::open_with_options_and_providers(user_id, options, |_| Ok(())).await
    }

    /// Opens the agent after extending the built-in registry with trusted providers.
    pub async fn open_default_with_providers<F>(
        user_id: &str,
        configure_providers: F,
    ) -> SdkResult<Self>
    where
        F: FnOnce(&mut ModelProviderRegistry) -> Result<(), BusinessError>,
    {
        Self::open_with_options_and_providers(
            user_id,
            SdkOpenOptions {
                environment_source: suncode_config::EnvironmentSource::Process,
                ..SdkOpenOptions::default()
            },
            configure_providers,
        )
        .await
    }

    /// Opens the agent with a typed host capability ceiling and trusted providers.
    pub async fn open_with_options_and_providers<F>(
        user_id: &str,
        options: SdkOpenOptions,
        configure_providers: F,
    ) -> SdkResult<Self>
    where
        F: FnOnce(&mut ModelProviderRegistry) -> Result<(), BusinessError>,
    {
        let user_id = validate_user_id(user_id)?;
        let config = Config::load_with_environment(options.environment_source)
            .map_err(BusinessError::invalid)?;
        let lock = AgentLock::acquire(&config.data_dir).map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                BusinessError::new("agent_already_active", error.to_string())
            } else {
                BusinessError::unavailable(format!("agent lock unavailable: {error}"))
            }
        })?;
        let state = build_state(&config, &user_id, options, configure_providers).await?;
        let remote = options.host_capabilities.remote_control.then(|| {
            let remote_host = Arc::new(remote::SdkRemoteHost {
                state: state.clone(),
            });
            Arc::new(suncode_remote::RemoteController::new(
                state.store.clone(),
                remote_host,
            ))
        });
        let sdk = Self {
            _lock: Some(lock),
            data_dir: config.data_dir,
            state,
            remote,
        };
        if let Some(remote) = &sdk.remote {
            if let Err(error) = remote.start_saved() {
                logging::write_business_error("remote", "restore", &error, "phase=open");
            }
        }
        logging::info("agent", "open completed");
        Ok(sdk)
    }

    pub async fn shutdown(self) -> SdkResult<()> {
        logging::info("agent", "shutdown begin");
        if let Some(remote) = &self.remote {
            remote.disconnect()?;
        }
        let result = self.state.agent.shutdown().await;
        match &result {
            Ok(()) => logging::info("agent", "shutdown completed"),
            Err(error) => {
                logging::write_business_error("agent", "shutdown", error, "phase=cleanup")
            }
        }
        result
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

    /// Save using the provider's configured mode (system keychain for a new provider).
    pub fn set_credential(&self, provider: &str, api_key: &str) -> SdkResult<CredentialUpdate> {
        let mode = self
            .state
            .store
            .llm_model_providers(false)?
            .into_iter()
            .find(|record| record.provider_id == provider)
            .ok_or_else(|| BusinessError::invalid("provider is not supported"))?
            .credential_mode;
        self.save_credential(provider, Some(api_key), mode)
    }

    /// `None` keeps the current key while changing its storage mode.
    pub fn save_credential(
        &self,
        provider: &str,
        api_key: Option<&str>,
        mode: CredentialStorageMode,
    ) -> SdkResult<CredentialUpdate> {
        self.state
            .store
            .save_provider_credential(provider, api_key, mode)?;
        self.credential_update(provider)
    }

    pub fn check_credential(&self, provider: &str) -> SdkResult<CredentialUpdate> {
        self.state.store.resolve_provider_credential(provider)?;
        self.credential_update(provider)
    }

    fn credential_update(&self, provider: &str) -> SdkResult<CredentialUpdate> {
        let record = self
            .state
            .store
            .llm_model_providers(false)?
            .into_iter()
            .find(|record| record.provider_id == provider)
            .ok_or_else(|| BusinessError::invalid("provider is not supported"))?;
        let status = credential_states(&self.state.store)?
            .into_iter()
            .find(|state| state.provider == provider)
            .map(|state| state.status)
            .unwrap_or_else(|| "missing".into());
        Ok(CredentialUpdate {
            provider: provider.to_string(),
            configured: status == "configured",
            status,
            mode: record.credential_mode,
            masked_value: record.api_key_mask,
        })
    }

    pub fn remove_credential(&self, provider: &str) -> SdkResult<CredentialUpdate> {
        self.state.store.remove_provider_credential(provider)?;
        self.credential_update(provider)
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
        let adapter = provider_adapter(
            &provider,
            Arc::new(ApiKeyResolver {
                store: self.state.store.clone(),
            }),
            self.state.verify_https_certificates.clone(),
            self.state.use_system_certificates.clone(),
            self.state.certificate_path.clone(),
            self.state.proxy_configuration.clone(),
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

fn validate_user_id(value: &str) -> SdkResult<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(BusinessError::invalid("user_id is required"));
    }
    if value.len() > 256 || value.chars().any(char::is_control) {
        return Err(BusinessError::invalid("user_id is invalid"));
    }
    Ok(value.to_owned())
}
