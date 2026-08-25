use serde::Serialize;
use suncode_data::{PersistenceError, Store};
use suncode_llm::ApiKeyResolver;

#[derive(Clone)]
pub struct CredentialStore {
    store: Store,
    deepseek_override: Option<String>,
    zhipu_override: Option<String>,
    openai_override: Option<String>,
    kimi_override: Option<String>,
    claude_override: Option<String>,
    gemini_override: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    DeepSeek,
    Zhipu,
    OpenAI,
    Kimi,
    Claude,
    Gemini,
}

impl ProviderKind {
    pub const ALL: [Self; 6] = [
        Self::DeepSeek,
        Self::Zhipu,
        Self::OpenAI,
        Self::Kimi,
        Self::Claude,
        Self::Gemini,
    ];

    pub fn api_key_envs(self) -> &'static [&'static str] {
        match self {
            Self::DeepSeek => &["DEEPSEEK_API_KEY"],
            Self::Zhipu => &["ZHIPU_API_KEY", "ZAI_API_KEY"],
            Self::OpenAI => &["OPENAI_API_KEY"],
            Self::Kimi => &["MOONSHOT_API_KEY", "KIMI_API_KEY"],
            Self::Claude => &["ANTHROPIC_API_KEY", "CLAUDE_API_KEY"],
            Self::Gemini => &["GEMINI_API_KEY", "GOOGLE_API_KEY"],
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CredentialState {
    pub provider: String,
    pub configured: bool,
}

impl CredentialStore {
    pub fn load(store: Store, non_interactive: bool) -> Self {
        let deepseek_override = load_override(non_interactive, ProviderKind::DeepSeek);
        let zhipu_override = load_override(non_interactive, ProviderKind::Zhipu);
        let openai_override = load_override(non_interactive, ProviderKind::OpenAI);
        let kimi_override = load_override(non_interactive, ProviderKind::Kimi);
        let claude_override = load_override(non_interactive, ProviderKind::Claude);
        let gemini_override = load_override(non_interactive, ProviderKind::Gemini);
        Self {
            store,
            deepseek_override,
            zhipu_override,
            openai_override,
            kimi_override,
            claude_override,
            gemini_override,
        }
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
        Self {
            store,
            deepseek_override: None,
            zhipu_override: None,
            openai_override: None,
            kimi_override: None,
            claude_override: None,
            gemini_override: None,
        }
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
        let override_value = match provider_id {
            "deepseek" => self.deepseek_override.as_ref(),
            "zhipu" => self.zhipu_override.as_ref(),
            "openai" => self.openai_override.as_ref(),
            "kimi" => self.kimi_override.as_ref(),
            "claude" => self.claude_override.as_ref(),
            "gemini" => self.gemini_override.as_ref(),
            _ => None,
        };
        override_value
            .cloned()
            .or_else(|| self.store.llm_provider_api_key(provider_id).ok().flatten())
    }
}

impl ApiKeyResolver for CredentialStore {
    fn api_key(&self, provider_id: &str) -> Option<String> {
        self.value(provider_id)
    }
}

fn map_error(error: PersistenceError) -> String {
    error.to_string()
}

fn load_override(non_interactive: bool, provider: ProviderKind) -> Option<String> {
    if !non_interactive {
        return None;
    }
    provider
        .api_key_envs()
        .iter()
        .find_map(|name| std::env::var(name).ok())
        .filter(|value| !value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::{CredentialStore, ProviderKind};

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
        assert_eq!(states.len(), ProviderKind::ALL.len());
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
}
