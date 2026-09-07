use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, RwLock,
    },
};
use suncode_agent::logging::{self, Level};
use suncode_agent::{agent::Agent, domain::SessionEvent, AgentLock};
use suncode_common::BusinessError;
use suncode_config::Config;
use suncode_data::{LlmModelProviderInput, LlmModelProviderRecord, Store};
use suncode_llm::{
    ModelCapabilities, ModelDescriptor, ModelLimits, ModelProviderRegistry,
    OpenAiCompatibleProvider,
};
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use crate::types::*;

mod checkpoints;
mod lifecycle;
mod project_sessions;
mod projects;
mod sessions;
mod settings;
mod subscriptions;
#[cfg(test)]
mod tests;
mod turns;

pub use subscriptions::AgentSubscription;

#[derive(Clone)]
struct AgentState {
    store: Store,
    operations: Arc<suncode_tool::Operations>,
    active_project: Arc<Mutex<Option<String>>>,
    events: broadcast::Sender<SessionEvent>,
    verify_https_certificates: Arc<AtomicBool>,
    use_system_certificates: Arc<AtomicBool>,
    certificate_path: Arc<RwLock<Option<PathBuf>>>,
    agent: Agent,
    providers: Arc<ModelProviderRegistry>,
}

#[derive(Clone)]
struct SqliteApiKeyResolver {
    store: Store,
}

impl suncode_llm::ApiKeyResolver for SqliteApiKeyResolver {
    fn api_key(&self, provider_id: &str) -> Option<String> {
        self.store.llm_provider_api_key(provider_id).ok().flatten()
    }
}

fn credential_states(store: &Store) -> SdkResult<Vec<CredentialState>> {
    store
        .llm_model_providers(false)?
        .into_iter()
        .map(|provider| {
            Ok(CredentialState {
                configured: store.llm_provider_api_key(&provider.provider_id)?.is_some(),
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

fn openai_provider(
    provider: &LlmModelProviderRecord,
    keys: Arc<dyn suncode_llm::ApiKeyResolver>,
    verify_https_certificates: Arc<AtomicBool>,
    use_system_certificates: Arc<AtomicBool>,
    certificate_path: Arc<RwLock<Option<PathBuf>>>,
) -> SdkResult<Arc<dyn suncode_llm::LlmProvider>> {
    match provider.adapter_type.as_str() {
        "openai" => Ok(Arc::new(
            OpenAiCompatibleProvider::new_with_tls_configuration(
                provider.provider_id.clone(),
                provider.display_name.clone(),
                provider.endpoint.clone(),
                keys,
                verify_https_certificates,
                use_system_certificates,
                certificate_path,
            ),
        )),
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
        let adapter = openai_provider(
            &provider,
            keys.clone(),
            verify_https_certificates.clone(),
            use_system_certificates.clone(),
            certificate_path.clone(),
        )?;
        registry
            .register(provider.provider_id, adapter, provider_models)
            .map_err(|error| {
                BusinessError::new("provider_registration_failed", error.to_string())
            })?;
    }
    Ok(registry)
}

async fn build_state<F>(config: &Config, configure_providers: F) -> SdkResult<AgentState>
where
    F: FnOnce(&mut ModelProviderRegistry) -> Result<(), BusinessError>,
{
    let store = Store::open(&config.database_path)?;
    configure_logging(&store, &config.data_dir)?;
    let verify_https_certificates = Arc::new(AtomicBool::new(global_bool_setting(
        &store,
        "verify_https_certificates",
        true,
    )?));
    let use_system_certificates = Arc::new(AtomicBool::new(global_bool_setting(
        &store,
        "use_system_certificates",
        true,
    )?));
    let certificate_path = Arc::new(RwLock::new(
        global_string_setting(&store, "certificate_path")?
            .filter(|value| !value.trim().is_empty())
            .map(PathBuf::from),
    ));
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
    let (events, _) = broadcast::channel(256);
    let mut providers = registry_from_store(
        &store,
        Arc::new(SqliteApiKeyResolver {
            store: store.clone(),
        }),
        verify_https_certificates.clone(),
        use_system_certificates.clone(),
        certificate_path.clone(),
    )?;
    configure_providers(&mut providers)
        .map_err(|error| BusinessError::new("provider_registration_failed", error.to_string()))?;
    let providers = Arc::new(providers);
    let agent = Agent::new(
        store.clone(),
        providers.clone(),
        operations.clone(),
        events.clone(),
        config.non_interactive,
    );
    let state = AgentState {
        store,
        operations,
        active_project: Arc::new(Mutex::new(None)),
        events,
        verify_https_certificates,
        use_system_certificates,
        certificate_path,
        agent,
        providers,
    };
    state.agent.recover().await?;
    Ok(state)
}

fn global_bool_setting(store: &Store, key: &str, fallback: bool) -> SdkResult<bool> {
    Ok(store
        .settings(None, None)?
        .into_iter()
        .find(|record| record.key == key)
        .and_then(|record| record.value.as_bool())
        .unwrap_or(fallback))
}

fn global_string_setting(store: &Store, key: &str) -> SdkResult<Option<String>> {
    Ok(store
        .settings(None, None)?
        .into_iter()
        .find(|record| record.key == key)
        .and_then(|record| record.value.as_str().map(str::to_string)))
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

fn configure_logging(store: &Store, data_dir: &Path) -> SdkResult<()> {
    let settings = store.settings(None, None)?;
    let value = |key: &str| {
        settings
            .iter()
            .find(|record| record.key == key)
            .map(|record| &record.value)
    };
    let level = value("log_level").and_then(Value::as_str).unwrap_or("INFO");
    let directory = value("log_directory").and_then(Value::as_str);
    let max_bytes = value("log_max_bytes")
        .and_then(Value::as_u64)
        .unwrap_or(10 * 1024 * 1024);
    let retention = value("log_retention")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(5);
    logging::configure(
        data_dir,
        logging::Config {
            level,
            directory,
            max_bytes,
            retention,
        },
    );
    Ok(())
}

fn validate_setting(scope: &str, key: &str, value: &Value) -> SdkResult<()> {
    if key == "image_directory" {
        if scope != "global" {
            return Err(BusinessError::invalid(
                "image_directory is a global-only setting",
            ));
        }
        if !value.is_string() {
            return Err(BusinessError::invalid("image_directory must be a string"));
        }
        return Ok(());
    }
    if key == "verify_https_certificates" {
        if scope != "global" {
            return Err(BusinessError::invalid(
                "verify_https_certificates is a global-only setting",
            ));
        }
        if !value.is_boolean() {
            return Err(BusinessError::invalid(
                "verify_https_certificates must be a boolean",
            ));
        }
        return Ok(());
    }
    if key == "use_system_certificates" {
        if scope != "global" {
            return Err(BusinessError::invalid(
                "use_system_certificates is a global-only setting",
            ));
        }
        if !value.is_boolean() {
            return Err(BusinessError::invalid(
                "use_system_certificates must be a boolean",
            ));
        }
        return Ok(());
    }
    if key == "certificate_path" {
        if scope != "global" {
            return Err(BusinessError::invalid(
                "certificate_path is a global-only setting",
            ));
        }
        if !value.is_string() {
            return Err(BusinessError::invalid("certificate_path must be a string"));
        }
        return Ok(());
    }
    if key == "tool_call_limit" {
        if scope != "project" {
            return Err(BusinessError::invalid(
                "tool_call_limit is a project-only setting",
            ));
        }
        if value
            .as_u64()
            .is_none_or(|limit| !(1..=256).contains(&limit))
        {
            return Err(BusinessError::invalid(
                "tool_call_limit must be an integer between 1 and 256",
            ));
        }
        return Ok(());
    }
    if key == "full_control" {
        if scope != "session" {
            return Err(BusinessError::invalid(
                "full_control is a session-only setting",
            ));
        }
        if !value.is_boolean() {
            return Err(BusinessError::invalid("full_control must be a boolean"));
        }
        return Ok(());
    }
    let is_logging_setting = matches!(
        key,
        "log_level" | "log_directory" | "log_max_bytes" | "log_retention"
    );
    if !is_logging_setting {
        return Ok(());
    }
    if scope != "global" {
        return Err(BusinessError::invalid(format!(
            "{key} is a global-only setting"
        )));
    }
    match key {
        "log_level" => {
            let Some(level) = value.as_str() else {
                return Err(BusinessError::invalid("log_level must be a string"));
            };
            if !matches!(
                level.trim().to_ascii_uppercase().as_str(),
                "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR" | "OFF"
            ) {
                return Err(BusinessError::invalid(
                    "log_level must be TRACE, DEBUG, INFO, WARN, ERROR, or OFF",
                ));
            }
        }
        "log_directory" if !value.is_string() => {
            return Err(BusinessError::invalid("log_directory must be a string"));
        }
        "log_max_bytes" if value.as_u64().is_none_or(|size| size < 1024) => {
            return Err(BusinessError::invalid(
                "log_max_bytes must be an integer greater than or equal to 1024",
            ));
        }
        "log_retention" if value.as_u64().is_none_or(|count| count > 100) => {
            return Err(BusinessError::invalid(
                "log_retention must be an integer between 0 and 100",
            ));
        }
        _ => {}
    }
    Ok(())
}

pub struct AgentSdk {
    _lock: Option<AgentLock>,
    data_dir: PathBuf,
    runtime: tokio::runtime::Runtime,
    state: AgentState,
}

#[cfg(test)]
impl AgentSdk {
    fn from_state_for_test(state: AgentState) -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("suncode-sdk-test")
            .build()
            .unwrap();
        Self {
            _lock: None,
            data_dir: PathBuf::new(),
            runtime,
            state,
        }
    }
}

fn operation_error(error: Value) -> BusinessError {
    BusinessError::new(
        error
            .get("code")
            .and_then(Value::as_str)
            .unwrap_or("operation_failed"),
        error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("operation failed"),
    )
}

fn decode_operation<T: DeserializeOwned>(value: Value, operation: &str) -> SdkResult<T> {
    serde_json::from_value(value).map_err(|error| {
        BusinessError::unavailable(format!("{operation} returned an invalid result: {error}"))
    })
}

fn detail_string(error: &BusinessError, name: &str) -> SdkResult<String> {
    error
        .details
        .get(name)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| BusinessError::unavailable(format!("approval outcome is missing {name}")))
}

fn emit_event(
    state: &AgentState,
    session_id: &str,
    event_type: &str,
    payload: Value,
) -> SdkResult<()> {
    let event = state
        .store
        .append_content(session_id, event_type, &payload)?;
    let _ = state.events.send(event);
    Ok(())
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
