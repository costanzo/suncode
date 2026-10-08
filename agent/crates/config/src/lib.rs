//! Shared bootstrap configuration used by the embedded agent and native hosts.

use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentSource {
    Disabled,
    Process,
}

impl Default for EnvironmentSource {
    fn default() -> Self {
        Self::Disabled
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PersistedSettings {
    pub global: BTreeMap<String, Value>,
    pub project: BTreeMap<String, Value>,
    pub session: BTreeMap<String, Value>,
}

impl PersistedSettings {
    pub fn insert(&mut self, scope: &str, key: String, value: Value) -> Result<(), String> {
        match scope {
            "global" => {
                self.global.insert(key, value);
            }
            "project" => {
                self.project.insert(key, value);
            }
            "session" => {
                self.session.insert(key, value);
            }
            _ => return Err(format!("unknown persisted settings scope: {scope}")),
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub full_control: Option<bool>,
    pub tool_allowlist: Option<BTreeSet<String>>,
    pub turn_timeout_ms: u64,
    pub bash_timeout_ms: Option<u64>,
    pub tool_call_limit: Option<u32>,
    pub credentials: BTreeMap<String, String>,
}

impl RuntimeConfig {
    pub const DEFAULT_TURN_TIMEOUT_MS: u64 = 600_000;
}

#[derive(Debug, Clone)]
pub struct Config {
    pub data_dir: PathBuf,
    pub database_path: PathBuf,
    pub user_directory: PathBuf,
    pub non_interactive: bool,
    pub environment_source: EnvironmentSource,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        Self::load_with_environment(EnvironmentSource::Process)
    }

    pub fn load_with_environment(source: EnvironmentSource) -> Result<Self, String> {
        let data_dir = env_path(source, "SUNCODE_DATA_DIRECTORY").unwrap_or_else(default_data_dir);
        let database_path = env_path(source, "SUNCODE_DATABASE_PATH")
            .unwrap_or_else(|| data_dir.join("data/sqlite/agent.sqlite3"));
        let user_directory =
            env_path(source, "SUNCODE_USER_DIRECTORY").unwrap_or_else(default_user_directory);
        let non_interactive = env_bool(source, "SUNCODE_NON_INTERACTIVE", false)?;
        Ok(Self {
            data_dir,
            database_path,
            user_directory,
            non_interactive,
            environment_source: source,
        })
    }

    pub fn runtime(&self, persisted: &PersistedSettings) -> Result<RuntimeConfig, String> {
        let full_control = env_or_setting_bool(
            self.environment_source,
            "SUNCODE_FULL_CONTROL",
            &persisted.session,
            &persisted.project,
            &persisted.global,
            "full_control",
        )?;
        let tool_allowlist = match env_value(self.environment_source, "SUNCODE_TOOL_ALLOWLIST") {
            Some(value) => Some(parse_allowlist(&value)?),
            None => setting_string(&persisted.session, "tool_allowlist")
                .or_else(|| setting_string(&persisted.project, "tool_allowlist"))
                .or_else(|| setting_string(&persisted.global, "tool_allowlist"))
                .map(|value| parse_allowlist(&value))
                .transpose()?,
        };
        let turn_timeout_ms = env_or_setting_u64(
            self.environment_source,
            "SUNCODE_TURN_TIMEOUT_MS",
            &persisted.session,
            &persisted.project,
            &persisted.global,
            "turn_timeout_ms",
        )?
        .unwrap_or(RuntimeConfig::DEFAULT_TURN_TIMEOUT_MS);
        validate_range("turn timeout", turn_timeout_ms, 1_000, 86_400_000)?;
        let bash_timeout_ms = env_or_setting_u64(
            self.environment_source,
            "SUNCODE_BASH_TIMEOUT_MS",
            &persisted.session,
            &persisted.project,
            &persisted.global,
            "bash_timeout_ms",
        )?;
        if let Some(value) = bash_timeout_ms {
            validate_range("bash timeout", value, 1, 600_000)?;
        }
        let tool_call_limit = env_or_setting_u64(
            self.environment_source,
            "SUNCODE_TOOL_CALL_LIMIT",
            &persisted.session,
            &persisted.project,
            &persisted.global,
            "tool_call_limit",
        )?
        .map(|value| {
            validate_range("tool call limit", value, 1, 256)
                .and_then(|v| u32::try_from(v).map_err(|_| "tool call limit is invalid".to_owned()))
        })
        .transpose()?;
        let mut credentials = BTreeMap::new();
        if self.environment_source == EnvironmentSource::Process {
            for provider in ["openai", "anthropic", "deepseek", "zhipu", "kimi", "gemini"] {
                let name = format!("SUNCODE_{}_API_KEY", provider.to_ascii_uppercase());
                if let Some(value) = std::env::var_os(&name)
                    .and_then(|v| v.into_string().ok())
                    .filter(|v| !v.is_empty())
                {
                    credentials.insert(provider.to_string(), value);
                }
            }
        }
        Ok(RuntimeConfig {
            full_control,
            tool_allowlist,
            turn_timeout_ms,
            bash_timeout_ms,
            tool_call_limit,
            credentials,
        })
    }
}

fn env_path(source: EnvironmentSource, name: &str) -> Option<PathBuf> {
    (source == EnvironmentSource::Process)
        .then(|| std::env::var_os(name).map(PathBuf::from))
        .flatten()
}

fn env_value(source: EnvironmentSource, name: &str) -> Option<String> {
    (source == EnvironmentSource::Process)
        .then(|| std::env::var(name).ok())
        .flatten()
}

fn env_bool(source: EnvironmentSource, name: &str, fallback: bool) -> Result<bool, String> {
    let Some(value) = env_value(source, name) else {
        return Ok(fallback);
    };
    match value.to_ascii_lowercase().as_str() {
        "1" | "true" => Ok(true),
        "0" | "false" => Ok(false),
        _ => Err(format!("{name} must be true or false")),
    }
}

fn parse_allowlist(value: &str) -> Result<BTreeSet<String>, String> {
    let values = value
        .split(',')
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    if values.is_empty() {
        return Err("SUNCODE_TOOL_ALLOWLIST must contain at least one tool".into());
    }
    if values
        .iter()
        .any(|v| v.len() > 128 || v.chars().any(char::is_control))
    {
        return Err("SUNCODE_TOOL_ALLOWLIST contains an invalid tool name".into());
    }
    Ok(values)
}

fn setting_value<'a>(
    session: &'a BTreeMap<String, Value>,
    project: &'a BTreeMap<String, Value>,
    global: &'a BTreeMap<String, Value>,
    key: &str,
) -> Option<&'a Value> {
    session
        .get(key)
        .or_else(|| project.get(key))
        .or_else(|| global.get(key))
}

fn setting_string(map: &BTreeMap<String, Value>, key: &str) -> Option<String> {
    map.get(key).and_then(Value::as_str).map(str::to_string)
}

fn env_or_setting_bool(
    source: EnvironmentSource,
    env: &str,
    session: &BTreeMap<String, Value>,
    project: &BTreeMap<String, Value>,
    global: &BTreeMap<String, Value>,
    key: &str,
) -> Result<Option<bool>, String> {
    if let Some(value) = env_value(source, env) {
        return Ok(Some(match value.to_ascii_lowercase().as_str() {
            "1" | "true" => true,
            "0" | "false" => false,
            _ => return Err(format!("{env} must be true or false")),
        }));
    }
    setting_value(session, project, global, key)
        .map(|v| {
            v.as_bool()
                .ok_or_else(|| format!("{key} must be a boolean"))
        })
        .transpose()
}

fn env_or_setting_u64(
    source: EnvironmentSource,
    env: &str,
    session: &BTreeMap<String, Value>,
    project: &BTreeMap<String, Value>,
    global: &BTreeMap<String, Value>,
    key: &str,
) -> Result<Option<u64>, String> {
    if let Some(value) = env_value(source, env) {
        return value
            .parse()
            .map(Some)
            .map_err(|_| format!("{env} must be an integer"));
    }
    setting_value(session, project, global, key)
        .map(|v| {
            v.as_u64()
                .ok_or_else(|| format!("{key} must be an integer"))
        })
        .transpose()
}

fn validate_range(name: &str, value: u64, min: u64, max: u64) -> Result<u64, String> {
    if (min..=max).contains(&value) {
        Ok(value)
    } else {
        Err(format!("{name} must be between {min} and {max}"))
    }
}

fn default_data_dir() -> PathBuf {
    default_user_directory().join(".suncode")
}

fn default_user_directory() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn environment_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    #[test]
    fn disabled_environment_uses_persisted_values() {
        let config = Config::load_with_environment(EnvironmentSource::Disabled).unwrap();
        let persisted = PersistedSettings {
            session: [("full_control".into(), Value::Bool(true))]
                .into_iter()
                .collect(),
            project: [("tool_call_limit".into(), Value::Number(32.into()))]
                .into_iter()
                .collect(),
            ..Default::default()
        };
        let runtime = config.runtime(&persisted).unwrap();
        assert_eq!(runtime.full_control, Some(true));
        assert_eq!(runtime.tool_call_limit, Some(32));
    }

    #[test]
    fn process_environment_overrides_persisted_values() {
        let _guard = environment_lock();
        std::env::set_var("SUNCODE_FULL_CONTROL", "false");
        std::env::set_var("SUNCODE_TOOL_ALLOWLIST", "read,bash");
        std::env::set_var("SUNCODE_TOOL_CALL_LIMIT", "12");
        let config = Config::load_with_environment(EnvironmentSource::Process).unwrap();
        let persisted = PersistedSettings {
            session: [("full_control".into(), Value::Bool(true))]
                .into_iter()
                .collect(),
            ..Default::default()
        };
        let runtime = config.runtime(&persisted).unwrap();
        assert_eq!(runtime.full_control, Some(false));
        assert_eq!(runtime.tool_call_limit, Some(12));
        assert!(runtime.tool_allowlist.unwrap().contains("bash"));
        for name in [
            "SUNCODE_FULL_CONTROL",
            "SUNCODE_TOOL_ALLOWLIST",
            "SUNCODE_TOOL_CALL_LIMIT",
        ] {
            std::env::remove_var(name);
        }
    }

    #[test]
    fn disabled_environment_ignores_process_overrides_and_credentials() {
        let _guard = environment_lock();
        std::env::set_var("SUNCODE_FULL_CONTROL", "true");
        std::env::set_var("SUNCODE_OPENAI_API_KEY", "secret");
        let config = Config::load_with_environment(EnvironmentSource::Disabled).unwrap();
        let runtime = config.runtime(&PersistedSettings::default()).unwrap();
        assert_eq!(runtime.full_control, None);
        assert!(runtime.credentials.is_empty());
        std::env::remove_var("SUNCODE_FULL_CONTROL");
        std::env::remove_var("SUNCODE_OPENAI_API_KEY");
    }

    #[test]
    fn persisted_settings_reject_unknown_scope() {
        let mut settings = PersistedSettings::default();
        assert!(settings
            .insert("unknown", "value".into(), Value::Null)
            .is_err());
    }
}
