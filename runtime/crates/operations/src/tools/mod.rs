//! Operation dispatch is intentionally explicit: one audited entry per tool.

mod apply_patch;
mod artifact;
mod checkpoint_restore;
mod edit;
mod fs_delete;
mod fs_metadata;
mod fs_move;
mod glob;
mod grep;
mod process;
mod project_inspect;
mod read;
mod write;

use super::CoreFailure;
use serde_json::Value;
use std::path::Path;

pub(super) fn dispatch(
    method: &str,
    params: &Value,
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
) -> Option<Result<Value, CoreFailure>> {
    Some(match method {
        "project/inspect" => project_inspect::execute(project_root, params),
        "tool/read" | "fs/read" => read::execute(project_root, params),
        "fs/metadata" => fs_metadata::execute(project_root, params),
        "tool/glob" | "search/glob" => glob::execute(project_root, params),
        "tool/grep" | "search/find" => grep::execute(project_root, params),
        "artifact/read" => artifact::read(checkpoint_root, params),
        "artifact/sweep" => artifact::sweep(checkpoint_root, params),
        "tool/write" | "fs/write" => write::execute(project_root, checkpoint_root, params),
        "tool/edit" | "fs/edit" => edit::execute(project_root, checkpoint_root, params),
        "tool/apply_patch" | "fs/patch" => {
            apply_patch::execute(project_root, checkpoint_root, params)
        }
        "fs/move" => fs_move::execute(project_root, checkpoint_root, params),
        "fs/delete" => fs_delete::execute(project_root, checkpoint_root, params),
        "tool/bash" | "shell/run" | "process/run" => {
            process::run(project_root, checkpoint_root, params)
        }
        "process/start" => process::start(project_root, params),
        "process/status" => process::status(params),
        "operation/cancel" => process::cancel(params),
        "operation/status" => process::operation_status(checkpoint_root, params),
        "operation/reconcile" => process::reconcile(project_root, checkpoint_root, params),
        "core/recovery" => process::recovery(project_root, checkpoint_root),
        "checkpoint/restore" => checkpoint_restore::execute(project_root, checkpoint_root, params),
        _ => return None,
    })
}
