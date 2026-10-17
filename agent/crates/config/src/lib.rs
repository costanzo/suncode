//! Shared bootstrap configuration used by the embedded agent and native hosts.
//!
//! - `bootstrap`: `Config`, the host's storage locations and interaction mode.
//! - `runtime`: `RuntimeConfig`, per-turn policy resolved from layered sources.
//! - `settings`: the persisted settings schema, write validation, and typed
//!   views of the global, project, and session scopes.
//! - `environment`: `SUNCODE_*` process environment access gated by the host.

mod bootstrap;
mod environment;
mod runtime;
mod settings;

pub use bootstrap::Config;
pub use environment::EnvironmentSource;
pub use runtime::RuntimeConfig;
pub use settings::{
    validate_setting, GlobalSettings, LogSettings, PersistedSettings, ProjectSettings,
    SessionSettings, SettingScope, SettingValues, TlsSettings,
};

#[cfg(test)]
mod tests;
