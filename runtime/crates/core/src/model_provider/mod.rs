//! Provider registry and built-in model catalog.

mod catalog;
pub mod deepseek;

use crate::{credentials::CredentialStore, llm::LlmProvider};
use std::sync::Arc;

pub use catalog::ModelDescriptor;
pub use deepseek::DeepSeekProvider;

#[derive(Clone)]
pub struct ModelProviderRegistry {
    deepseek: Arc<DeepSeekProvider>,
}

impl ModelProviderRegistry {
    pub fn new(endpoint: String, wire_model: String, credentials: CredentialStore) -> Self {
        Self {
            deepseek: Arc::new(DeepSeekProvider::new(endpoint, wire_model, credentials)),
        }
    }

    pub fn models(&self) -> Vec<ModelDescriptor> {
        vec![catalog::deepseek_model()]
    }

    pub fn provider(&self, model_id: &str) -> Option<Arc<dyn LlmProvider>> {
        (model_id == catalog::DEEPSEEK_MODEL_ID)
            .then(|| self.deepseek.clone() as Arc<dyn LlmProvider>)
    }

    pub fn is_advertised(&self, model_id: &str) -> bool {
        self.provider(model_id).is_some()
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
            CredentialStore::memory(None),
        );
        assert!(registry.is_advertised("deepseek-v4-flash"));
        assert!(!registry.is_advertised("unknown-model"));
    }
}
