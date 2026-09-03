//! Operation dispatch is intentionally explicit: one audited entry per tool.

mod checkpoint_restore;
mod edit;
mod glob;
mod grep;
mod process;
mod read;
mod webfetch;
mod write;

use super::arguments::{
    CheckpointRestoreArguments, EditArguments, GitDiffArguments, GlobArguments, GrepArguments,
    ProcessArguments, ReadArguments, WebfetchArguments, WriteArguments,
};
use super::{BusinessError, ProcessOutputCallback};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::path::Path;
use std::sync::atomic::AtomicBool;

pub(super) fn dispatch_with_output(
    method: &str,
    params: &Value,
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
    cancellation: Option<&AtomicBool>,
    verify_https_certificates: bool,
    output_callback: Option<ProcessOutputCallback>,
    use_system_certificates: bool,
    certificate_path: Option<&Path>,
) -> Option<Result<Value, BusinessError>> {
    Some(match method {
        "tool/read" => run_read_typed(params, project_root),
        "tool/glob" => run_typed(params, |args: GlobArguments| {
            glob::execute(project_root, args)
        }),
        "tool/grep" => run_typed(params, |args: GrepArguments| {
            grep::execute(project_root, args)
        }),
        "tool/webfetch" => run_typed(params, |args: WebfetchArguments| {
            webfetch::execute(
                checkpoint_root,
                args,
                cancellation,
                verify_https_certificates,
                use_system_certificates,
                certificate_path,
            )
        }),
        "tool/write" => run_typed(params, |args: WriteArguments| {
            write::execute(project_root, checkpoint_root, args)
        }),
        "tool/edit" => run_typed(params, |args: EditArguments| {
            edit::execute(project_root, checkpoint_root, args)
        }),
        "tool/bash" => run_typed(params, |args: ProcessArguments| {
            process::run(project_root, checkpoint_root, args, cancellation, output_callback)
        }),
        "checkpoint/restore" => run_typed(params, |args: CheckpointRestoreArguments| {
            checkpoint_restore::execute(project_root, checkpoint_root, args)
        }),
        "git/status" => super::git::status(project_root),
        "git/diff-file" => run_typed(params, |args: GitDiffArguments| {
            super::git::diff_file(project_root, &args)
        }),
        _ => return None,
    })
}

fn run_typed<T, F>(params: &Value, execute: F) -> Result<Value, BusinessError>
where
    T: DeserializeOwned,
    F: FnOnce(T) -> Result<Value, BusinessError>,
{
    let arguments = serde_json::from_value(params.clone()).map_err(|_| {
        BusinessError::new("invalid_arguments", "tool arguments have an invalid shape")
            .with_retryable(false)
    })?;
    execute(arguments)
}

fn run_read_typed(params: &Value, project_root: Option<&Path>) -> Result<Value, BusinessError> {
    // Keep the established field-level error for malformed public read calls.
    if params.get("path").and_then(Value::as_str).is_none() {
        return Err(
            BusinessError::new("invalid_arguments", "path is required").with_retryable(false)
        );
    }
    run_typed(params, |args: ReadArguments| {
        read::execute(project_root, args)
    })
}
