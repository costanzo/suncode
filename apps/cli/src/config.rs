#[cfg(test)]
use std::sync::{Mutex, OnceLock};

use clap::ValueEnum;

use crate::{
    args::{Cli, ColorMode, OutputMode},
    error::CliError,
};

#[cfg(test)]
static ENVIRONMENT: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliConfig {
    pub output: OutputMode,
    pub color: ColorMode,
    pub user_id: String,
}

impl CliConfig {
    pub fn resolve(cli: &Cli) -> Result<Self, CliError> {
        let output = resolve_enum(cli.output, "SUNCODE_OUTPUT", OutputMode::Text)?;
        let color = resolve_enum(cli.color, "SUNCODE_COLOR", ColorMode::Auto)?;
        let user_id = cli
            .user_id
            .clone()
            .or_else(|| std::env::var("SUNCODE_USER_ID").ok())
            .unwrap_or_else(default_user_id);
        validate_user_id(&user_id)?;
        Ok(Self {
            output,
            color,
            user_id,
        })
    }
}

fn resolve_enum<T>(explicit: Option<T>, name: &str, fallback: T) -> Result<T, CliError>
where
    T: ValueEnum + Copy,
{
    if let Some(value) = explicit {
        return Ok(value);
    }
    let Ok(value) = std::env::var(name) else {
        return Ok(fallback);
    };
    T::from_str(&value, true).map_err(|_| {
        CliError::invalid(format!(
            "{name} must be one of: {}",
            T::value_variants()
                .iter()
                .filter_map(ValueEnum::to_possible_value)
                .map(|value| value.get_name().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    })
}

fn validate_user_id(value: &str) -> Result<(), CliError> {
    if value.trim().is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
        return Err(CliError::invalid("SUNCODE_USER_ID or --user-id is invalid"));
    }
    Ok(())
}

fn default_user_id() -> String {
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".into());
    format!("os:{user}")
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn explicit_options_override_suncode_environment() {
        let _guard = ENVIRONMENT.get_or_init(|| Mutex::new(())).lock().unwrap();
        std::env::set_var("SUNCODE_OUTPUT", "text");
        std::env::set_var("SUNCODE_COLOR", "always");
        std::env::set_var("SUNCODE_USER_ID", "environment-user");
        let cli = Cli::try_parse_from([
            "suncode",
            "--output",
            "jsonl",
            "--color",
            "never",
            "--user-id",
            "explicit-user",
            "models",
        ])
        .unwrap();

        let resolved = CliConfig::resolve(&cli).unwrap();
        assert_eq!(resolved.output, OutputMode::Jsonl);
        assert_eq!(resolved.color, ColorMode::Never);
        assert_eq!(resolved.user_id, "explicit-user");

        std::env::remove_var("SUNCODE_OUTPUT");
        std::env::remove_var("SUNCODE_COLOR");
        std::env::remove_var("SUNCODE_USER_ID");
    }

    #[test]
    fn invalid_suncode_environment_fails_closed() {
        let _guard = ENVIRONMENT.get_or_init(|| Mutex::new(())).lock().unwrap();
        std::env::set_var("SUNCODE_OUTPUT", "xml");
        let cli = Cli::try_parse_from(["suncode", "doctor"]).unwrap();

        let error = CliConfig::resolve(&cli).unwrap_err();
        assert_eq!(error.exit_code, 2);

        std::env::remove_var("SUNCODE_OUTPUT");
    }
}
