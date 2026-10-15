//! Process environment access, gated by the host-selected source.

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnvironmentSource {
    #[default]
    Disabled,
    Process,
}

impl EnvironmentSource {
    fn enabled(self) -> bool {
        self == Self::Process
    }

    pub(crate) fn value(self, name: &str) -> Option<String> {
        self.enabled().then(|| std::env::var(name).ok()).flatten()
    }

    /// Like `value`, but treats empty and non-UTF-8 values as absent.
    pub(crate) fn non_empty_value(self, name: &str) -> Option<String> {
        self.enabled()
            .then(|| std::env::var_os(name))
            .flatten()
            .and_then(|value| value.into_string().ok())
            .filter(|value| !value.is_empty())
    }

    pub(crate) fn path(self, name: &str) -> Option<PathBuf> {
        self.enabled()
            .then(|| std::env::var_os(name).map(PathBuf::from))
            .flatten()
    }

    pub(crate) fn bool(self, name: &str) -> Result<Option<bool>, String> {
        self.value(name)
            .map(|value| parse_bool(&value).ok_or_else(|| format!("{name} must be true or false")))
            .transpose()
    }

    pub(crate) fn u64(self, name: &str) -> Result<Option<u64>, String> {
        self.value(name)
            .map(|value| {
                value
                    .parse()
                    .map_err(|_| format!("{name} must be an integer"))
            })
            .transpose()
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "1" | "true" => Some(true),
        "0" | "false" => Some(false),
        _ => None,
    }
}
