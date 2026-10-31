//! The persisted settings schema and typed views of each scope.
//!
//! Settings are stored as one key/value row per scope. Every key SunCode
//! reads is declared once in [`SCHEMA`] with its type and the scopes that
//! may store it; writes are validated against it, and reads go through the
//! typed views below instead of looking values up by name.

use serde_json::Value;
use std::{path::PathBuf, str::FromStr};
use suncode_common::{HttpProxyConfiguration, HttpProxyMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingScope {
    Global,
    Project,
    Session,
}

impl SettingScope {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Project => "project",
            Self::Session => "session",
        }
    }
}

impl FromStr for SettingScope {
    type Err = String;

    fn from_str(scope: &str) -> Result<Self, Self::Err> {
        match scope {
            "global" => Ok(Self::Global),
            "project" => Ok(Self::Project),
            "session" => Ok(Self::Session),
            _ => Err(format!("unknown persisted settings scope: {scope}")),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Kind {
    Bool,
    String,
    StringArray,
    LogLevel,
    Shell,
    Integer { min: u64, max: u64 },
}

struct Spec {
    key: &'static str,
    scopes: &'static [SettingScope],
    kind: Kind,
}

const fn spec(key: &'static str, scopes: &'static [SettingScope], kind: Kind) -> Spec {
    Spec { key, scopes, kind }
}

/// An integer from `min` through `max`; `u64::MAX` means no upper bound.
const fn int(min: u64, max: u64) -> Kind {
    Kind::Integer { min, max }
}

use SettingScope::{Global, Project, Session};

const ANY_SCOPE: &[SettingScope] = &[Global, Project, Session];
const LOG_LEVELS: [&str; 6] = ["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "OFF"];

/// Every known key. Keys not listed here (such as the remote connection's
/// own state) are stored as given.
#[rustfmt::skip]
const SCHEMA: &[Spec] = &[
    spec("agent_shell", &[Global], Kind::Shell),
    spec("terminal_shell", &[Global], Kind::Shell),
    spec("default_model", &[Global, Project], Kind::String),
    spec("log_level", &[Global], Kind::LogLevel),
    spec("log_directory", &[Global], Kind::String),
    spec("log_max_bytes", &[Global], int(1024, u64::MAX)),
    spec("log_retention", &[Global], int(0, 100)),
    spec("image_directory", &[Global], Kind::String),
    spec("browser_use_enabled", &[Global], Kind::Bool),
    spec("computer_use_enabled", &[Global], Kind::Bool),
    spec("verify_https_certificates", &[Global], Kind::Bool),
    spec("use_system_certificates", &[Global], Kind::Bool),
    spec("certificate_path", &[Global], Kind::String),
    spec("proxy_mode", &[Global], Kind::String),
    spec("proxy_url", &[Global], Kind::String),
    spec("proxy_username", &[Global], Kind::String),
    spec("proxy_password", &[Global], Kind::String),
    spec("proxy_bypass", &[Global], Kind::StringArray),
    spec("tool_call_limit", &[Project], int(1, 256)),
    spec("preview_program", &[Project], Kind::String),
    spec("preview_url", &[Project], Kind::String),
    spec("preview_cwd", &[Project], Kind::String),
    spec("preview_args", &[Project], Kind::StringArray),
    spec("full_control", &[Session], Kind::Bool),
    spec("tool_allowlist", ANY_SCOPE, Kind::String),
    spec("turn_timeout_ms", ANY_SCOPE, int(1_000, 86_400_000)),
    spec("bash_timeout_ms", ANY_SCOPE, int(1, 600_000)),
];

/// Checks that `value` may be stored under `key` in `scope`. Unknown keys are
/// accepted unchanged.
pub fn validate_setting(scope: SettingScope, key: &str, value: &Value) -> Result<(), String> {
    let Some(spec) = SCHEMA.iter().find(|spec| spec.key == key) else {
        return Ok(());
    };
    if !spec.scopes.contains(&scope) {
        return Err(match spec.scopes {
            [only] => format!("{key} is a {}-only setting", only.as_str()),
            scopes => format!(
                "{key} is a {} setting",
                scopes
                    .iter()
                    .map(|scope| scope.as_str())
                    .collect::<Vec<_>>()
                    .join(" or ")
            ),
        });
    }
    let valid = match spec.kind {
        Kind::Shell => {
            serde_json::from_value::<suncode_common::shell::ShellConfiguration>(value.clone())
                .is_ok_and(|configuration| configuration.validate())
        }
        Kind::Bool => value.is_boolean(),
        Kind::String => value.is_string(),
        Kind::StringArray => value
            .as_array()
            .is_some_and(|values| values.iter().all(Value::is_string)),
        Kind::LogLevel => {
            let Some(level) = value.as_str() else {
                return Err(format!("{key} must be a string"));
            };
            LOG_LEVELS.contains(&level.trim().to_ascii_uppercase().as_str())
        }
        Kind::Integer { min, max } => value
            .as_u64()
            .is_some_and(|number| (min..=max).contains(&number)),
    };
    if valid {
        return Ok(());
    }
    Err(match spec.kind {
        Kind::Shell => format!("{key} must be a valid shell configuration"),
        Kind::Bool => format!("{key} must be a boolean"),
        Kind::String => format!("{key} must be a string"),
        Kind::StringArray => format!("{key} must be an array of strings"),
        Kind::LogLevel => format!("{key} must be {}", join_or(&LOG_LEVELS)),
        Kind::Integer { min, max: u64::MAX } => {
            format!("{key} must be an integer greater than or equal to {min}")
        }
        Kind::Integer { min, max } => {
            format!("{key} must be an integer between {min} and {max}")
        }
    })
}

fn join_or(items: &[&str]) -> String {
    match items {
        [rest @ .., last] if !rest.is_empty() => format!("{}, or {last}", rest.join(", ")),
        items => items.join(""),
    }
}

/// The stored settings that apply to one session, one typed struct per scope.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PersistedSettings {
    pub global: GlobalSettings,
    pub project: ProjectSettings,
    pub session: SessionSettings,
}

impl PersistedSettings {
    /// Applies one stored row to its scope. Unknown keys are ignored.
    pub fn insert(&mut self, scope: &str, key: &str, value: &Value) -> Result<(), String> {
        match scope.parse()? {
            Global => {
                self.global.insert(key, value);
                Ok(())
            }
            Project => self.project.insert(key, value),
            Session => self.session.insert(key, value),
        }
    }

    /// Project settings with an unset `default_model` taken from global.
    pub fn project(&self) -> ProjectSettings {
        let mut project = self.project.clone();
        if project.default_model.is_none() {
            project.default_model = self.global.default_model.clone();
        }
        project
    }

    /// Overrides resolved from the most specific scope.
    pub(crate) fn overrides(&self) -> RuntimeOverrides {
        let layers = [
            &self.session.overrides,
            &self.project.overrides,
            &self.global.overrides,
        ];
        RuntimeOverrides {
            tool_allowlist: layers.iter().find_map(|layer| layer.tool_allowlist.clone()),
            turn_timeout_ms: layers.iter().find_map(|layer| layer.turn_timeout_ms),
            bash_timeout_ms: layers.iter().find_map(|layer| layer.bash_timeout_ms),
        }
    }
}

/// Runtime limits any scope may set; the most specific scope wins.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeOverrides {
    pub tool_allowlist: Option<String>,
    pub turn_timeout_ms: Option<u64>,
    pub bash_timeout_ms: Option<u64>,
}

impl RuntimeOverrides {
    /// Applies `key` if it is an override; `Ok(false)` for any other key.
    fn insert(&mut self, key: &str, value: &Value) -> Result<bool, String> {
        match key {
            "tool_allowlist" => self.tool_allowlist = Some(strict(key, key, value, string)?),
            "turn_timeout_ms" => {
                self.turn_timeout_ms = Some(strict(key, key, value, Value::as_u64)?)
            }
            "bash_timeout_ms" => {
                self.bash_timeout_ms = Some(strict(key, key, value, Value::as_u64)?)
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

/// A value that must have the declared type. `label` names it in the error.
fn strict<'a, T>(
    key: &str,
    label: &str,
    value: &'a Value,
    read: impl Fn(&'a Value) -> Option<T>,
) -> Result<T, String> {
    read(value).ok_or_else(|| {
        let kind = SCHEMA
            .iter()
            .find(|spec| spec.key == key)
            .map(|spec| spec.kind);
        match kind {
            Some(Kind::Bool) => format!("{label} must be a boolean"),
            Some(Kind::Integer { .. }) => format!("{label} must be an integer"),
            Some(Kind::StringArray) => format!("{label} must be an array of strings"),
            _ => format!("{label} must be a string"),
        }
    })
}

fn string(value: &Value) -> Option<String> {
    value.as_str().map(str::to_string)
}

fn trimmed(value: &Value) -> Option<Option<String>> {
    value.as_str().map(|value| {
        Some(value.trim())
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    })
}

fn strings(value: &Value) -> Option<Vec<String>> {
    value.as_array().map(|values| {
        values
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect()
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogSettings {
    pub level: String,
    /// Empty means the default `<data directory>/logs`.
    pub directory: String,
    pub max_bytes: u64,
    pub retention: usize,
}

impl Default for LogSettings {
    fn default() -> Self {
        Self {
            level: "INFO".into(),
            directory: String::new(),
            max_bytes: 10 * 1024 * 1024,
            retention: 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsSettings {
    pub verify_https_certificates: bool,
    pub use_system_certificates: bool,
    pub certificate_path: Option<PathBuf>,
}

impl Default for TlsSettings {
    fn default() -> Self {
        Self {
            verify_https_certificates: true,
            use_system_certificates: true,
            certificate_path: None,
        }
    }
}

/// Replaces `target` when the stored value had the expected type.
fn set<T>(target: &mut T, value: Option<T>) {
    if let Some(value) = value {
        *target = value;
    }
}

/// Global settings. A stored value with the wrong type keeps the default, so
/// a damaged row cannot prevent startup.
#[derive(Clone, Default, PartialEq)]
pub struct GlobalSettings {
    /// A non-empty trimmed model id.
    pub default_model: Option<String>,
    pub agent_shell: suncode_common::shell::ShellConfiguration,
    pub terminal_shell: suncode_common::shell::ShellConfiguration,
    pub log: LogSettings,
    /// Empty means the default `<data directory>/data/images`.
    pub image_directory: String,
    pub browser_use_enabled: bool,
    pub computer_use_enabled: bool,
    pub tls: TlsSettings,
    pub proxy: HttpProxyConfiguration,
    /// Mistyped overrides are ignored here; the runtime reads them strictly
    /// from the session and project rows.
    pub overrides: RuntimeOverrides,
}

impl GlobalSettings {
    pub fn from_rows<'a>(rows: impl IntoIterator<Item = (&'a str, &'a Value)>) -> Self {
        let mut settings = Self::default();
        for (key, value) in rows {
            settings.insert(key, value);
        }
        settings
    }

    fn insert(&mut self, key: &str, value: &Value) {
        match key {
            "agent_shell" => {
                if let Ok(configuration) = serde_json::from_value(value.clone()) {
                    self.agent_shell = configuration;
                }
            }
            "terminal_shell" => {
                if let Ok(configuration) = serde_json::from_value(value.clone()) {
                    self.terminal_shell = configuration;
                }
            }
            "default_model" => set(&mut self.default_model, trimmed(value)),
            "log_level" => set(&mut self.log.level, string(value)),
            "log_directory" => set(&mut self.log.directory, string(value)),
            "log_max_bytes" => set(&mut self.log.max_bytes, value.as_u64()),
            "log_retention" => set(
                &mut self.log.retention,
                value.as_u64().and_then(|value| usize::try_from(value).ok()),
            ),
            "image_directory" => set(&mut self.image_directory, string(value)),
            "browser_use_enabled" => set(&mut self.browser_use_enabled, value.as_bool()),
            "computer_use_enabled" => set(&mut self.computer_use_enabled, value.as_bool()),
            "verify_https_certificates" => {
                set(&mut self.tls.verify_https_certificates, value.as_bool())
            }
            "use_system_certificates" => {
                set(&mut self.tls.use_system_certificates, value.as_bool())
            }
            "certificate_path" => set(
                &mut self.tls.certificate_path,
                value.as_str().map(|path| {
                    Some(path)
                        .filter(|path| !path.trim().is_empty())
                        .map(PathBuf::from)
                }),
            ),
            "proxy_mode" => set(
                &mut self.proxy.mode,
                value.as_str().and_then(HttpProxyMode::parse),
            ),
            "proxy_url" => set(&mut self.proxy.url, string(value)),
            "proxy_username" => set(&mut self.proxy.username, string(value)),
            "proxy_password" => set(&mut self.proxy.password, string(value)),
            "proxy_bypass" => set(&mut self.proxy.bypass, strings(value)),
            key => {
                let _ = self.overrides.insert(key, value);
            }
        }
    }

    /// The stored rows for a proxy configuration.
    pub fn proxy_values(proxy: &HttpProxyConfiguration) -> Vec<(String, Value)> {
        vec![
            (
                "proxy_mode".into(),
                Value::String(proxy.mode.as_str().into()),
            ),
            ("proxy_url".into(), Value::String(proxy.url.clone())),
            (
                "proxy_username".into(),
                Value::String(proxy.username.clone()),
            ),
            (
                "proxy_password".into(),
                Value::String(proxy.password.clone()),
            ),
            (
                "proxy_bypass".into(),
                Value::Array(proxy.bypass.iter().cloned().map(Value::String).collect()),
            ),
        ]
    }
}

// The proxy password is a secret, so Debug omits it.
impl std::fmt::Debug for GlobalSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlobalSettings")
            .field("default_model", &self.default_model)
            .field("log", &self.log)
            .field("image_directory", &self.image_directory)
            .field("browser_use_enabled", &self.browser_use_enabled)
            .field("computer_use_enabled", &self.computer_use_enabled)
            .field("tls", &self.tls)
            .field("proxy_mode", &self.proxy.mode.as_str())
            .field("proxy_url", &self.proxy.url)
            .field(
                "proxy_password_configured",
                &!self.proxy.password.is_empty(),
            )
            .field("overrides", &self.overrides)
            .finish()
    }
}

/// Project settings. A mistyped value fails, as turn admission depends on
/// these values.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectSettings {
    /// A non-empty trimmed model id.
    pub default_model: Option<String>,
    pub tool_call_limit: Option<u32>,
    pub preview_program: Option<String>,
    pub preview_url: Option<String>,
    pub preview_cwd: Option<String>,
    pub preview_args: Option<Vec<String>>,
    pub overrides: RuntimeOverrides,
}

impl ProjectSettings {
    pub fn from_rows<'a>(
        rows: impl IntoIterator<Item = (&'a str, &'a Value)>,
    ) -> Result<Self, String> {
        let mut settings = Self::default();
        for (key, value) in rows {
            settings.insert(key, value)?;
        }
        Ok(settings)
    }

    fn insert(&mut self, key: &str, value: &Value) -> Result<(), String> {
        match key {
            "default_model" => {
                self.default_model = strict(key, "project default_model", value, trimmed)?;
            }
            "tool_call_limit" => {
                let limit = strict(key, key, value, Value::as_u64)?;
                self.tool_call_limit = Some(
                    u32::try_from(limit)
                        .ok()
                        .filter(|limit| (1..=256).contains(limit))
                        .ok_or("project tool_call_limit must be an integer between 1 and 256")?,
                );
            }
            // Preview values only prefill the host's form, so a mistyped one
            // is left unset.
            "preview_program" => self.preview_program = string(value),
            "preview_url" => self.preview_url = string(value),
            "preview_cwd" => self.preview_cwd = string(value),
            "preview_args" => self.preview_args = strings(value),
            key => {
                self.overrides.insert(key, value)?;
            }
        }
        Ok(())
    }
}

/// Session settings. A mistyped value fails, as approval depends on them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionSettings {
    /// `None` when unset, so a turn snapshot does not pin it and a later
    /// "allow for this session" still applies.
    pub full_control: Option<bool>,
    pub overrides: RuntimeOverrides,
}

impl SessionSettings {
    pub fn from_rows<'a>(
        rows: impl IntoIterator<Item = (&'a str, &'a Value)>,
    ) -> Result<Self, String> {
        let mut settings = Self::default();
        for (key, value) in rows {
            settings.insert(key, value)?;
        }
        Ok(settings)
    }

    fn insert(&mut self, key: &str, value: &Value) -> Result<(), String> {
        match key {
            "full_control" => {
                self.full_control = Some(strict(
                    key,
                    "session full_control configuration",
                    value,
                    Value::as_bool,
                )?);
            }
            key => {
                self.overrides.insert(key, value)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn persisted(rows: &[(&str, &str, Value)]) -> Result<PersistedSettings, String> {
        let mut settings = PersistedSettings::default();
        for (scope, key, value) in rows {
            settings.insert(scope, key, value)?;
        }
        Ok(settings)
    }

    #[test]
    fn persisted_settings_reject_unknown_scope() {
        assert!(persisted(&[("unknown", "value", Value::Null)]).is_err());
    }

    #[test]
    fn most_specific_scope_wins() {
        let mut rows = vec![
            ("global", "turn_timeout_ms", json!(1_000)),
            ("project", "turn_timeout_ms", json!(2_000)),
        ];
        assert_eq!(
            persisted(&rows).unwrap().overrides().turn_timeout_ms,
            Some(2_000)
        );
        rows.push(("session", "turn_timeout_ms", json!(3_000)));
        assert_eq!(
            persisted(&rows).unwrap().overrides().turn_timeout_ms,
            Some(3_000)
        );
    }

    #[test]
    fn writes_are_checked_against_the_schema() {
        let ok = |scope, key, value: Value| validate_setting(scope, key, &value);
        assert!(ok(Global, "log_level", json!("trace")).is_ok());
        assert!(ok(Global, "log_max_bytes", json!(1024)).is_ok());
        assert!(ok(Global, "log_retention", json!(0)).is_ok());
        assert!(ok(Project, "tool_call_limit", json!(256)).is_ok());
        assert!(ok(Project, "preview_args", json!(["run", "dev"])).is_ok());
        assert!(ok(Session, "full_control", json!(true)).is_ok());
        assert!(ok(Global, "remote_server_url", json!("https://example.test")).is_ok());

        assert_eq!(
            ok(Global, "log_level", json!("VERBOSE")).unwrap_err(),
            "log_level must be TRACE, DEBUG, INFO, WARN, ERROR, or OFF"
        );
        assert_eq!(
            ok(Project, "log_level", json!("INFO")).unwrap_err(),
            "log_level is a global-only setting"
        );
        assert_eq!(
            ok(Global, "log_max_bytes", json!(1023)).unwrap_err(),
            "log_max_bytes must be an integer greater than or equal to 1024"
        );
        assert_eq!(
            ok(Project, "tool_call_limit", json!(257)).unwrap_err(),
            "tool_call_limit must be an integer between 1 and 256"
        );
        assert_eq!(
            ok(Global, "full_control", json!(true)).unwrap_err(),
            "full_control is a session-only setting"
        );
        assert_eq!(
            ok(Session, "default_model", json!("m")).unwrap_err(),
            "default_model is a global or project setting"
        );
        assert!(ok(Project, "preview_args", json!(["run", 1])).is_err());
        assert!(ok(Global, "browser_use_enabled", json!("yes")).is_err());
    }

    #[test]
    fn global_settings_keep_defaults_for_missing_or_mistyped_values() {
        let defaults = GlobalSettings::default();
        assert_eq!(defaults.log, LogSettings::default());
        assert_eq!(defaults.tls, TlsSettings::default());
        assert!(defaults.proxy.mode == HttpProxyMode::System);
        assert!(!defaults.browser_use_enabled && !defaults.computer_use_enabled);

        let rows = [
            ("log_max_bytes", json!("large")),
            ("verify_https_certificates", json!(false)),
            ("certificate_path", json!("  ")),
            ("proxy_bypass", json!(["localhost", 3])),
            ("turn_timeout_ms", json!("slow")),
        ];
        let settings = GlobalSettings::from_rows(rows.iter().map(|(key, value)| (*key, value)));
        assert_eq!(settings.log.max_bytes, LogSettings::default().max_bytes);
        assert!(!settings.tls.verify_https_certificates);
        assert_eq!(settings.tls.certificate_path, None);
        assert_eq!(settings.proxy.bypass, vec!["localhost".to_string()]);
        assert_eq!(settings.overrides.turn_timeout_ms, None);
    }

    #[test]
    fn proxy_values_round_trip_and_debug_hides_the_password() {
        let proxy = HttpProxyConfiguration {
            mode: HttpProxyMode::Custom,
            url: "http://proxy.test:8080".into(),
            username: "user".into(),
            password: "secret-password".into(),
            bypass: vec!["internal.test".into()],
        };
        let rows = GlobalSettings::proxy_values(&proxy);
        let settings =
            GlobalSettings::from_rows(rows.iter().map(|(key, value)| (key.as_str(), value)));
        assert!(settings.proxy == proxy);
        assert!(!format!("{settings:?}").contains("secret-password"));
    }

    #[test]
    fn project_and_session_settings_reject_mistyped_values() {
        let mut rows = vec![("global", "default_model", json!("global-model"))];
        assert_eq!(
            persisted(&rows).unwrap().project().default_model.as_deref(),
            Some("global-model")
        );
        rows.push(("project", "default_model", json!(" project-model ")));
        rows.push(("project", "tool_call_limit", json!(32)));
        let project = persisted(&rows).unwrap().project();
        assert_eq!(project.default_model.as_deref(), Some("project-model"));
        assert_eq!(project.tool_call_limit, Some(32));

        assert!(persisted(&[("project", "tool_call_limit", json!(0))]).is_err());
        assert!(persisted(&[("project", "bash_timeout_ms", json!("slow"))]).is_err());
        assert_eq!(
            persisted(&[("session", "full_control", json!("yes"))]).unwrap_err(),
            "session full_control configuration must be a boolean"
        );
    }
}
