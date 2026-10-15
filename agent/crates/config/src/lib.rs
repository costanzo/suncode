//! Shared bootstrap configuration used by the embedded agent and native hosts.
//!
//! - `bootstrap`: `Config`, the host's storage locations and interaction mode.
//! - `runtime`: `RuntimeConfig`, per-turn policy resolved from layered sources.
//! - `settings`: persisted settings layered by global, project, and session scope.
//! - `environment`: `SUNCODE_*` process environment access gated by the host.

mod bootstrap;
mod environment;
mod runtime;
mod settings;

pub use bootstrap::Config;
pub use environment::EnvironmentSource;
pub use runtime::RuntimeConfig;
pub use settings::PersistedSettings;

#[cfg(test)]
mod tests;
