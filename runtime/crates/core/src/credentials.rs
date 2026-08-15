use crate::{persistence::PersistenceError, persistence::Store};
use serde::Serialize;
use std::process::{Command, Stdio};

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

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|provider| provider.as_str() == value)
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

    pub fn label(self) -> &'static str {
        match self {
            Self::DeepSeek => "DeepSeek",
            Self::Zhipu => "Zhipu GLM",
            Self::OpenAI => "OpenAI",
            Self::Kimi => "Kimi",
            Self::Claude => "Claude",
            Self::Gemini => "Gemini",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CredentialState {
    pub provider: &'static str,
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
                    .set_secret(provider.as_str(), value)
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

    pub fn configured(&self, provider: ProviderKind) -> bool {
        self.value(provider).is_some()
    }

    pub fn state(&self) -> Vec<CredentialState> {
        ProviderKind::ALL
            .into_iter()
            .map(|provider| CredentialState {
                provider: provider.as_str(),
                configured: self.configured(provider),
            })
            .collect()
    }

    pub fn api_key(&self, provider: ProviderKind) -> Option<String> {
        self.value(provider)
    }

    pub fn set(&self, provider: ProviderKind, value: &str) -> Result<(), String> {
        self.store
            .set_secret(provider.as_str(), value)
            .map_err(map_error)
    }

    pub fn delete(&self, provider: ProviderKind) -> Result<(), String> {
        self.store
            .delete_secret(provider.as_str())
            .map_err(map_error)
    }

    fn value(&self, provider: ProviderKind) -> Option<String> {
        let override_value = match provider {
            ProviderKind::DeepSeek => self.deepseek_override.as_ref(),
            ProviderKind::Zhipu => self.zhipu_override.as_ref(),
            ProviderKind::OpenAI => self.openai_override.as_ref(),
            ProviderKind::Kimi => self.kimi_override.as_ref(),
            ProviderKind::Claude => self.claude_override.as_ref(),
            ProviderKind::Gemini => self.gemini_override.as_ref(),
        };
        override_value
            .cloned()
            .or_else(|| self.store.secret_value(provider.as_str()).ok().flatten())
    }

    fn migrate_legacy_keychain(&self) {
        if self.non_interactive || !cfg!(target_os = "macos") {
            return;
        }
        for provider in ProviderKind::ALL {
            if self
                .store
                .secret_value(provider.as_str())
                .ok()
                .flatten()
                .is_some()
            {
                continue;
            }
            let Some(value) = keychain_get(provider.as_str()) else {
                continue;
            };
            if self.store.set_secret(provider.as_str(), &value).is_ok() {
                let _ = keychain_delete(provider.as_str());
            }
        }
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
