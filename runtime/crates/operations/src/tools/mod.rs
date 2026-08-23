//! Operation dispatch is intentionally explicit: one audited entry per tool.

mod checkpoint_restore;
mod edit;
mod glob;
mod grep;
mod process;
mod read;
mod webfetch;
mod write;

use super::CoreFailure;
use serde_json::Value;
use std::path::Path;
use std::sync::atomic::AtomicBool;

pub(super) fn dispatch(
    method: &str,
    params: &Value,
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
    cancellation: Option<&AtomicBool>,
) -> Option<Result<Value, CoreFailure>> {
    Some(match method {
        "tool/read" => read::execute(project_root, params),
        "tool/glob" => glob::execute(project_root, params),
        "tool/grep" => grep::execute(project_root, params),
        "tool/webfetch" => webfetch::execute(checkpoint_root, params, cancellation),
        "tool/write" => write::execute(project_root, checkpoint_root, params),
        "tool/edit" => edit::execute(project_root, checkpoint_root, params),
        "tool/bash" => process::run(project_root, checkpoint_root, params, cancellation),
        "checkpoint/restore" => checkpoint_restore::execute(project_root, checkpoint_root, params),
        _ => return None,
    })
}
