//! Resolution tests that span bootstrap, environment, and persisted settings.

use super::*;
use serde_json::Value;
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
fn disabled_config_matches_disabled_load() {
    let _guard = environment_lock();
    std::env::set_var("SUNCODE_NON_INTERACTIVE", "not-a-bool");
    let loaded = Config::load_with_environment(EnvironmentSource::Disabled).unwrap();
    let disabled = Config::disabled();
    std::env::remove_var("SUNCODE_NON_INTERACTIVE");
    assert_eq!(disabled.data_dir, loaded.data_dir);
    assert_eq!(disabled.database_path, loaded.database_path);
    assert_eq!(disabled.user_directory, loaded.user_directory);
    assert!(!disabled.non_interactive);
    assert_eq!(disabled.environment_source, EnvironmentSource::Disabled);
}
