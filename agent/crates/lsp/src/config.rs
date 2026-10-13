//! Language-server launch configuration and its validation.

use serde_json::Value;
use std::{collections::BTreeMap, path::PathBuf, time::Duration};
use suncode_common::BusinessError;

#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    pub command: String,
    pub arguments: Vec<String>,
    pub working_directory: PathBuf,
    pub environment: BTreeMap<String, String>,
    pub initialization_options: Value,
    pub startup_timeout: Duration,
    pub request_timeout: Duration,
}

pub(crate) fn validate_connection_config(config: &ConnectionConfig) -> Result<(), BusinessError> {
    if config.command.trim().is_empty() || config.command.chars().count() > 2_048 {
        return Err(BusinessError::invalid(
            "Language server command is required and must not exceed 2048 characters",
        ));
    }
    if config.arguments.len() > 128
        || config
            .arguments
            .iter()
            .any(|value| value.chars().count() > 8_192)
    {
        return Err(BusinessError::invalid(
            "Language server arguments exceed the supported bounds",
        ));
    }
    if !config.initialization_options.is_null() && !config.initialization_options.is_object() {
        return Err(BusinessError::invalid(
            "Language server initialization options must be a JSON object",
        ));
    }
    if !config.working_directory.is_dir() {
        return Err(BusinessError::invalid(
            "Language server working directory is unavailable",
        ));
    }
    if config.startup_timeout.is_zero() || config.request_timeout.is_zero() {
        return Err(BusinessError::invalid(
            "Language server timeouts must be greater than zero",
        ));
    }
    Ok(())
}
