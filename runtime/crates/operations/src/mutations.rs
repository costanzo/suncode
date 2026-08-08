use super::{checkpoint, existing_file, require_project, safe_relative_path, write, CoreFailure};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

fn verify_expected(current: &[u8], params: &Value) -> Result<(), CoreFailure> {
    let expected = params
        .get("expected_base64")
        .and_then(Value::as_str)
        .ok_or(CoreFailure {
            code: "precondition_required",
            message: "expected_base64 is required",
            retryable: false,
        })?;
    let bytes = STANDARD.decode(expected).map_err(|_| CoreFailure {
        code: "invalid_arguments",
        message: "expected_base64 is invalid",
        retryable: false,
    })?;
    if current != bytes {
        return Err(CoreFailure {
            code: "conflict",
            message: "file changed since it was read",
            retryable: false,
        });
    }
    Ok(())
}

pub(super) fn edit(
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
    params: &Value,
) -> Result<Value, CoreFailure> {
    let root = require_project(project_root)?;
    let path = params
        .get("path")
        .and_then(Value::as_str)
        .ok_or(CoreFailure {
            code: "invalid_arguments",
            message: "path is required",
            retryable: false,
        })?;
    let (_, current) = existing_file(root, path)?;
    verify_expected(&current, params)?;
    let mut text = String::from_utf8(current.clone()).map_err(|_| CoreFailure {
        code: "encoding_unsupported",
        message: "edit requires UTF-8 text",
        retryable: false,
    })?;
    let replacements = params
        .get("replacements")
        .and_then(Value::as_array)
        .ok_or(CoreFailure {
            code: "invalid_arguments",
            message: "replacements is required",
            retryable: false,
        })?;
    if replacements.len() > 200 {
        return Err(CoreFailure {
            code: "resource_limit",
            message: "too many replacements",
            retryable: false,
        });
    }
    for replacement in replacements {
        let object = replacement.as_object().ok_or(CoreFailure {
            code: "invalid_arguments",
            message: "replacement must be an object",
            retryable: false,
        })?;
        let old = object
            .get("old")
            .and_then(Value::as_str)
            .ok_or(CoreFailure {
                code: "invalid_arguments",
                message: "replacement old text is required",
                retryable: false,
            })?;
        let new = object
            .get("new")
            .and_then(Value::as_str)
            .ok_or(CoreFailure {
                code: "invalid_arguments",
                message: "replacement new text is required",
                retryable: false,
            })?;
        if old.is_empty() {
            return Err(CoreFailure {
                code: "invalid_arguments",
                message: "replacement old text cannot be empty",
                retryable: false,
            });
        }
        let replace_all = object
            .get("replace_all")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if !text.contains(old) {
            return Err(CoreFailure {
                code: "edit_conflict",
                message: "replacement text was not found",
                retryable: false,
            });
        }
        text = if replace_all {
            text.replace(old, new)
        } else {
            text.replacen(old, new, 1)
        };
    }
    let mut write_params = json!({"path": path, "content_base64": STANDARD.encode(text.as_bytes()), "expected_base64": STANDARD.encode(current)});
    if let Some(key) = params.get("idempotency_key") {
        write_params["idempotency_key"] = key.clone();
    }
    let mut result = write::write(project_root, checkpoint_root, &write_params)?;
    result["operation"] = json!("fs.edit");
    Ok(result)
}

pub(super) fn patch(
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
    params: &Value,
) -> Result<Value, CoreFailure> {
    let root = require_project(project_root)?;
    let path = params
        .get("path")
        .and_then(Value::as_str)
        .ok_or(CoreFailure {
            code: "invalid_arguments",
            message: "path is required",
            retryable: false,
        })?;
    let patch = params
        .get("patch")
        .and_then(Value::as_str)
        .ok_or(CoreFailure {
            code: "invalid_arguments",
            message: "patch is required",
            retryable: false,
        })?;
    let (_, current) = existing_file(root, path)?;
    verify_expected(&current, params)?;
    let text = String::from_utf8(current.clone()).map_err(|_| CoreFailure {
        code: "encoding_unsupported",
        message: "patch requires UTF-8 text",
        retryable: false,
    })?;
    let (old_block, new_block) = parse_patch_blocks(patch)?;
    if old_block.is_empty() || !text.contains(&old_block) {
        return Err(CoreFailure {
            code: "patch_conflict",
            message: "patch context does not match the current file",
            retryable: false,
        });
    }
    let updated = text.replacen(&old_block, &new_block, 1);
    let write_params = json!({"path": path, "content_base64": STANDARD.encode(updated.as_bytes()), "expected_base64": STANDARD.encode(current)});
    let mut result = write::write(project_root, checkpoint_root, &write_params)?;
    result["operation"] = json!("fs.patch");
    Ok(result)
}

fn parse_patch_blocks(patch: &str) -> Result<(String, String), CoreFailure> {
    let mut old_lines = Vec::new();
    let mut new_lines = Vec::new();
    for line in patch.lines() {
        if line.starts_with("---")
            || line.starts_with("+++")
            || line.starts_with("@@")
            || line == "\\ No newline at end of file"
        {
            continue;
        }
        if let Some(value) = line.strip_prefix('-') {
            old_lines.push(value);
        } else if let Some(value) = line.strip_prefix('+') {
            new_lines.push(value);
        } else if let Some(value) = line.strip_prefix(' ') {
            old_lines.push(value);
            new_lines.push(value);
        }
    }
    if old_lines.is_empty() && new_lines.is_empty() {
        return Err(CoreFailure {
            code: "invalid_arguments",
            message: "patch has no changes",
            retryable: false,
        });
    }
    Ok((old_lines.join("\n"), new_lines.join("\n")))
}

pub(super) fn move_file(
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
    params: &Value,
) -> Result<Value, CoreFailure> {
    let root = require_project(project_root)?;
    let from = params
        .get("from")
        .and_then(Value::as_str)
        .ok_or(CoreFailure {
            code: "invalid_arguments",
            message: "from is required",
            retryable: false,
        })?;
    let to = params
        .get("to")
        .and_then(Value::as_str)
        .ok_or(CoreFailure {
            code: "invalid_arguments",
            message: "to is required",
            retryable: false,
        })?;
    let (source, source_bytes) = existing_file(root, from)?;
    verify_expected(&source_bytes, params)?;
    let destination = root.join(safe_relative_path(to)?);
    if fs::symlink_metadata(&destination).is_ok() {
        return Err(CoreFailure {
            code: "conflict",
            message: "destination already exists",
            retryable: false,
        });
    }
    let parent = destination
        .parent()
        .ok_or(CoreFailure {
            code: "path_unavailable",
            message: "destination parent is unavailable",
            retryable: false,
        })?
        .canonicalize()
        .map_err(|_| CoreFailure {
            code: "path_unavailable",
            message: "destination parent is unavailable",
            retryable: false,
        })?;
    if !parent.starts_with(root) {
        return Err(CoreFailure {
            code: "scope_denied",
            message: "destination is outside the project",
            retryable: false,
        });
    }
    let checkpoints = checkpoint_root.ok_or(CoreFailure {
        code: "checkpoint_unavailable",
        message: "checkpoint storage is not configured",
        retryable: false,
    })?;
    let source_checkpoint =
        checkpoint::capture_state(checkpoints, root, from, Some(&source_bytes), None)?;
    let destination_checkpoint =
        checkpoint::capture_state(checkpoints, root, to, None, Some(&source_bytes))?;
    fs::rename(&source, &destination).map_err(|_| CoreFailure {
        code: "move_failed",
        message: "file could not be moved",
        retryable: true,
    })?;
    Ok(
        json!({"operation": "fs.move", "from": from, "to": to, "checkpoint_ids": [source_checkpoint, destination_checkpoint]}),
    )
}

pub(super) fn delete(
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
    params: &Value,
) -> Result<Value, CoreFailure> {
    let root = require_project(project_root)?;
    let path = params
        .get("path")
        .and_then(Value::as_str)
        .ok_or(CoreFailure {
            code: "invalid_arguments",
            message: "path is required",
            retryable: false,
        })?;
    let (canonical, current) = existing_file(root, path)?;
    verify_expected(&current, params)?;
    let checkpoints = checkpoint_root.ok_or(CoreFailure {
        code: "checkpoint_unavailable",
        message: "checkpoint storage is not configured",
        retryable: false,
    })?;
    let checkpoint_id = checkpoint::capture_state(checkpoints, root, path, Some(&current), None)?;
    fs::remove_file(canonical).map_err(|_| CoreFailure {
        code: "delete_failed",
        message: "file could not be deleted",
        retryable: true,
    })?;
    Ok(
        json!({"operation": "fs.delete", "path": path, "deleted": true, "checkpoint_id": checkpoint_id}),
    )
}
