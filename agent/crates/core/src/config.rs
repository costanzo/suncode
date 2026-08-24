use std::path::PathBuf;

use crate::credentials::ProviderKind;

#[derive(Debug, Clone)]
pub struct Config {
    pub data_dir: PathBuf,
    pub database_path: PathBuf,
    pub non_interactive: bool,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let data_dir = std::env::var_os("SUNCODE_DATA_DIRECTORY")
            .map(PathBuf::from)
            .unwrap_or_else(default_data_dir);
        let database_path = std::env::var_os("SUNCODE_DATABASE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|| data_dir.join("data/sqlite/agent.sqlite3"));
        let non_interactive = env_bool("SUNCODE_NON_INTERACTIVE", false)?;
        if !non_interactive
            && ProviderKind::ALL
                .iter()
                .flat_map(|provider| provider.api_key_envs())
                .any(|name| std::env::var_os(name).is_some())
        {
            return Err(
                "provider API key environment overrides require SUNCODE_NON_INTERACTIVE=true"
                    .to_string(),
            );
        }
        Ok(Self {
            data_dir,
            database_path,
            non_interactive,
        })
    }
}

fn env_bool(name: &str, fallback: bool) -> Result<bool, String> {
    let Ok(value) = std::env::var(name) else {
        return Ok(fallback);
    };
    match value.to_ascii_lowercase().as_str() {
        "1" | "true" => Ok(true),
        "0" | "false" => Ok(false),
        _ => Err(format!("{name} must be true or false")),
    }
}

fn default_data_dir() -> PathBuf {
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".suncode")
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
}
