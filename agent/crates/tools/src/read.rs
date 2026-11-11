use crate::arguments::ReadArguments;
use crate::artifacts::write_artifact;
use crate::error::{fatal, retryable};
use crate::scope::safe_relative_path;
use crate::support::sha256_hex;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use suncode_common::BusinessError;

/// Bytes returned inline when `max_bytes` is not given.
const DEFAULT_PREVIEW_BYTES: u64 = 64 * 1024;
const MAX_PREVIEW_BYTES: u64 = 1024 * 1024;
/// Maximum retained source bytes; one extra byte detects truncation.
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
    let mut raw = Vec::new();
    fs::File::open(&canonical)
        .and_then(|file| file.take(MAX_SOURCE_BYTES + 1).read_to_end(&mut raw))
        .map_err(|_| retryable("read_failed", "file could not be read"))?;
    let source_complete = raw.len() as u64 <= MAX_SOURCE_BYTES;
    let source_sha256 = source_complete.then(|| sha256_hex(&raw));
    raw.truncate(MAX_SOURCE_BYTES as usize);
    let utf16 = raw.starts_with(&[0xff, 0xfe]);
    if !source_complete && utf16 {
        return Err(fatal(
            "resource_limit",
            "UTF-16LE reads require a source no larger than 16 MiB",
        ));
    }
    let decoded = if utf16 {
        crate::text::decode(&raw)?.0.into_bytes()
    } else {
        raw
    };
    let ranged = offset != 1 || limit.is_some();
    let bytes = if !ranged {
        decoded
    } else if source_complete {
        read_line_range(&decoded, offset, limit)?
    } else {
        read_large_line_range(&canonical, offset, limit)?
    };
    let selected_complete = source_complete || ranged;
    let total_bytes = if selected_complete {
        bytes.len() as u64
    } else {
        metadata.len()
    };
    let preview_len = utf8_preview_len(&bytes, max_bytes);
    let truncated = preview_len < bytes.len() || !selected_complete;
    let mut result = json!({
        "path": path,
        "bytes": preview_len,
        "total_bytes": total_bytes,
        "data_base64": STANDARD.encode(&bytes[..preview_len]),
        "truncated": truncated,
        "offset": offset,
        "limit": limit,
    });
    if offset == 1 && limit.is_none() && source_complete {
        result["sha256"] = json!(source_sha256);
    }
    // The digest describes the raw source even when only a line range or
    // preview was shown. It is observation metadata, not proof of a full read.
    if let Some(digest) = source_sha256 {
        result["source_sha256"] = json!(digest);
    }
    if utf16 {
        result["encoding"] = json!("utf-16le");
    }
    if truncated && selected_complete {
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

fn read_line_range(
    bytes: &[u8],
    offset: u64,
    limit: Option<u64>,
) -> Result<Vec<u8>, BusinessError> {
    let text = std::str::from_utf8(bytes).map_err(|_| {
        fatal(
            "encoding_unsupported",
            "offset and limit require UTF-8 or BOM-marked UTF-16LE text",
        )
    })?;
    let mut lines = text.split_inclusive('\n');
    for _ in 1..offset {
        if lines.next().is_none() {
            return Err(fatal(
                "invalid_arguments",
                "offset is beyond the end of the file",
            ));
        }
    }
    Ok(lines
        .take(limit.unwrap_or(u64::MAX).min(usize::MAX as u64) as usize)
        .collect::<String>()
        .into_bytes())
}

/// Preserve line-range access to large UTF-8 files without buffering an
/// unbounded skipped line or claiming a comparable whole-file observation.
fn read_large_line_range(
    path: &Path,
    offset: u64,
    limit: Option<u64>,
) -> Result<Vec<u8>, BusinessError> {
    let mut reader = BufReader::new(
        fs::File::open(path).map_err(|_| fatal("read_failed", "file could not be read"))?,
    );
    let mut line = 1u64;
    let end_line = limit.map(|limit| offset.saturating_add(limit));
    let mut selected = Vec::new();
    loop {
        let buffer = reader
            .fill_buf()
            .map_err(|_| fatal("read_failed", "file could not be read"))?;
        if buffer.is_empty() || end_line.is_some_and(|end| line >= end) {
            break;
        }
        let count = buffer
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(buffer.len(), |index| index + 1);
        if line >= offset {
            if selected.len() + count > MAX_SOURCE_BYTES as usize {
                return Err(fatal(
                    "resource_limit",
                    "selected line range exceeds 16 MiB; request fewer lines",
                ));
            }
            selected.extend_from_slice(&buffer[..count]);
        }
        if buffer[count - 1] == b'\n' {
            line += 1;
        }
        reader.consume(count);
    }
    if line < offset {
        return Err(fatal(
            "invalid_arguments",
            "offset is beyond the end of the file",
        ));
    }
    std::str::from_utf8(&selected)
        .map_err(|_| fatal("encoding_unsupported", "line ranges require valid text"))?;
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
