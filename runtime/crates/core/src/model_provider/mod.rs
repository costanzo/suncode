//! Provider registry and built-in model catalog.

mod catalog;
pub mod deepseek;
pub(crate) mod normalize;
mod openai_compatible;
pub(crate) mod stream;

use crate::{
    credentials::{CredentialStore, ProviderKind},
    llm::LlmProvider,
};
use std::sync::Arc;

pub use catalog::{ModelCapabilities, ModelDescriptor, ModelLimits};
pub use deepseek::DeepSeekProvider;
pub use openai_compatible::OpenAiCompatibleProvider;

#[derive(Clone)]
pub struct ModelRoute {
    pub provider: Arc<dyn LlmProvider>,
    pub wire_model: &'static str,
}

#[derive(Clone)]
pub struct ModelProviderRegistry {
    deepseek: Arc<DeepSeekProvider>,
    zhipu: Arc<OpenAiCompatibleProvider>,
    openai: Arc<OpenAiCompatibleProvider>,
    kimi: Arc<OpenAiCompatibleProvider>,
    claude: Arc<OpenAiCompatibleProvider>,
    gemini: Arc<OpenAiCompatibleProvider>,
}

impl ModelProviderRegistry {
    pub fn new(credentials: CredentialStore) -> Self {
        Self {
            deepseek: Arc::new(DeepSeekProvider::new(
                "https://api.deepseek.com".into(),
                credentials.clone(),
            )),
            zhipu: Arc::new(OpenAiCompatibleProvider::new(
                ProviderKind::Zhipu,
                "Zhipu GLM",
                "https://open.bigmodel.cn/api/paas/v4".into(),
                credentials.clone(),
            )),
            openai: Arc::new(OpenAiCompatibleProvider::new(
                ProviderKind::OpenAI,
                "OpenAI",
                "https://api.openai.com/v1".into(),
                credentials.clone(),
            )),
            kimi: Arc::new(OpenAiCompatibleProvider::new(
                ProviderKind::Kimi,
                "Kimi",
                "https://api.moonshot.ai/v1".into(),
                credentials.clone(),
            )),
            claude: Arc::new(OpenAiCompatibleProvider::new(
                ProviderKind::Claude,
                "Claude",
                "https://api.anthropic.com/v1".into(),
                credentials.clone(),
            )),
            gemini: Arc::new(OpenAiCompatibleProvider::new(
                ProviderKind::Gemini,
                "Gemini",
                "https://generativelanguage.googleapis.com/v1beta/openai".into(),
                credentials,
            )),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_deepseek_endpoint(endpoint: String, credentials: CredentialStore) -> Self {
        let mut registry = Self::new(credentials.clone());
        registry.deepseek = Arc::new(DeepSeekProvider::new(endpoint, credentials));
        registry
    }

    pub fn models(&self) -> Vec<ModelDescriptor> {
        catalog::all_models()
    }

    pub fn limits(&self, model_id: &str) -> Option<ModelLimits> {
        self.models()
            .into_iter()
            .find(|model| model.id == model_id)
            .map(|model| model.limits)
    }

    pub fn route(&self, model_id: &str) -> Option<ModelRoute> {
        let model = self
            .models()
            .into_iter()
            .find(|model| model.id == model_id)?;
        let provider = match model.provider {
            "deepseek" => self.deepseek.clone() as Arc<dyn LlmProvider>,
            "zhipu" => self.zhipu.clone() as Arc<dyn LlmProvider>,
            "openai" => self.openai.clone() as Arc<dyn LlmProvider>,
            "kimi" => self.kimi.clone() as Arc<dyn LlmProvider>,
            "claude" => self.claude.clone() as Arc<dyn LlmProvider>,
            "gemini" => self.gemini.clone() as Arc<dyn LlmProvider>,
            _ => return None,
        };
        Some(ModelRoute {
            provider,
            wire_model: model.wire_model,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ModelProviderRegistry;
    use crate::credentials::CredentialStore;

    #[test]
    fn advertises_only_registered_models() {
        let registry =
            ModelProviderRegistry::new(CredentialStore::memory(None, None, None, None, None, None));
        for model in registry.models() {
            assert!(
                registry.route(model.id).is_some(),
                "missing route for {}",
                model.id
            );
        }
        assert_eq!(registry.route("gpt-5.5").unwrap().wire_model, "gpt-5.5");
        assert_eq!(
            registry.route("gpt-5.6-sol").unwrap().wire_model,
            "gpt-5.6-sol"
        );
        assert_eq!(registry.models().len(), 12);
        assert!(registry.route("unknown-model").is_none());

        for provider in ["deepseek", "zhipu", "openai", "kimi", "claude", "gemini"] {
            assert_eq!(
                registry
                    .models()
                    .iter()
                    .filter(|model| model.provider == provider)
                    .count(),
                2,
                "provider {provider} should expose two models"
            );
        }
    }
}
