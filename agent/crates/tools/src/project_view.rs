//! Bounded project browsing for client file trees and the read-only viewer.

use crate::error::{fatal, retryable};
use crate::scope::safe_relative_path;
use serde_json::{json, Value};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use suncode_common::BusinessError;

const MAX_DIRECTORY_ENTRIES: usize = 1000;
const MAX_VIEWER_BYTES: usize = 4 * 1024 * 1024;

pub(crate) fn list_directory(
    root_path: &Path,
    relative_path: &str,
    max_entries: usize,
) -> Result<Value, BusinessError> {
    let canonical_root = root_path
        .canonicalize()
        .map_err(|_| fatal("project_unavailable", "directory root is unavailable"))?;
    if !canonical_root.is_dir() {
        return Err(fatal(
            "project_unavailable",
            "directory root is not a directory",
        ));
    }
    let relative = if relative_path.trim().is_empty() || relative_path == "." {
        PathBuf::new()
    } else {
        safe_relative_path(relative_path)?
    };
    let candidate = canonical_root.join(&relative);
    let canonical_directory = candidate
        .canonicalize()
        .map_err(|_| fatal("path_unavailable", "directory is unavailable"))?;
    if !canonical_directory.starts_with(&canonical_root) || !canonical_directory.is_dir() {
        return Err(fatal(
            "scope_denied",
            "directory must remain inside its root",
        ));
    }
    let limit = max_entries.clamp(1, MAX_DIRECTORY_ENTRIES);
    let mut entries = fs::read_dir(&canonical_directory)
        .map_err(|_| retryable("project_read_failed", "directory could not be read"))?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let file_type = entry.file_type().ok()?;
            if file_type.is_symlink() {
                return None;
            }
            let kind = if file_type.is_dir() {
                "directory"
            } else if file_type.is_file() {
                "file"
            } else {
                return None;
            };
            let name = entry.file_name().to_string_lossy().into_owned();
            let entry_path = relative
                .join(&name)
                .to_string_lossy()
                .replace(std::path::MAIN_SEPARATOR, "/");
            Some((
                kind == "file",
                name.to_ascii_lowercase(),
                json!({
                    "name": name,
                    "path": entry_path,
                    "kind": kind,
                    "expandable": kind == "directory"
                }),
            ))
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| (left.0, &left.1).cmp(&(right.0, &right.1)));
    let truncated = entries.len() > limit;
    entries.truncate(limit);
    Ok(json!({
        "path": relative.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/"),
        "entries": entries.into_iter().map(|(_, _, value)| value).collect::<Vec<_>>(),
        "truncated": truncated
    }))
}

pub(crate) fn read_text_file(
    root_path: &Path,
    relative_path: &str,
    max_bytes: usize,
) -> Result<Value, BusinessError> {
    let canonical_root = root_path
        .canonicalize()
        .map_err(|_| fatal("project_unavailable", "file root is unavailable"))?;
    if !canonical_root.is_dir() {
        return Err(fatal("project_unavailable", "file root is not a directory"));
    }
    let relative = safe_relative_path(relative_path)?;
    let candidate = canonical_root.join(&relative);
    let metadata = fs::symlink_metadata(&candidate)
        .map_err(|_| fatal("path_unavailable", "file is unavailable"))?;
    if metadata.file_type().is_symlink() {
        return Err(fatal("scope_denied", "symbolic links are not allowed"));
    }
    let canonical_file = candidate
        .canonicalize()
        .map_err(|_| fatal("path_unavailable", "file is unavailable"))?;
    if !canonical_file.starts_with(&canonical_root) {
        return Err(fatal("scope_denied", "file must remain inside its root"));
    }
    if !metadata.is_file() {
        return Err(fatal("not_a_file", "path is not a regular file"));
    }
    let limit = max_bytes.clamp(1, MAX_VIEWER_BYTES);
    if metadata.len() > limit as u64 {
        return Err(fatal(
            "file_too_large",
            format!("file exceeds the {limit} byte viewer limit"),
        ));
    }
    let file = fs::File::open(&canonical_file)
        .map_err(|_| retryable("project_read_failed", "file could not be read"))?;
    let mut bytes = Vec::with_capacity(metadata.len().min(limit as u64) as usize);
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| retryable("project_read_failed", "file could not be read"))?;
    if bytes.len() > limit {
        return Err(fatal(
            "file_too_large",
            format!("file exceeds the {limit} byte viewer limit"),
        ));
    }
    if bytes.contains(&0) {
        return Err(fatal("encoding_unsupported", "file is not UTF-8 text"));
    }
    let content = String::from_utf8(bytes)
        .map_err(|_| fatal("encoding_unsupported", "file is not valid UTF-8 text"))?;
    Ok(json!({
        "path": relative.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/"),
        "bytes": content.len(),
        "content": content
    }))
}
