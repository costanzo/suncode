//! The persisted settings schema and typed views of each scope.
//!
//! Settings are stored as one key/value row per scope. Every key SunCode
//! reads is declared once in [`SCHEMA`] with its type and the scopes that
//! may store it; writes are validated against it, and reads go through the
//! typed views below instead of looking values up by name.

use serde_json::Value;
use std::{collections::BTreeMap, path::PathBuf, str::FromStr};
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

/// Stored values of one scope, by key.
pub type SettingValues = BTreeMap<String, Value>;

/// The stored settings that apply to one session, layered by scope.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PersistedSettings {
    pub global: SettingValues,
    pub project: SettingValues,
    pub session: SettingValues,
}

impl PersistedSettings {
    pub fn insert(&mut self, scope: &str, key: String, value: Value) -> Result<(), String> {
        self.scope_mut(scope.parse()?).insert(key, value);
        Ok(())
    }

    fn scope_mut(&mut self, scope: SettingScope) -> &mut SettingValues {
        match scope {
            Global => &mut self.global,
            Project => &mut self.project,
            Session => &mut self.session,
        }
    }

    /// Scopes from most to least specific.
    fn layers(&self) -> [&SettingValues; 3] {
        [&self.session, &self.project, &self.global]
    }

    /// The most specific value for `key`.
    fn get(&self, key: &str) -> Option<&Value> {
        self.layers().into_iter().find_map(|layer| layer.get(key))
    }

    /// Settings resolved per session, from the most specific scope.
    pub(crate) fn layered(&self) -> Result<LayeredSettings, String> {
        Ok(LayeredSettings {
            full_control: strict(
                self.get("full_control"),
                "full_control",
                "full_control",
                Value::as_bool,
            )?,
            tool_allowlist: self
                .layers()
                .into_iter()
                .find_map(|layer| layer.get("tool_allowlist").and_then(Value::as_str))
                .map(str::to_string),
            turn_timeout_ms: strict(
                self.get("turn_timeout_ms"),
                "turn_timeout_ms",
                "turn_timeout_ms",
                Value::as_u64,
            )?,
            bash_timeout_ms: strict(
                self.get("bash_timeout_ms"),
                "bash_timeout_ms",
                "bash_timeout_ms",
                Value::as_u64,
            )?,
            tool_call_limit: strict(
                self.get("tool_call_limit"),
                "tool_call_limit",
                "tool_call_limit",
                Value::as_u64,
            )?,
        })
    }

    /// The global settings view.
    pub fn global(&self) -> GlobalSettings {
        GlobalSettings::from_values(&self.global)
    }

    /// The project settings view; `default_model` falls back to global.
    pub fn project(&self) -> Result<ProjectSettings, String> {
        let mut settings = ProjectSettings::from_values(&self.project)?;
        if settings.default_model.is_none() {
            settings.default_model = self
                .global
                .get("default_model")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|model| !model.is_empty())
                .map(str::to_string);
        }
        Ok(settings)
    }

    /// The session settings view.
    pub fn session(&self) -> Result<SessionSettings, String> {
        SessionSettings::from_values(&self.session)
    }
}

/// Values the runtime resolves across all scopes.
pub(crate) struct LayeredSettings {
    pub full_control: Option<bool>,
    pub tool_allowlist: Option<String>,
    pub turn_timeout_ms: Option<u64>,
    pub bash_timeout_ms: Option<u64>,
    pub tool_call_limit: Option<u64>,
}

/// A value that must have the declared type when present. `label` names the
/// value in the error.
fn strict<'a, T>(
    value: Option<&'a Value>,
    key: &str,
    label: &str,
    read: impl Fn(&'a Value) -> Option<T>,
) -> Result<Option<T>, String> {
    value
        .map(|value| read(value).ok_or_else(|| type_error(key, label)))
        .transpose()
}

fn type_error(key: &str, label: &str) -> String {
    let spec = SCHEMA.iter().find(|spec| spec.key == key);
    match spec.map(|spec| spec.kind) {
        Some(Kind::Bool) => format!("{label} must be a boolean"),
        Some(Kind::Integer { .. }) => format!("{label} must be an integer"),
        Some(Kind::StringArray) => format!("{label} must be an array of strings"),
        _ => format!("{label} must be a string"),
    }
}

/// A value read leniently: a missing or mistyped value is treated as unset.
fn lenient<'a, T>(
    values: &'a SettingValues,
    key: &str,
    read: impl Fn(&'a Value) -> Option<T>,
) -> Option<T> {
    values.get(key).and_then(read)
}

fn string(value: &Value) -> Option<String> {
    value.as_str().map(str::to_string)
}

fn non_empty_path(value: &Value) -> Option<PathBuf> {
    value
        .as_str()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
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

/// Global settings. Stored values with the wrong type read as their
/// defaults, so a damaged row cannot prevent startup.
#[derive(Clone, Default, PartialEq)]
pub struct GlobalSettings {
    pub default_model: Option<String>,
    pub log: LogSettings,
    /// Empty means the default `<data directory>/data/images`.
    pub image_directory: String,
    pub browser_use_enabled: bool,
    pub computer_use_enabled: bool,
    pub tls: TlsSettings,
    pub proxy: HttpProxyConfiguration,
}

impl GlobalSettings {
    pub fn from_values(values: &SettingValues) -> Self {
        let log = LogSettings::default();
        let tls = TlsSettings::default();
        Self {
            default_model: lenient(values, "default_model", string),
            log: LogSettings {
                level: lenient(values, "log_level", string).unwrap_or(log.level),
                directory: lenient(values, "log_directory", string).unwrap_or(log.directory),
                max_bytes: lenient(values, "log_max_bytes", Value::as_u64).unwrap_or(log.max_bytes),
                retention: lenient(values, "log_retention", Value::as_u64)
                    .and_then(|value| usize::try_from(value).ok())
                    .unwrap_or(log.retention),
            },
            image_directory: lenient(values, "image_directory", string).unwrap_or_default(),
            browser_use_enabled: lenient(values, "browser_use_enabled", Value::as_bool)
                .unwrap_or(false),
            computer_use_enabled: lenient(values, "computer_use_enabled", Value::as_bool)
                .unwrap_or(false),
            tls: TlsSettings {
                verify_https_certificates: lenient(
                    values,
                    "verify_https_certificates",
                    Value::as_bool,
                )
                .unwrap_or(tls.verify_https_certificates),
                use_system_certificates: lenient(values, "use_system_certificates", Value::as_bool)
                    .unwrap_or(tls.use_system_certificates),
                certificate_path: lenient(values, "certificate_path", non_empty_path),
            },
            proxy: HttpProxyConfiguration {
                mode: lenient(values, "proxy_mode", Value::as_str)
                    .and_then(HttpProxyMode::parse)
                    .unwrap_or(HttpProxyMode::System),
                url: lenient(values, "proxy_url", string).unwrap_or_default(),
                username: lenient(values, "proxy_username", string).unwrap_or_default(),
                password: lenient(values, "proxy_password", string).unwrap_or_default(),
                bypass: lenient(values, "proxy_bypass", strings).unwrap_or_default(),
            },
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
            .finish()
    }
}

/// Project settings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectSettings {
    /// A non-empty trimmed model id.
    pub default_model: Option<String>,
    pub tool_call_limit: Option<u32>,
    pub preview_program: Option<String>,
    pub preview_url: Option<String>,
    pub preview_cwd: Option<String>,
    pub preview_args: Option<Vec<String>>,
}

impl ProjectSettings {
    /// Fails when a stored value has the wrong type, as turn admission depends
    /// on these values.
    pub fn from_values(values: &SettingValues) -> Result<Self, String> {
        let default_model = strict(
            values.get("default_model"),
            "default_model",
            "project default_model",
            Value::as_str,
        )?
        .map(str::trim)
        .filter(|model| !model.is_empty())
        .map(str::to_string);
        let tool_call_limit = strict(
            values.get("tool_call_limit"),
            "tool_call_limit",
            "tool_call_limit",
            Value::as_u64,
        )?
        .map(|limit| {
            u32::try_from(limit)
                .ok()
                .filter(|limit| (1..=256).contains(limit))
                .ok_or_else(|| {
                    "project tool_call_limit must be an integer between 1 and 256".to_string()
                })
        })
        .transpose()?;
        Ok(Self {
            default_model,
            tool_call_limit,
            preview_program: lenient(values, "preview_program", string),
            preview_url: lenient(values, "preview_url", string),
            preview_cwd: lenient(values, "preview_cwd", string),
            preview_args: lenient(values, "preview_args", strings),
        })
    }
}

/// Session settings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionSettings {
    pub full_control: bool,
}

impl SessionSettings {
    pub fn from_values(values: &SettingValues) -> Result<Self, String> {
        Ok(Self {
            full_control: strict(
                values.get("full_control"),
                "full_control",
                "session full_control configuration",
                Value::as_bool,
            )?
            .unwrap_or(false),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn persisted_settings_reject_unknown_scope() {
        let mut settings = PersistedSettings::default();
        assert!(settings
            .insert("unknown", "value".into(), Value::Null)
            .is_err());
    }

    #[test]
    fn most_specific_scope_wins() {
        let mut settings = PersistedSettings::default();
        settings
            .insert("global", "turn_timeout_ms".into(), 1_000.into())
            .unwrap();
        settings
            .insert("project", "turn_timeout_ms".into(), 2_000.into())
            .unwrap();
        assert_eq!(settings.layered().unwrap().turn_timeout_ms, Some(2_000));
        settings
            .insert("session", "turn_timeout_ms".into(), 3_000.into())
            .unwrap();
        assert_eq!(settings.layered().unwrap().turn_timeout_ms, Some(3_000));
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
    fn global_view_uses_defaults_for_missing_or_mistyped_values() {
        let defaults = GlobalSettings::from_values(&SettingValues::new());
        assert_eq!(defaults.log, LogSettings::default());
        assert_eq!(defaults.tls, TlsSettings::default());
        assert!(defaults.proxy.mode == HttpProxyMode::System);
        assert!(!defaults.browser_use_enabled && !defaults.computer_use_enabled);

        let values = SettingValues::from([
            ("log_max_bytes".into(), json!("large")),
            ("verify_https_certificates".into(), json!(false)),
            ("certificate_path".into(), json!("  ")),
            ("proxy_bypass".into(), json!(["localhost", 3])),
        ]);
        let settings = GlobalSettings::from_values(&values);
        assert_eq!(settings.log.max_bytes, LogSettings::default().max_bytes);
        assert!(!settings.tls.verify_https_certificates);
        assert_eq!(settings.tls.certificate_path, None);
        assert_eq!(settings.proxy.bypass, vec!["localhost".to_string()]);
    }

    #[test]
    fn proxy_values_round_trip_and_debug_hides_the_password() {
        let mut values = SettingValues::new();
        let proxy = HttpProxyConfiguration {
            mode: HttpProxyMode::Custom,
            url: "http://proxy.test:8080".into(),
            username: "user".into(),
            password: "secret-password".into(),
            bypass: vec!["internal.test".into()],
        };
        values.extend(GlobalSettings::proxy_values(&proxy));
        let settings = GlobalSettings::from_values(&values);
        assert!(settings.proxy == proxy);
        assert!(!format!("{settings:?}").contains("secret-password"));
    }

    #[test]
    fn project_and_session_views_reject_mistyped_values() {
        let mut persisted = PersistedSettings::default();
        persisted
            .insert("global", "default_model".into(), json!("global-model"))
            .unwrap();
        assert_eq!(
            persisted.project().unwrap().default_model.as_deref(),
            Some("global-model")
        );
        persisted
            .insert("project", "default_model".into(), json!(" project-model "))
            .unwrap();
        persisted
            .insert("project", "tool_call_limit".into(), json!(32))
            .unwrap();
        let project = persisted.project().unwrap();
        assert_eq!(project.default_model.as_deref(), Some("project-model"));
        assert_eq!(project.tool_call_limit, Some(32));

        persisted
            .insert("project", "tool_call_limit".into(), json!(0))
            .unwrap();
        assert!(persisted.project().is_err());
        persisted
            .insert("session", "full_control".into(), json!("yes"))
            .unwrap();
        assert_eq!(
            persisted.session().unwrap_err(),
            "session full_control configuration must be a boolean"
        );
    }
}
