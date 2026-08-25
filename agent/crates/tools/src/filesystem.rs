use super::arguments::ReadArguments;
use super::artifacts::{checkpoint_root_from_env, write_artifact};
use super::{safe_relative_path, BusinessError};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub(super) fn read(
    project_root: Option<&Path>,
    args: &ReadArguments,
) -> Result<Value, BusinessError> {
    let root = project_root.ok_or(
        BusinessError::new("project_unconfigured", "project root is not configured")
            .with_retryable(false),
    )?;
    let path = args.path.as_str();
    let relative = safe_relative_path(path)?;
    let candidate = root.join(relative);
    let canonical = candidate.canonicalize().map_err(|_| {
        BusinessError::new("path_unavailable", "path is unavailable").with_retryable(false)
    })?;
    if !canonical.starts_with(root) {
        return Err(
            BusinessError::new("scope_denied", "path is outside the project").with_retryable(false),
        );
    }
    let metadata = fs::metadata(&canonical).map_err(|_| {
        BusinessError::new("path_unavailable", "path is unavailable").with_retryable(false)
    })?;
    if !metadata.is_file() {
        return Err(
            BusinessError::new("not_a_file", "path is not a regular file").with_retryable(false),
        );
    }
    let offset = positive_integer(args.offset, 1, "offset must be at least 1")?;
    let limit = optional_positive_integer(args.limit, "limit must be at least 1")?;
    let max_bytes = positive_integer(args.max_bytes, 64 * 1024, "max_bytes must be at least 1")?
        .clamp(1, 1024 * 1024) as usize;
    let original = if offset == 1 && limit.is_none() {
        Some(fs::read(&canonical).map_err(|_| {
            BusinessError::new("read_failed", "file could not be read").with_retryable(true)
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

fn read_line_range(path: &Path, offset: u64, limit: Option<u64>) -> Result<Vec<u8>, BusinessError> {
    let file = fs::File::open(path).map_err(|_| {
        BusinessError::new("read_failed", "file could not be read").with_retryable(true)
    })?;
    let mut reader = BufReader::new(file);
    let mut line = Vec::new();
    let mut line_number = 1_u64;
    let mut selected = Vec::new();
    let mut selected_lines = 0_u64;
    loop {
        line.clear();
        let count = reader.read_until(b'\n', &mut line).map_err(|_| {
            BusinessError::new("read_failed", "file could not be read").with_retryable(true)
        })?;
        if count == 0 {
            break;
        }
        if std::str::from_utf8(&line).is_err() {
            return Err(BusinessError::new(
                "encoding_unsupported",
                "offset and limit require UTF-8 text",
            )
            .with_retryable(false));
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
        return Err(BusinessError::new(
            "invalid_arguments",
            "offset is beyond the end of the file",
        )
        .with_retryable(false));
    }
    Ok(selected)
}

fn positive_integer(
    value: Option<u64>,
    default: u64,
    message: &'static str,
) -> Result<u64, BusinessError> {
    let number = value.unwrap_or(default);
    if number == 0 {
        return Err(BusinessError::invalid(message));
    }
    Ok(number)
}

fn optional_positive_integer(
    value: Option<u64>,
    message: &'static str,
) -> Result<Option<u64>, BusinessError> {
    let Some(number) = value else { return Ok(None) };
    if number == 0 {
        return Err(BusinessError::invalid(message));
    }
    Ok(Some(number))
}
