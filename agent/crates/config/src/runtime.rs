//! Per-turn runtime policy resolved from the environment and persisted settings.

use crate::{environment::EnvironmentSource, settings::PersistedSettings};
use std::{collections::BTreeSet, ops::RangeInclusive};

const TURN_TIMEOUT_RANGE_MS: RangeInclusive<u64> = 1_000..=86_400_000;
const BASH_TIMEOUT_RANGE_MS: RangeInclusive<u64> = 1..=600_000;
const TOOL_CALL_LIMIT_RANGE: RangeInclusive<u64> = 1..=256;
const MAX_TOOL_NAME_BYTES: usize = 128;

#[derive(Clone, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub full_control: Option<bool>,
    pub tool_allowlist: Option<BTreeSet<String>>,
    pub turn_timeout_ms: u64,
    pub bash_timeout_ms: Option<u64>,
    pub tool_call_limit: Option<u32>,
}

impl RuntimeConfig {
    pub const DEFAULT_TURN_TIMEOUT_MS: u64 = 600_000;

    /// Environment values override persisted settings; persisted settings
    /// prefer session, then project, then global scope.
    pub(crate) fn resolve(
        source: EnvironmentSource,
        persisted: &PersistedSettings,
    ) -> Result<Self, String> {
        let overrides = persisted.overrides();
        let full_control = match source.bool("SUNCODE_FULL_CONTROL")? {
            Some(value) => Some(value),
            None => persisted.session.full_control,
        };
        let tool_allowlist = source
            .value("SUNCODE_TOOL_ALLOWLIST")
            .or(overrides.tool_allowlist)
            .map(|value| parse_allowlist(&value))
            .transpose()?;
        let turn_timeout_ms = source
            .u64("SUNCODE_TURN_TIMEOUT_MS")?
            .or(overrides.turn_timeout_ms)
            .unwrap_or(Self::DEFAULT_TURN_TIMEOUT_MS);
        validate_range("turn timeout", turn_timeout_ms, TURN_TIMEOUT_RANGE_MS)?;
        let bash_timeout_ms = source
            .u64("SUNCODE_BASH_TIMEOUT_MS")?
            .or(overrides.bash_timeout_ms)
            .map(|value| validate_range("bash timeout", value, BASH_TIMEOUT_RANGE_MS))
            .transpose()?;
        let tool_call_limit = source
            .u64("SUNCODE_TOOL_CALL_LIMIT")?
            .or(persisted.project.tool_call_limit.map(u64::from))
            .map(|value| {
                validate_range("tool call limit", value, TOOL_CALL_LIMIT_RANGE).and_then(|v| {
                    u32::try_from(v).map_err(|_| "tool call limit is invalid".to_owned())
                })
            })
            .transpose()?;
        Ok(Self {
            full_control,
            tool_allowlist,
            turn_timeout_ms,
            bash_timeout_ms,
            tool_call_limit,
        })
    }
}

// Credentials are secrets, so Debug lists only their names.
impl std::fmt::Debug for RuntimeConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeConfig")
            .field("full_control", &self.full_control)
            .field("tool_allowlist", &self.tool_allowlist)
            .field("turn_timeout_ms", &self.turn_timeout_ms)
            .field("bash_timeout_ms", &self.bash_timeout_ms)
            .field("tool_call_limit", &self.tool_call_limit)
            .finish()
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
        .any(|v| v.len() > MAX_TOOL_NAME_BYTES || v.chars().any(char::is_control))
    {
        return Err("SUNCODE_TOOL_ALLOWLIST contains an invalid tool name".into());
    }
    Ok(values)
}

fn validate_range(name: &str, value: u64, range: RangeInclusive<u64>) -> Result<u64, String> {
    if range.contains(&value) {
        Ok(value)
    } else {
        Err(format!(
            "{name} must be between {} and {}",
            range.start(),
            range.end()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_output_hides_credential_values() {
        let config = RuntimeConfig {
            full_control: None,
            tool_allowlist: None,
            turn_timeout_ms: RuntimeConfig::DEFAULT_TURN_TIMEOUT_MS,
            bash_timeout_ms: None,
            tool_call_limit: None,
        };
        let rendered = format!("{config:?}");
        assert!(rendered.contains("RuntimeConfig"));
        assert!(!rendered.contains("sk-secret-value"));
    }
}
