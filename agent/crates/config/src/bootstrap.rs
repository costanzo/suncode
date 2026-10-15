//! Host bootstrap configuration: storage locations and interaction mode.

use crate::{environment::EnvironmentSource, runtime::RuntimeConfig, settings::PersistedSettings};
use std::path::{Path, PathBuf};

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

    /// Default locations with environment overrides disabled. Infallible
    /// because only environment values can be malformed.
    pub fn disabled() -> Self {
        let user_directory = default_user_directory();
        let data_dir = user_directory.join(".suncode");
        Self {
            database_path: default_database_path(&data_dir),
            data_dir,
            user_directory,
            non_interactive: false,
            environment_source: EnvironmentSource::Disabled,
        }
    }

    pub fn load_with_environment(source: EnvironmentSource) -> Result<Self, String> {
        let data_dir = source
            .path("SUNCODE_DATA_DIRECTORY")
            .unwrap_or_else(default_data_dir);
        let database_path = source
            .path("SUNCODE_DATABASE_PATH")
            .unwrap_or_else(|| default_database_path(&data_dir));
        let user_directory = source
            .path("SUNCODE_USER_DIRECTORY")
            .unwrap_or_else(default_user_directory);
        let non_interactive = source.bool("SUNCODE_NON_INTERACTIVE")?.unwrap_or(false);
        Ok(Self {
            data_dir,
            database_path,
            user_directory,
            non_interactive,
            environment_source: source,
        })
    }

    pub fn runtime(&self, persisted: &PersistedSettings) -> Result<RuntimeConfig, String> {
        RuntimeConfig::resolve(self.environment_source, persisted)
    }
}

fn default_database_path(data_dir: &Path) -> PathBuf {
    data_dir.join("data/sqlite/agent.sqlite3")
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
