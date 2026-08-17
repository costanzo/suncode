use crate::{LlmProvider, ModelDescriptor, ModelLimits};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;

#[derive(Clone)]
pub struct ModelRoute {
    pub provider: Arc<dyn LlmProvider>,
    pub provider_id: String,
    pub wire_model: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RegistrationError {
    #[error("provider ID cannot be empty")]
    EmptyProviderId,
    #[error("provider `{0}` is already registered")]
    DuplicateProvider(String),
    #[error("provider `{0}` must register at least one model")]
    EmptyModels(String),
    #[error("model ID cannot be empty")]
    EmptyModelId,
    #[error("model `{model_id}` belongs to `{actual_provider}`, not `{expected_provider}`")]
    ProviderMismatch {
        model_id: String,
        expected_provider: String,
        actual_provider: String,
    },
    #[error("model `{0}` is already registered")]
    DuplicateModel(String),
}

#[derive(Default)]
pub struct ModelProviderRegistry {
    providers: HashMap<String, Arc<dyn LlmProvider>>,
    models: Vec<ModelDescriptor>,
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
    ) -> Result<(), RegistrationError> {
        let provider_id = provider_id.into();
        if provider_id.trim().is_empty() {
            return Err(RegistrationError::EmptyProviderId);
        }
        if self.providers.contains_key(&provider_id) {
            return Err(RegistrationError::DuplicateProvider(provider_id));
        }
        if models.is_empty() {
            return Err(RegistrationError::EmptyModels(provider_id));
        }

        let mut pending_ids = std::collections::HashSet::new();
        for model in &models {
            if model.id.trim().is_empty() {
                return Err(RegistrationError::EmptyModelId);
            }
            if model.provider != provider_id {
                return Err(RegistrationError::ProviderMismatch {
                    model_id: model.id.clone(),
                    expected_provider: provider_id.clone(),
                    actual_provider: model.provider.clone(),
                });
            }
            if self.models.iter().any(|current| current.id == model.id)
                || !pending_ids.insert(model.id.clone())
            {
                return Err(RegistrationError::DuplicateModel(model.id.clone()));
            }
        }

        self.providers.insert(provider_id, provider);
        self.models.extend(models);
        Ok(())
    }

    pub fn models(&self) -> Vec<ModelDescriptor> {
        self.models.clone()
    }

    pub fn limits(&self, model_id: &str) -> Option<ModelLimits> {
        self.models
            .iter()
            .find(|model| model.id == model_id)
            .map(|model| model.limits)
    }

    pub fn route(&self, model_id: &str) -> Option<ModelRoute> {
        let model = self.models.iter().find(|model| model.id == model_id)?;
        Some(ModelRoute {
            provider: self.providers.get(&model.provider)?.clone(),
            provider_id: model.provider.clone(),
            wire_model: model.wire_model.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{ModelProviderRegistry, RegistrationError};
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
            capabilities: ModelCapabilities {
                streaming: true,
                tool_use: true,
                vision: false,
                structured_output: false,
                cancellation: true,
            },
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
        assert_eq!(
            result,
            Err(RegistrationError::DuplicateModel("same".into()))
        );
        assert!(registry.models().is_empty());
        assert!(registry.route("same").is_none());
    }
}
