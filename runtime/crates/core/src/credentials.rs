use serde::Serialize;
use std::process::{Command, Stdio};
use suncode_db::{PersistenceError, Store};
use suncode_llm::ApiKeyResolver;

#[derive(Clone)]
pub struct CredentialStore {
    store: Store,
    non_interactive: bool,
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

    pub fn as_str(self) -> &'static str {
        match self {
            Self::DeepSeek => "deepseek",
            Self::Zhipu => "zhipu",
            Self::OpenAI => "openai",
            Self::Kimi => "kimi",
            Self::Claude => "claude",
            Self::Gemini => "gemini",
        }
    }

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
        let credentials = Self {
            store,
            non_interactive,
            deepseek_override,
            zhipu_override,
            openai_override,
            kimi_override,
            claude_override,
            gemini_override,
        };
        credentials.migrate_legacy_keychain();
        credentials
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
        for (provider, value) in ProviderKind::ALL
            .into_iter()
            .zip([deepseek, zhipu, openai, kimi, claude, gemini])
        {
            if let Some(value) = value {
                store
                    .set_llm_provider_api_key(provider.as_str(), value)
                    .expect("provider secret");
            }
        }
        Self {
            store,
            non_interactive: false,
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

    fn migrate_legacy_keychain(&self) {
        if self.non_interactive || !cfg!(target_os = "macos") {
            return;
        }
        for provider in ProviderKind::ALL {
            if self
                .store
                .llm_provider_api_key(provider.as_str())
                .ok()
                .flatten()
                .is_some()
            {
                continue;
            }
            let Some(value) = keychain_get(provider.as_str()) else {
                continue;
            };
            if self
                .store
                .set_llm_provider_api_key(provider.as_str(), &value)
                .is_ok()
            {
                let _ = keychain_delete(provider.as_str());
            }
        }
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

#[cfg(target_os = "macos")]
fn keychain_get(account: &str) -> Option<String> {
    let output = Command::new("/usr/bin/security")
        .args([
            "find-generic-password",
            "-a",
            account,
            "-s",
            "com.suncode.runtime",
            "-w",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(not(target_os = "macos"))]
fn keychain_get(_account: &str) -> Option<String> {
    None
}

#[cfg(target_os = "macos")]
fn keychain_delete(account: &str) -> Result<(), ()> {
    let status = Command::new("/usr/bin/security")
        .args([
            "delete-generic-password",
            "-a",
            account,
            "-s",
            "com.suncode.runtime",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|_| ())?;
    if status.success() {
        Ok(())
    } else {
        Err(())
    }
}

#[cfg(not(target_os = "macos"))]
fn keychain_delete(_account: &str) -> Result<(), ()> {
    Ok(())
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
