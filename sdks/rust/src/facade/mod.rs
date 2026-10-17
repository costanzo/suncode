use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, RwLock,
    },
};
use suncode_common::logging;
use suncode_common::{BusinessError, HttpProxyConfiguration, HttpProxyMode};
use suncode_config::Config;
use suncode_config::{LogSettings, SettingScope};
use suncode_core::{
    agent::{AgentHostCapabilities, AgentRuntime},
    AgentLock, AttentionEventHub, CheckpointItemRestoredPayload, CheckpointRestoreFailedPayload,
    CheckpointRestoredPayload, EventPayload, SessionEventHub,
};
use suncode_data::{LlmModelProviderInput, LlmModelProviderRecord, Store};
use suncode_llm::{
    AnthropicProvider, ModelCapabilities, ModelDescriptor, ModelLimits, ModelProviderRegistry,
    OpenAiCompatibleProvider,
};

use crate::types::*;

mod agents;
mod attention;
pub mod blocking;
mod browser;
pub use browser::BrowserHostCallback;
mod checkpoints;
mod computer;
mod language_servers;
mod lifecycle;
mod mcp;
mod preview;
mod project_sessions;
mod projects;
mod remote;
mod sessions;
mod settings;
mod skills;
mod subscriptions;
#[cfg(test)]
mod tests;
mod turns;

pub use blocking::AgentSdk;
pub use subscriptions::{
    AttentionEventStream, AttentionEventStreamControl, SessionEventStream,
    SessionEventStreamControl, SessionWatch, SubscriptionError,
};
pub use suncode_remote::{RemoteServerConfiguration, RemoteServerStatus};

#[derive(Clone)]
pub(crate) struct AgentState {
    store: Store,
    user_id: String,
    operations: Arc<suncode_tool::Operations>,
    active_project: Arc<Mutex<Option<String>>>,
    events: SessionEventHub,
    attention_events: AttentionEventHub,
    verify_https_certificates: Arc<AtomicBool>,
    use_system_certificates: Arc<AtomicBool>,
    certificate_path: Arc<RwLock<Option<PathBuf>>>,
    proxy_configuration: Arc<RwLock<HttpProxyConfiguration>>,
    agent: AgentRuntime,
    providers: Arc<ModelProviderRegistry>,
    credential_overrides: std::collections::BTreeMap<String, String>,
    host_capabilities: SdkHostCapabilities,
    log_console_output: SdkLogConsoleOutput,
}

impl AsyncAgentSdk {
    pub(crate) fn project_for_user(
        &self,
        project_id: &str,
    ) -> SdkResult<suncode_data::ProjectRecord> {
        self.state
            .store
            .project_by_id_for_user(&self.state.user_id, project_id)?
            .ok_or_else(|| BusinessError::missing("project"))
    }

    pub(crate) fn session_for_user(
        &self,
        session_id: &str,
    ) -> SdkResult<suncode_data::SessionRecord> {
        let session = self
            .state
            .store
            .session_by_id(session_id)?
            .ok_or_else(|| BusinessError::missing("session"))?;
        let project_id = session.project_id.as_deref().ok_or_else(|| {
            BusinessError::new("scope_denied", "session is not bound to a project")
        })?;
        self.project_for_user(project_id)?;
        Ok(session)
    }
}

#[derive(Clone)]
struct SqliteApiKeyResolver {
    store: Store,
    overrides: std::collections::BTreeMap<String, String>,
}

impl suncode_llm::ApiKeyResolver for SqliteApiKeyResolver {
    fn api_key(&self, provider_id: &str) -> Option<String> {
        self.overrides
            .get(provider_id)
            .cloned()
            .or_else(|| self.store.llm_provider_api_key(provider_id).ok().flatten())
    }
}

fn credential_states(
    store: &Store,
    overrides: &std::collections::BTreeMap<String, String>,
) -> SdkResult<Vec<CredentialState>> {
    store
        .llm_model_providers(false)?
        .into_iter()
        .map(|provider| {
            Ok(CredentialState {
                configured: overrides.contains_key(&provider.provider_id)
                    || store.llm_provider_api_key(&provider.provider_id)?.is_some(),
                provider: provider.provider_id,
            })
        })
        .collect()
}

fn provider_exists(store: &Store, provider_id: &str) -> SdkResult<bool> {
    Ok(store
        .llm_model_providers(false)?
        .iter()
        .any(|provider| provider.provider_id == provider_id))
}

fn provider_models(
    store: &Store,
    provider: &LlmModelProviderRecord,
) -> SdkResult<Vec<ModelDescriptor>> {
    Ok(store
        .llm_models(true)?
        .into_iter()
        .filter(|model| model.provider_id == provider.provider_id)
        .map(|model| ModelDescriptor {
            provider: provider.provider_id.clone(),
            provider_label: provider.display_name.clone(),
            id: model.model_id,
            wire_model: model.request_model,
            api_base: provider.endpoint.clone(),
            default_api_base: provider.default_endpoint.clone(),
            capabilities: ModelCapabilities {
                streaming: model.supports_streaming,
                tool_use: model.supports_tool_use,
                vision: model.supports_vision,
                structured_output: model.supports_structured_output,
                cancellation: model.supports_cancellation,
                reasoning_effort: model.supports_reasoning_effort,
                computer_use: model.supports_computer_use,
            },
            reasoning_efforts: model.reasoning_efforts.clone(),
            limits: ModelLimits {
                max_input_tokens: Some(model.context_tokens),
                auto_compact_tokens: Some(model.auto_compact_tokens),
                max_output_tokens: model.max_output_tokens,
            },
            availability: "configured".into(),
        })
        .collect())
}

fn provider_adapter(
    provider: &LlmModelProviderRecord,
    keys: Arc<dyn suncode_llm::ApiKeyResolver>,
    verify_https_certificates: Arc<AtomicBool>,
    use_system_certificates: Arc<AtomicBool>,
    certificate_path: Arc<RwLock<Option<PathBuf>>>,
    proxy_configuration: Arc<RwLock<HttpProxyConfiguration>>,
) -> SdkResult<Arc<dyn suncode_llm::LlmProvider>> {
    match provider.adapter_type.as_str() {
        "openai" => Ok(Arc::new(
            OpenAiCompatibleProvider::new_with_network_configuration(
                provider.provider_id.clone(),
                provider.display_name.clone(),
                provider.endpoint.clone(),
                keys,
                verify_https_certificates,
                use_system_certificates,
                certificate_path,
                proxy_configuration,
            ),
        )),
        "anthropic" => Ok(Arc::new(AnthropicProvider::new_with_network_configuration(
            provider.provider_id.clone(),
            provider.display_name.clone(),
            provider.endpoint.clone(),
            keys,
            verify_https_certificates,
            use_system_certificates,
            certificate_path,
            proxy_configuration,
        ))),
        adapter_type => Err(BusinessError::new(
            "provider_adapter_unsupported",
            format!("provider adapter is not supported: {adapter_type}"),
        )),
    }
}

fn registry_from_store(
    store: &Store,
    keys: Arc<dyn suncode_llm::ApiKeyResolver>,
    verify_https_certificates: Arc<AtomicBool>,
    use_system_certificates: Arc<AtomicBool>,
    certificate_path: Arc<RwLock<Option<PathBuf>>>,
    proxy_configuration: Arc<RwLock<HttpProxyConfiguration>>,
) -> SdkResult<ModelProviderRegistry> {
    let providers = store.llm_model_providers(true)?;
    let models = store.llm_models(true)?;
    let mut registry = ModelProviderRegistry::new();
    for provider in providers {
        let provider_models = models
            .iter()
            .filter(|model| model.provider_id == provider.provider_id)
            .map(|model| ModelDescriptor {
                provider: provider.provider_id.clone(),
                provider_label: provider.display_name.clone(),
                id: model.model_id.clone(),
                wire_model: model.request_model.clone(),
                api_base: provider.endpoint.clone(),
                default_api_base: provider.default_endpoint.clone(),
                capabilities: ModelCapabilities {
                    streaming: model.supports_streaming,
                    tool_use: model.supports_tool_use,
                    vision: model.supports_vision,
                    structured_output: model.supports_structured_output,
                    cancellation: model.supports_cancellation,
                    reasoning_effort: model.supports_reasoning_effort,
                    computer_use: model.supports_computer_use,
                },
                reasoning_efforts: model.reasoning_efforts.clone(),
                limits: ModelLimits {
                    max_input_tokens: Some(model.context_tokens),
                    auto_compact_tokens: Some(model.auto_compact_tokens),
                    max_output_tokens: model.max_output_tokens,
                },
                availability: "configured".into(),
            })
            .collect::<Vec<_>>();
        if provider_models.is_empty() {
            continue;
        }
        let adapter = provider_adapter(
            &provider,
            keys.clone(),
            verify_https_certificates.clone(),
            use_system_certificates.clone(),
            certificate_path.clone(),
            proxy_configuration.clone(),
        )?;
        registry
            .register(provider.provider_id, adapter, provider_models)
            .map_err(|error| {
                BusinessError::new("provider_registration_failed", error.to_string())
            })?;
    }
    Ok(registry)
}

async fn build_state<F>(
    config: &Config,
    user_id: &str,
    options: SdkOpenOptions,
    configure_providers: F,
) -> SdkResult<AgentState>
where
    F: FnOnce(&mut ModelProviderRegistry) -> Result<(), BusinessError>,
{
    let store = Store::open(&config.database_path)?;
    let persisted = store.persisted_settings(None, None)?;
    let runtime_config = config.runtime(&persisted).map_err(BusinessError::invalid)?;
    let global = persisted.global();
    configure_logging(&global.log, &config.data_dir, options.log_console_output);
    let verify_https_certificates = Arc::new(AtomicBool::new(global.tls.verify_https_certificates));
    let use_system_certificates = Arc::new(AtomicBool::new(global.tls.use_system_certificates));
    let certificate_path = Arc::new(RwLock::new(global.tls.certificate_path.clone()));
    let proxy_configuration = Arc::new(RwLock::new(global.proxy.clone()));
    let operations = Arc::new(
        suncode_tool::Operations::new_with_https_certificate_verification(
            config.data_dir.join("operations"),
            verify_https_certificates.clone(),
        )
        .map_err(|error| BusinessError::unavailable(error.to_string()))?,
    );
    operations.set_certificate_configuration(
        use_system_certificates.load(Ordering::SeqCst),
        certificate_path.read().ok().and_then(|path| path.clone()),
    );
    operations.set_proxy_configuration(
        proxy_configuration
            .read()
            .map(|configuration| configuration.clone())
            .unwrap_or_default(),
    );
    let events = SessionEventHub::new(256);
    let mut providers = registry_from_store(
        &store,
        Arc::new(SqliteApiKeyResolver {
            store: store.clone(),
            overrides: runtime_config.credentials.clone(),
        }),
        verify_https_certificates.clone(),
        use_system_certificates.clone(),
        certificate_path.clone(),
        proxy_configuration.clone(),
    )?;
    configure_providers(&mut providers)
        .map_err(|error| BusinessError::new("provider_registration_failed", error.to_string()))?;
    let providers = Arc::new(providers);
    let agent = AgentRuntime::new_with_user_id_and_capabilities_and_config(
        store.clone(),
        providers.clone(),
        operations.clone(),
        events.clone(),
        config.non_interactive,
        config.data_dir.clone(),
        user_id.to_owned(),
        AgentHostCapabilities {
            browser_use: options.host_capabilities.browser_use,
            computer_use: options.host_capabilities.computer_use,
        },
        config.clone(),
    );
    logging::debug(
        "browser.host",
        format!(
            "sdk_open host_capability={} persisted_enabled={}",
            options.host_capabilities.browser_use, global.browser_use_enabled
        ),
    );
    let attention_events = agent.attention_event_hub();
    if options.host_capabilities.computer_use && global.computer_use_enabled {
        match suncode_computer::EnigoBackend::new() {
            Ok(backend) => agent.install_computer_backend(Box::new(backend))?,
            Err(error) => logging::warn(
                "computer.backend",
                format!("state=unavailable error={error}"),
            ),
        }
    }
    let state = AgentState {
        store,
        user_id: user_id.to_owned(),
        operations,
        active_project: Arc::new(Mutex::new(None)),
        events,
        attention_events,
        verify_https_certificates,
        use_system_certificates,
        certificate_path,
        proxy_configuration,
        agent,
        providers,
        credential_overrides: runtime_config.credentials,
        host_capabilities: options.host_capabilities,
        log_console_output: options.log_console_output,
    };
    state.agent.recover().await?;
    Ok(state)
}

fn normalize_provider_endpoint(value: &str) -> SdkResult<String> {
    let value = value.trim();
    let url = url::Url::parse(value)
        .map_err(|_| BusinessError::invalid("provider URL must be a valid HTTP or HTTPS URL"))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(BusinessError::invalid(
            "provider URL must be a valid HTTP or HTTPS URL",
        ));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(BusinessError::invalid(
            "provider URL must not contain embedded credentials",
        ));
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err(BusinessError::invalid(
            "provider URL must not contain a query or fragment",
        ));
    }
    Ok(value.trim_end_matches('/').to_string())
}

fn configure_logging(log: &LogSettings, data_dir: &Path, console_output: SdkLogConsoleOutput) {
    logging::configure(
        data_dir,
        logging::Config {
            level: &log.level,
            directory: Some(&log.directory),
            max_bytes: log.max_bytes,
            retention: log.retention,
            console: match console_output {
                SdkLogConsoleOutput::None => logging::ConsoleOutput::None,
                SdkLogConsoleOutput::Stderr => logging::ConsoleOutput::Stderr,
                SdkLogConsoleOutput::Stdout => logging::ConsoleOutput::Stdout,
                SdkLogConsoleOutput::Both => logging::ConsoleOutput::Both,
            },
        },
    );
}

/// Checks a client write against the settings schema. Proxy rows change only
/// together, through `set_proxy_configuration`.
fn validate_setting(scope: &str, key: &str, value: &Value) -> SdkResult<()> {
    if matches!(
        key,
        "proxy_mode"
            | "proxy_url"
            | "proxy_username"
            | "proxy_password"
            | "proxy_bypass"
            | "proxy_password_configured"
    ) {
        return Err(BusinessError::invalid(
            "proxy settings must be updated through set_proxy_configuration",
        ));
    }
    let scope = scope
        .parse::<SettingScope>()
        .map_err(|_| BusinessError::invalid("scope must be global, project, or session"))?;
    suncode_config::validate_setting(scope, key, value).map_err(BusinessError::invalid)
}

pub struct AsyncAgentSdk {
    _lock: Option<AgentLock>,
    data_dir: PathBuf,
    state: AgentState,
    remote: Option<Arc<suncode_remote::RemoteController>>,
}

impl AsyncAgentSdk {
    pub(crate) fn from_state(state: AgentState) -> Self {
        Self {
            _lock: None,
            data_dir: PathBuf::new(),
            state,
            remote: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn from_state_for_test(state: AgentState) -> Self {
        Self::from_state(state)
    }
}

fn detail_string(error: &BusinessError, name: &str) -> SdkResult<String> {
    error
        .details
        .get(name)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| BusinessError::unavailable(format!("approval outcome is missing {name}")))
}

fn emit_event(state: &AgentState, session_id: &str, payload: EventPayload) -> SdkResult<()> {
    state
        .events
        .publish_projected(session_id, payload, |payload| {
            let event_type = payload.event_type();
            let projected = state.store.append_content(
                session_id,
                event_type.as_str(),
                &payload.clone().into_value(),
            )?;
            Ok(projected.occurred_at)
        })
        .map(|_| ())
}

fn sanitize_image_extension(value: &str) -> SdkResult<String> {
    let value = value.trim().trim_start_matches('.').to_ascii_lowercase();
    if value.is_empty() || value.len() > 16 {
        return Err(BusinessError::invalid("image extension is invalid"));
    }
    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric())
    {
        return Err(BusinessError::invalid("image extension is invalid"));
    }
    Ok(value)
}
