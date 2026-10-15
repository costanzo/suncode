mod environment;

use crate::error::mcp_error;
use rmcp::transport::TokioChildProcess;
use std::{collections::BTreeMap, path::Path};
use suncode_common::BusinessError;

pub(crate) fn transport(
    command: String,
    arguments: Vec<String>,
    working_directory: &Path,
    environment: &BTreeMap<String, String>,
) -> Result<TokioChildProcess, BusinessError> {
    let mut process = tokio::process::Command::new(command);
    process
        .args(arguments)
        .current_dir(working_directory)
        .kill_on_drop(true)
        .env_clear()
        .envs(environment::default_environment(
            working_directory,
            environment,
        ));
    TokioChildProcess::new(process)
        .map_err(|error| mcp_error("mcp_process_start_failed", error.to_string()))
}
