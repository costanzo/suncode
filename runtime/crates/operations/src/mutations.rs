use super::{existing_file, require_project, write, CoreFailure};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
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
    result["operation"] = json!("edit");
    Ok(result)
}
