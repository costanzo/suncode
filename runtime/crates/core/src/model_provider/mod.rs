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

pub use catalog::{ModelDescriptor, ModelLimits};
pub use deepseek::DeepSeekProvider;
pub use openai_compatible::OpenAiCompatibleProvider;

#[derive(Clone)]
pub struct ModelProviderRegistry {
    deepseek: Arc<DeepSeekProvider>,
    zhipu: Arc<OpenAiCompatibleProvider>,
    openai: Arc<OpenAiCompatibleProvider>,
}

impl ModelProviderRegistry {
    pub fn new(
        deepseek_endpoint: String,
        deepseek_model: String,
        zhipu_endpoint: String,
        zhipu_model: String,
        openai_endpoint: String,
        openai_model: String,
        credentials: CredentialStore,
    ) -> Self {
        Self {
            deepseek: Arc::new(DeepSeekProvider::new(
                deepseek_endpoint,
                deepseek_model,
                credentials.clone(),
            )),
            zhipu: Arc::new(OpenAiCompatibleProvider::new(
                ProviderKind::Zhipu,
                "Zhipu GLM",
                zhipu_endpoint,
                zhipu_model,
                credentials.clone(),
            )),
            openai: Arc::new(OpenAiCompatibleProvider::new(
                ProviderKind::OpenAI,
                "OpenAI",
                openai_endpoint,
                openai_model,
                credentials,
            )),
        }
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

    pub fn provider(&self, model_id: &str) -> Option<Arc<dyn LlmProvider>> {
        match model_id {
            catalog::DEEPSEEK_MODEL_ID => Some(self.deepseek.clone() as Arc<dyn LlmProvider>),
            catalog::ZHIPU_MODEL_ID => Some(self.zhipu.clone() as Arc<dyn LlmProvider>),
            catalog::OPENAI_MODEL_ID => Some(self.openai.clone() as Arc<dyn LlmProvider>),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ModelProviderRegistry;
    use crate::credentials::CredentialStore;

    #[test]
    fn advertises_only_registered_models() {
        let registry = ModelProviderRegistry::new(
            "http://localhost".into(),
            "deepseek-v4-flash".into(),
            "http://localhost".into(),
            "glm-5.2".into(),
            "http://localhost".into(),
            "gpt-5.6-sol".into(),
            CredentialStore::memory(None, None, None),
        );
        assert!(registry.provider("deepseek-v4-flash").is_some());
        assert!(registry.provider("glm-5.2").is_some());
        assert!(registry.provider("gpt-5.6-sol").is_some());
        assert!(registry.provider("unknown-model").is_none());
    }
}
