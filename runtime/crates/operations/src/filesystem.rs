use super::artifacts::{checkpoint_root_from_env, write_artifact};
use super::{safe_relative_path, CoreFailure};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub(super) fn read(project_root: Option<&Path>, params: &Value) -> Result<Value, CoreFailure> {
    let root = project_root.ok_or(CoreFailure {
        code: "project_unconfigured",
        message: "project root is not configured",
        retryable: false,
    })?;
    let path = params
        .get("path")
        .and_then(Value::as_str)
        .ok_or(CoreFailure {
            code: "invalid_arguments",
            message: "path is required",
            retryable: false,
        })?;
    let relative = safe_relative_path(path)?;
    let candidate = root.join(relative);
    let canonical = candidate.canonicalize().map_err(|_| CoreFailure {
        code: "path_unavailable",
        message: "path is unavailable",
        retryable: false,
    })?;
    if !canonical.starts_with(root) {
        return Err(CoreFailure {
            code: "scope_denied",
            message: "path is outside the project",
            retryable: false,
        });
    }
    let metadata = fs::metadata(&canonical).map_err(|_| CoreFailure {
        code: "path_unavailable",
        message: "path is unavailable",
        retryable: false,
    })?;
    if !metadata.is_file() {
        return Err(CoreFailure {
            code: "not_a_file",
            message: "path is not a regular file",
            retryable: false,
        });
    }
    let max_bytes = params
        .get("max_bytes")
        .and_then(Value::as_u64)
        .unwrap_or(64 * 1024)
        .clamp(1, 1024 * 1024) as usize;
    let mut bytes = fs::read(&canonical).map_err(|_| CoreFailure {
        code: "read_failed",
        message: "file could not be read",
        retryable: true,
    })?;
    let total_bytes = bytes.len();
    let truncated = total_bytes > max_bytes;
    bytes.truncate(max_bytes);
    if truncated {
        if let Some(root) = checkpoint_root_from_env() {
            let artifact_id = write_artifact(
                &root,
                &fs::read(&canonical).map_err(|_| CoreFailure {
                    code: "read_failed",
                    message: "file could not be read",
                    retryable: true,
                })?,
            )?;
            return Ok(
                json!({"path": path, "bytes": total_bytes, "data_base64": STANDARD.encode(bytes), "truncated": true, "artifact_id": artifact_id}),
            );
        }
    }
    Ok(
        json!({"path": path, "bytes": bytes.len(), "data_base64": STANDARD.encode(bytes), "truncated": truncated}),
    )
}
