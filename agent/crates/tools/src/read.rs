use crate::arguments::ReadArguments;
use crate::artifacts::write_artifact;
use crate::error::{fatal, retryable};
use crate::scope::safe_relative_path;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use suncode_common::BusinessError;

/// Bytes returned inline when `max_bytes` is not given.
const DEFAULT_PREVIEW_BYTES: u64 = 64 * 1024;
const MAX_PREVIEW_BYTES: u64 = 1024 * 1024;
/// Upper bound on bytes read into memory; larger files are previewed only.
const MAX_SOURCE_BYTES: u64 = 16 * 1024 * 1024;

pub(crate) fn read(
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
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
    let max_bytes = positive_integer(
        args.max_bytes,
        DEFAULT_PREVIEW_BYTES,
        "max_bytes must be at least 1",
    )?
    .clamp(1, MAX_PREVIEW_BYTES) as usize;
    let (bytes, source_complete) = if offset == 1 && limit.is_none() {
        let mut bytes = Vec::new();
        fs::File::open(&canonical)
            .and_then(|file| file.take(MAX_SOURCE_BYTES).read_to_end(&mut bytes))
            .map_err(|_| retryable("read_failed", "file could not be read"))?;
        let complete = metadata.len() <= MAX_SOURCE_BYTES;
        (bytes, complete)
    } else {
        (read_line_range(&canonical, offset, limit)?, true)
    };
    let total_bytes = if source_complete {
        bytes.len() as u64
    } else {
        metadata.len()
    };
    let preview_len = utf8_preview_len(&bytes, max_bytes);
    let truncated = preview_len < bytes.len() || !source_complete;
    let mut result = json!({
        "path": path,
        "bytes": preview_len,
        "total_bytes": total_bytes,
        "data_base64": STANDARD.encode(&bytes[..preview_len]),
        "truncated": truncated,
        "offset": offset,
        "limit": limit,
    });
    if truncated && source_complete {
        if let Some(checkpoint_root) = checkpoint_root {
            result["artifact_id"] = json!(write_artifact(checkpoint_root, &bytes)?);
        }
    }
    Ok(result)
}

/// The longest prefix of at most `max_bytes` that does not split a UTF-8
/// sequence. Non-UTF-8 content is cut at `max_bytes` unchanged.
fn utf8_preview_len(bytes: &[u8], max_bytes: usize) -> usize {
    if bytes.len() <= max_bytes {
        return bytes.len();
    }
    match std::str::from_utf8(&bytes[..max_bytes]) {
        Ok(_) => max_bytes,
        // An incomplete trailing sequence has no `error_len`; trim back to it.
        Err(error) if error.error_len().is_none() => error.valid_up_to(),
        Err(_) => max_bytes,
    }
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
