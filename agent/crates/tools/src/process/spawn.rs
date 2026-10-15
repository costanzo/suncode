//! Command construction: argument bounds, working directory, environment, spawn flags.

use crate::arguments::ProcessArguments;
use crate::error::fatal;
use crate::scope::safe_relative_path;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use suncode_common::BusinessError;

#[cfg(target_os = "windows")]
pub(super) const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub(super) fn command_arguments(
    args: &ProcessArguments,
) -> Result<(String, Vec<String>), BusinessError> {
    let program = args.program.as_str();
    if program.is_empty() || program.len() > 4096 {
        return Err(fatal("invalid_arguments", "program is invalid"));
    }
    if args.args.len() > 256 {
        return Err(fatal("resource_limit", "too many process arguments"));
    }
    Ok((program.to_string(), args.args.clone()))
}

pub(super) fn process_cwd(
    root: &Path,
    args: &ProcessArguments,
) -> Result<std::path::PathBuf, BusinessError> {
    let requested = args.cwd.as_deref().unwrap_or(".");
    let requested_path = Path::new(requested);
    let candidate = if requested_path.is_absolute() {
        requested_path.to_path_buf()
    } else {
        root.join(safe_relative_path(requested)?)
    };
    let path = candidate.canonicalize().map_err(|_| {
        fatal(
            "process_working_directory_unavailable",
            "working directory is unavailable",
        )
    })?;
    if !path.starts_with(root) || !path.is_dir() {
        return Err(fatal(
            "scope_denied",
            "working directory is outside the project",
        ));
    }
    Ok(path)
}

pub(super) fn configure_command(mut command: Command, args: &ProcessArguments) -> Command {
    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW);
    #[cfg(unix)]
    command.process_group(0);
    // Tool commands must never read or steal the host's terminal input.
    command.stdin(Stdio::null());
    command.env_clear();
    for key in [
        "PATH",
        "HOME",
        "TMPDIR",
        "TMP",
        "TEMP",
        "SystemRoot",
        "ComSpec",
        "PATHEXT",
        "USERPROFILE",
        "http_proxy",
        "https_proxy",
        "no_proxy",
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "NO_PROXY",
    ] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    for (key, value) in &args.env {
        if key.len() <= 128
            && key
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            command.env(key, value);
        }
    }
    command
}

pub(super) fn process_start_failure(error: std::io::Error) -> BusinessError {
    match error.kind() {
        std::io::ErrorKind::NotFound => fatal(
            "process_executable_not_found",
            "process executable could not be found",
        ),
        std::io::ErrorKind::PermissionDenied => fatal(
            "process_permission_denied",
            "process executable could not be started because permission was denied",
        ),
        _ => fatal("process_start_failed", "process could not be started"),
    }
}
