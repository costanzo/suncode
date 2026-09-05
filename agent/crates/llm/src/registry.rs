use crate::{BusinessError, LlmProvider, ModelDescriptor, ModelLimits};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct ModelRoute {
    pub provider: Arc<dyn LlmProvider>,
    pub provider_id: String,
    pub wire_model: String,
}

#[derive(Default)]
struct RegistryState {
    providers: HashMap<String, Arc<dyn LlmProvider>>,
    models: Vec<ModelDescriptor>,
}

#[derive(Default)]
pub struct ModelProviderRegistry {
    state: RwLock<RegistryState>,
}

impl ModelProviderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers one trusted provider and all of its models atomically.
    pub fn register(
        &mut self,
        provider_id: impl Into<String>,
        provider: Arc<dyn LlmProvider>,
        models: Vec<ModelDescriptor>,
    ) -> Result<(), BusinessError> {
        let provider_id = provider_id.into();
        let state = self.state.get_mut().map_err(|_| {
            BusinessError::new(
                "provider_registry_unavailable",
                "provider registry is unavailable",
            )
        })?;
        if state.providers.contains_key(&provider_id) {
            return Err(BusinessError::new(
                "provider_registration_failed",
                format!("provider `{provider_id}` is already registered"),
            ));
        }
        validate_models(&state.models, &provider_id, &models, false)?;

        state.providers.insert(provider_id, provider);
        state.models.extend(models);
        Ok(())
    }

    /// Replaces one registered provider and its model descriptors atomically.
    pub fn replace(
        &self,
        provider_id: impl Into<String>,
        provider: Arc<dyn LlmProvider>,
        models: Vec<ModelDescriptor>,
    ) -> Result<(), BusinessError> {
        let provider_id = provider_id.into();
        let mut state = self.state.write().map_err(|_| {
            BusinessError::new(
                "provider_registry_unavailable",
                "provider registry is unavailable",
            )
        })?;
        if !state.providers.contains_key(&provider_id) {
            return Err(BusinessError::new(
                "provider_not_found",
                format!("provider `{provider_id}` is not registered"),
            ));
        }
        validate_models(&state.models, &provider_id, &models, true)?;
        state.providers.insert(provider_id.clone(), provider);
        let mut replacements = models
            .into_iter()
            .map(|model| (model.id.clone(), model))
            .collect::<HashMap<_, _>>();
        for model in state
            .models
            .iter_mut()
            .filter(|model| model.provider == provider_id)
        {
            *model = replacements.remove(&model.id).ok_or_else(|| {
                BusinessError::new(
                    "provider_registration_failed",
                    "replacement models do not match the registered provider",
                )
            })?;
        }
        Ok(())
    }

    pub fn models(&self) -> Vec<ModelDescriptor> {
        self.state
            .read()
            .map(|state| state.models.clone())
            .unwrap_or_default()
    }

    pub fn limits(&self, model_id: &str) -> Option<ModelLimits> {
        self.state
            .read()
            .ok()?
            .models
            .iter()
            .find(|model| model.id == model_id)
            .map(|model| model.limits)
    }

    pub fn supports_reasoning_effort(&self, model_id: &str) -> bool {
        self.state.read().is_ok_and(|state| {
            state
                .models
                .iter()
                .find(|model| model.id == model_id)
                .is_some_and(|model| model.capabilities.reasoning_effort)
        })
    }

    pub fn reasoning_efforts(&self, model_id: &str) -> Vec<String> {
        self.state
            .read()
            .ok()
            .and_then(|state| {
                state
                    .models
                    .iter()
                    .find(|model| model.id == model_id)
                    .map(|model| model.reasoning_efforts.clone())
            })
            .unwrap_or_default()
    }

    pub fn supports_vision(&self, model_id: &str) -> bool {
        self.state.read().is_ok_and(|state| {
            state
                .models
                .iter()
                .find(|model| model.id == model_id)
                .is_some_and(|model| model.capabilities.vision)
        })
    }

    pub fn route(&self, model_id: &str) -> Option<ModelRoute> {
        let state = self.state.read().ok()?;
        let model = state.models.iter().find(|model| model.id == model_id)?;
        Some(ModelRoute {
            provider: state.providers.get(&model.provider)?.clone(),
            provider_id: model.provider.clone(),
            wire_model: model.wire_model.clone(),
        })
    }
}

fn validate_models(
    existing: &[ModelDescriptor],
    provider_id: &str,
    models: &[ModelDescriptor],
    replacing: bool,
) -> Result<(), BusinessError> {
    if provider_id.trim().is_empty() {
        return Err(BusinessError::invalid("provider ID cannot be empty"));
    }
    if models.is_empty() {
        return Err(BusinessError::new(
            "provider_registration_failed",
            format!("provider `{provider_id}` must register at least one model"),
        ));
    }
    if replacing {
        let existing_ids = existing
            .iter()
            .filter(|model| model.provider == provider_id)
            .map(|model| model.id.as_str())
            .collect::<std::collections::HashSet<_>>();
        let replacement_ids = models
            .iter()
            .map(|model| model.id.as_str())
            .collect::<std::collections::HashSet<_>>();
        if existing_ids != replacement_ids {
            return Err(BusinessError::new(
                "provider_registration_failed",
                "replacement models do not match the registered provider",
            ));
        }
    }
    let mut pending_ids = std::collections::HashSet::new();
    for model in models {
        if model.id.trim().is_empty() {
            return Err(BusinessError::invalid("model ID cannot be empty"));
        }
        if model.provider != provider_id {
            return Err(BusinessError::new(
                "provider_registration_failed",
                format!(
                    "model `{}` belongs to `{}`, not `{provider_id}`",
                    model.id, model.provider
                ),
            ));
        }
        if existing.iter().any(|current| {
            current.id == model.id && (!replacing || current.provider != provider_id)
        }) || !pending_ids.insert(model.id.clone())
        {
            return Err(BusinessError::new(
                "provider_registration_failed",
                format!("model `{}` is already registered", model.id),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::ModelProviderRegistry;
    use crate::{
        Completion, CompletionFuture, CompletionRequest, LlmProvider, ModelCapabilities,
        ModelDescriptor, ModelLimits,
    };
    use std::sync::Arc;
    use tokio::sync::mpsc;
    use tokio_util::sync::CancellationToken;

    struct CustomProvider;

    impl LlmProvider for CustomProvider {
        fn complete<'a>(
            &'a self,
            _request: CompletionRequest<'a>,
            _cancellation: &'a CancellationToken,
            _deltas: mpsc::UnboundedSender<String>,
        ) -> CompletionFuture<'a> {
            Box::pin(async {
                Ok(Completion {
                    text: "custom".into(),
                    tool_calls: Vec::new(),
                    finish_reason: "stop".into(),
                    usage: None,
                    provider_request_id: None,
                    provider_response_id: None,
                })
            })
        }
    }

    fn custom_model(id: &str) -> ModelDescriptor {
        ModelDescriptor {
            provider: "enterprise".into(),
            provider_label: "Enterprise Gateway".into(),
            id: id.into(),
            wire_model: "company-model-v1".into(),
            api_base: "https://llm.example.invalid/v1".into(),
            default_api_base: "https://llm.example.invalid/v1".into(),
            capabilities: ModelCapabilities {
                streaming: true,
                tool_use: true,
                vision: false,
                structured_output: false,
                cancellation: true,
                reasoning_effort: false,
            },
            reasoning_efforts: Vec::new(),
            limits: ModelLimits {
                max_input_tokens: Some(32_000),
                auto_compact_tokens: Some(28_000),
                max_output_tokens: Some(4_000),
            },
            availability: "configured".into(),
        }
    }

    #[test]
    fn registers_a_custom_provider_and_owned_model_identifiers() {
        let mut registry = ModelProviderRegistry::new();
        registry
            .register(
                "enterprise",
                Arc::new(CustomProvider),
                vec![custom_model("internal-code")],
            )
            .unwrap();

        let route = registry.route("internal-code").unwrap();
        assert_eq!(route.provider_id, "enterprise");
        assert_eq!(route.wire_model, "company-model-v1");
    }

    #[test]
    fn rejects_duplicate_models_without_partial_registration() {
        let mut registry = ModelProviderRegistry::new();
        let result = registry.register(
            "enterprise",
            Arc::new(CustomProvider),
            vec![custom_model("same"), custom_model("same")],
        );
        assert_eq!(result.unwrap_err().code, "provider_registration_failed");
        assert!(registry.models().is_empty());
        assert!(registry.route("same").is_none());
    }

    #[test]
    fn replaces_one_provider_without_changing_other_routes() {
        let mut registry = ModelProviderRegistry::new();
        registry
            .register(
                "enterprise",
                Arc::new(CustomProvider),
                vec![custom_model("internal-code")],
            )
            .unwrap();
        let mut second = custom_model("second-code");
        second.provider = "second".into();
        registry
            .register("second", Arc::new(CustomProvider), vec![second])
            .unwrap();

        let mut replacement = custom_model("internal-code");
        replacement.api_base = "https://gateway.example.invalid/v2".into();
        registry
            .replace("enterprise", Arc::new(CustomProvider), vec![replacement])
            .unwrap();

        assert_eq!(
            registry
                .models()
                .into_iter()
                .find(|model| model.id == "internal-code")
                .unwrap()
                .api_base,
            "https://gateway.example.invalid/v2"
        );
        assert!(registry.route("second-code").is_some());
        assert_eq!(
            registry
                .models()
                .into_iter()
                .map(|model| model.id)
                .collect::<Vec<_>>(),
            ["internal-code", "second-code"]
        );
    }
}
