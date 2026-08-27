use serde::Serialize;
use suncode_data::{BusinessError, Store};
use suncode_llm::ApiKeyResolver;

#[derive(Clone)]
pub struct CredentialStore {
    store: Store,
}

#[derive(Debug, Clone, Serialize)]
pub struct CredentialState {
    pub provider: String,
    pub configured: bool,
}

impl CredentialStore {
    pub fn load(store: Store) -> Self {
        Self { store }
    }

    #[cfg(test)]
    pub fn memory(
        deepseek: Option<&str>,
        zhipu: Option<&str>,
        openai: Option<&str>,
        kimi: Option<&str>,
        claude: Option<&str>,
        gemini: Option<&str>,
    ) -> Self {
        let store = Store::open_memory().expect("test store");
        for (provider, value) in ["deepseek", "zhipu", "openai", "kimi", "claude", "gemini"]
            .into_iter()
            .zip([deepseek, zhipu, openai, kimi, claude, gemini])
        {
            if let Some(value) = value {
                store
                    .set_llm_provider_api_key(provider, value)
                    .expect("provider secret");
            }
        }
        Self { store }
    }

    pub fn configured(&self, provider_id: &str) -> bool {
        self.value(provider_id).is_some()
    }

    pub fn state(&self) -> Vec<CredentialState> {
        self.store
            .llm_model_providers(false)
            .unwrap_or_default()
            .into_iter()
            .map(|provider| CredentialState {
                provider: provider.provider_id.clone(),
                configured: self.configured(&provider.provider_id),
            })
            .collect()
    }

    pub fn set(&self, provider_id: &str, value: &str) -> Result<(), String> {
        self.store
            .set_llm_provider_api_key(provider_id, value)
            .map_err(map_error)
    }

    pub fn delete(&self, provider_id: &str) -> Result<(), String> {
        self.store
            .delete_llm_provider_api_key(provider_id)
            .map_err(map_error)
    }

    fn value(&self, provider_id: &str) -> Option<String> {
        self.store.llm_provider_api_key(provider_id).ok().flatten()
    }
}

impl ApiKeyResolver for CredentialStore {
    fn api_key(&self, provider_id: &str) -> Option<String> {
        self.value(provider_id)
    }
}

fn map_error(error: BusinessError) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::CredentialStore;
    use suncode_data::Store;
    use suncode_llm::ApiKeyResolver;

    #[test]
    fn reports_every_provider_without_exposing_secret_values() {
        let credentials = CredentialStore::memory(
            None,
            None,
            None,
            Some("kimi-secret"),
            Some("claude-secret"),
            None,
        );

        let states = credentials.state();
        assert_eq!(states.len(), 6);
        assert!(states
            .iter()
            .any(|state| state.provider == "kimi" && state.configured));
        assert!(states
            .iter()
            .any(|state| state.provider == "claude" && state.configured));
        assert!(states
            .iter()
            .any(|state| state.provider == "gemini" && !state.configured));
    }

    #[test]
    fn resolves_api_keys_exclusively_from_sqlite() {
        let store = Store::open_memory().expect("test store");
        store
            .set_llm_provider_api_key("openai", "sqlite-secret")
            .expect("provider secret");
        let credentials = CredentialStore::load(store);

        assert_eq!(
            credentials.api_key("openai").as_deref(),
            Some("sqlite-secret")
        );
        assert_eq!(credentials.api_key("unknown-provider"), None);
    }
}
