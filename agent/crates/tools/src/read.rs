use crate::arguments::ReadArguments;
use crate::artifacts::{checkpoint_root_from_env, write_artifact};
use crate::error::{fatal, retryable};
use crate::scope::safe_relative_path;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use suncode_common::BusinessError;

pub(crate) fn read(
    project_root: Option<&Path>,
    args: &ReadArguments,
) -> Result<Value, BusinessError> {
    let root = project_root.ok_or(fatal(
        "project_unconfigured",
        "project root is not configured",
    ))?;
    let path = args.path.as_str();
    let relative = safe_relative_path(path)?;
    let candidate = root.join(relative);
    let canonical = candidate
        .canonicalize()
        .map_err(|_| fatal("path_unavailable", "path is unavailable"))?;
    if !canonical.starts_with(root) {
        return Err(fatal("scope_denied", "path is outside the project"));
    }
    let metadata =
        fs::metadata(&canonical).map_err(|_| fatal("path_unavailable", "path is unavailable"))?;
    if !metadata.is_file() {
        return Err(fatal("not_a_file", "path is not a regular file"));
    }
    let offset = positive_integer(args.offset, 1, "offset must be at least 1")?;
    let limit = optional_positive_integer(args.limit, "limit must be at least 1")?;
    let max_bytes = positive_integer(args.max_bytes, 64 * 1024, "max_bytes must be at least 1")?
        .clamp(1, 1024 * 1024) as usize;
    let original = if offset == 1 && limit.is_none() {
        Some(fs::read(&canonical).map_err(|_| retryable("read_failed", "file could not be read"))?)
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
    let file =
        fs::File::open(path).map_err(|_| retryable("read_failed", "file could not be read"))?;
    let mut reader = BufReader::new(file);
    let mut line = Vec::new();
    let mut line_number = 1_u64;
    let mut selected = Vec::new();
    let mut selected_lines = 0_u64;
    loop {
        line.clear();
        let count = reader
            .read_until(b'\n', &mut line)
            .map_err(|_| retryable("read_failed", "file could not be read"))?;
        if count == 0 {
            break;
        }
        if std::str::from_utf8(&line).is_err() {
            return Err(fatal(
                "encoding_unsupported",
                "offset and limit require UTF-8 text",
            ));
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
        return Err(fatal(
            "invalid_arguments",
            "offset is beyond the end of the file",
        ));
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
