//! Persisted settings layered by scope.

use serde_json::Value;
use std::{collections::BTreeMap, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SettingScope {
    Global,
    Project,
    Session,
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

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PersistedSettings {
    pub global: BTreeMap<String, Value>,
    pub project: BTreeMap<String, Value>,
    pub session: BTreeMap<String, Value>,
}

impl PersistedSettings {
    pub fn insert(&mut self, scope: &str, key: String, value: Value) -> Result<(), String> {
        self.scope_mut(scope.parse()?).insert(key, value);
        Ok(())
    }

    fn scope_mut(&mut self, scope: SettingScope) -> &mut BTreeMap<String, Value> {
        match scope {
            SettingScope::Global => &mut self.global,
            SettingScope::Project => &mut self.project,
            SettingScope::Session => &mut self.session,
        }
    }

    /// Scopes from most to least specific.
    fn layers(&self) -> [&BTreeMap<String, Value>; 3] {
        [&self.session, &self.project, &self.global]
    }

    /// The most specific value for `key`.
    pub(crate) fn get(&self, key: &str) -> Option<&Value> {
        self.layers().into_iter().find_map(|layer| layer.get(key))
    }

    /// The most specific string value for `key`; non-string values are skipped.
    pub(crate) fn string(&self, key: &str) -> Option<String> {
        self.layers()
            .into_iter()
            .find_map(|layer| layer.get(key).and_then(Value::as_str))
            .map(str::to_string)
    }

    pub(crate) fn bool(&self, key: &str) -> Result<Option<bool>, String> {
        self.get(key)
            .map(|value| {
                value
                    .as_bool()
                    .ok_or_else(|| format!("{key} must be a boolean"))
            })
            .transpose()
    }

    pub(crate) fn u64(&self, key: &str) -> Result<Option<u64>, String> {
        self.get(key)
            .map(|value| {
                value
                    .as_u64()
                    .ok_or_else(|| format!("{key} must be an integer"))
            })
            .transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        settings.insert("global", "key".into(), 1.into()).unwrap();
        settings.insert("project", "key".into(), 2.into()).unwrap();
        assert_eq!(settings.u64("key"), Ok(Some(2)));
        settings.insert("session", "key".into(), 3.into()).unwrap();
        assert_eq!(settings.u64("key"), Ok(Some(3)));
    }
}
