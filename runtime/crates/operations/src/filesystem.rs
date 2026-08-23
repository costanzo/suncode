use super::artifacts::{checkpoint_root_from_env, write_artifact};
use super::{safe_relative_path, CoreFailure};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::fs;
use std::io::{BufRead, BufReader};
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
    let offset = positive_integer(params, "offset", 1, "offset must be at least 1")?;
    let limit = optional_positive_integer(params, "limit", "limit must be at least 1")?;
    let max_bytes = positive_integer(
        params,
        "max_bytes",
        64 * 1024,
        "max_bytes must be at least 1",
    )?
    .clamp(1, 1024 * 1024) as usize;
    let original = if offset == 1 && limit.is_none() {
        Some(fs::read(&canonical).map_err(|_| CoreFailure {
            code: "read_failed",
            message: "file could not be read",
            retryable: true,
        })?)
    } else {
        None
    };
    let mut bytes = match original {
        Some(bytes) => bytes,
        None => read_line_range(&canonical, offset, limit)?,
    };
    let total_bytes = bytes.len();
    let truncated = total_bytes > max_bytes;
    let full_bytes = bytes.clone();
    bytes.truncate(max_bytes);
    if truncated {
        if let Some(root) = checkpoint_root_from_env() {
            let artifact_id = write_artifact(&root, &full_bytes)?;
            return Ok(
                json!({"path": path, "bytes": total_bytes, "total_bytes": total_bytes, "data_base64": STANDARD.encode(bytes), "truncated": true, "offset": offset, "limit": limit, "artifact_id": artifact_id}),
            );
        }
    }
    Ok(
        json!({"path": path, "bytes": bytes.len(), "total_bytes": total_bytes, "data_base64": STANDARD.encode(bytes), "truncated": truncated, "offset": offset, "limit": limit}),
    )
}

fn read_line_range(path: &Path, offset: u64, limit: Option<u64>) -> Result<Vec<u8>, CoreFailure> {
    let file = fs::File::open(path).map_err(|_| CoreFailure {
        code: "read_failed",
        message: "file could not be read",
        retryable: true,
    })?;
    let mut reader = BufReader::new(file);
    let mut line = Vec::new();
    let mut line_number = 1_u64;
    let mut selected = Vec::new();
    let mut selected_lines = 0_u64;
    loop {
        line.clear();
        let count = reader
            .read_until(b'\n', &mut line)
            .map_err(|_| CoreFailure {
                code: "read_failed",
                message: "file could not be read",
                retryable: true,
            })?;
        if count == 0 {
            break;
        }
        if std::str::from_utf8(&line).is_err() {
            return Err(CoreFailure {
                code: "encoding_unsupported",
                message: "offset and limit require UTF-8 text",
                retryable: false,
            });
        }
        if line_number >= offset {
            if limit.is_none_or(|value| selected_lines < value) {
                selected.extend_from_slice(&line);
                selected_lines += 1;
            }
            if limit.is_some_and(|value| selected_lines >= value) {
                break;
            }
        }
        line_number += 1;
    }
    if line_number < offset && selected_lines == 0 {
        return Err(CoreFailure {
            code: "invalid_arguments",
            message: "offset is beyond the end of the file",
            retryable: false,
        });
    }
    Ok(selected)
}

fn positive_integer(
    params: &Value,
    key: &str,
    default: u64,
    message: &'static str,
) -> Result<u64, CoreFailure> {
    let Some(value) = params.get(key) else {
        return Ok(default);
    };
    let number = value.as_i64().ok_or(CoreFailure {
        code: "invalid_arguments",
        message,
        retryable: false,
    })?;
    if number <= 0 {
        return Err(CoreFailure {
            code: "invalid_arguments",
            message,
            retryable: false,
        });
    }
    Ok(number as u64)
}

fn optional_positive_integer(
    params: &Value,
    key: &str,
    message: &'static str,
) -> Result<Option<u64>, CoreFailure> {
    let Some(value) = params.get(key) else {
        return Ok(None);
    };
    let number = value.as_i64().ok_or(CoreFailure {
        code: "invalid_arguments",
        message,
        retryable: false,
    })?;
    if number <= 0 {
        return Err(CoreFailure {
            code: "invalid_arguments",
            message,
            retryable: false,
        });
    }
    Ok(Some(number as u64))
}
