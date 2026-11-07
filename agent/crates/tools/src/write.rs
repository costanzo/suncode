use crate::arguments::{WriteArguments, DeleteArguments};
use crate::checkpoint;
use crate::error::{fatal, retryable};
use crate::scope::safe_relative_path;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use suncode_common::BusinessError;

pub(crate) fn write(
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
    args: &WriteArguments,
) -> Result<Value, BusinessError> {
    let root = project_root.ok_or(fatal(
        "project_unconfigured",
        "project root is not configured",
    ))?;
    let path = args.path.as_str();
    let content = args.content_base64.as_str();
    let bytes = STANDARD
        .decode(content)
        .map_err(|_| fatal("invalid_arguments", "content_base64 is invalid"))?;
    if bytes.len() > 1024 * 1024 {
        return Err(fatal(
            "output_limit",
            "file content exceeds the write limit",
        ));
    }
    let (relative, candidate, current) = target(root, path)?;
    let existed = current.is_some();
    match (current.as_deref(), &args.expected_base64) {
        (Some(actual), Some(expected)) => check_unchanged(actual, expected)?,
        (Some(_), None) => {
            return Err(fatal(
                "precondition_required",
                "expected_base64 is required for an existing file",
            ))
        }
        (None, Some(_)) => return Err(fatal("conflict", "file appeared before the write")),
        (None, None) => {}
    }
    if let Some(parent) = candidate.parent() {
        ensure_parent_directory(root, parent)?;
    }
    let checkpoint_root = checkpoint_root.ok_or(fatal(
        "checkpoint_unavailable",
        "checkpoint storage is not configured",
    ))?;
    let day = checkpoint::today();
    let baseline_sha256 = current
        .as_deref()
        .map(|bytes| checkpoint::store(checkpoint_root, &day, bytes))
        .transpose()?;
    let current_sha256 = checkpoint::store(checkpoint_root, &day, &bytes)?;
    fs::write(&candidate, &bytes)
        .map_err(|_| retryable("write_failed", "file could not be written"))?;
    // `checkpoint` is runtime metadata; the agent core records it as the
    // turn's file change and strips it before the model sees the result.
    Ok(json!({
        "path": path,
        "bytes": bytes.len(),
        "created": !existed,
        "checkpoint": {
            "path": checkpoint::change_path(&relative),
            "stored_on": day,
            "baseline_sha256": baseline_sha256,
            "current_sha256": current_sha256,
        },
    }))
}

/// Deletes one regular project file whose bytes still match what the model
/// read, keeping its content as the turn change baseline.
pub(crate) fn delete(
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
    args: &DeleteArguments,
) -> Result<Value, BusinessError> {
    let root = project_root.ok_or(fatal(
        "project_unconfigured",
        "project root is not configured",
    ))?;
    let path = args.path.as_str();
    let (relative, candidate, current) = target(root, path)?;
    let current = current.ok_or(fatal("not_found", "file does not exist", ))?;
    check_unchanged(&current, &args.expected_base64)?;
    let checkpoint_root = checkpoint_root.ok_or(fatal(
        "project_unconfigured",
        "checkpoint storage is not configured",
    ))?;
    let day = checkpoint::today();
    let baseline_sha256 = checkpoint::store(checkpoint_root, &day, &current)?;
    fs::remove_file(&candidate)
        .map_err(|_| retryable("delete_failed", "file could not be deleted"))?;
    Ok(json!({
        "path": path,
        "bytes": current.len(),
        "deleted": true,
        "checkpoint": {
            "path": checkpoint::change_path(&relative),
            "stored_on": day,
            "baseline_sha256": baseline_sha256,
            "current_sha256": null,
        },
    }))
}

/// Resolves `path` inside the project without following a final symbolic
/// link, returning the current bytes when it is an existing regular file.
fn target(root: &Path, path: &str) -> Result<(PathBuf, PathBuf, Option<Vec<u8>>), BusinessError> {
    let relative = safe_relative_path(path)?;
    let candidate = root.join(&relative);
    let metadata = match fs::symlink_metadata(&candidate) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Ok((relative, candidate, None))
        },
        Err(_) => return Err(fatal("path_unavailable", "path is unavailable")),
    };
    if metadata.file_type().is_symlink() {
        return Err(fatal(
            "scope_denied",
            "write through symbolic links are not allowed",
        ));
    }
    let canonical = candidate
        .canonicalize()
        .map_err(|_| fatal("path_unavailable", "path is unavailable"))?;
    if !canonical.starts_with(root) {
        return Err(fatal("scope_denied", "path is outside the project root",));
    }
    let metadata =
        fs::metadata(&canonical).map_err(|_| fatal("path_unavailable", "path is unavailable"))?;
    if !metadata.is_file() {
        return Err(fatal("not_a_file", "path is not a regular file", ))
    }
    let current =
        fs::read(canonical).map_err(|_| fatal("read_failed", "file content could not be read"))?;
    Ok((relative, candidate, Some(current)))
}

fn check_unchanged(actual: &[u8], expected_base64: &str) -> Result<(), BusinessError> {
    let expected = STANDARD
        .decode(expected_base64)
        .map_err(|_| fatal("invalid_arguments", "expected_base64 is invalid"))?;
    if actual != expected {
        return Err(fatal("write_conflict", "file content has changed since the last read", ));
    }
    Ok(())
}

fn ensure_parent_directory(root: &Path, parent: &Path) -> Result<(), BusinessError> {
    let mut existing = parent.to_path_buf();
    while !existing.exists() {
        if !existing.pop() {
            return Err(fatal("path_unavailable", "parent directory is unavailable"));
        }
    }
    let canonical_existing = existing
        .canonicalize()
        .map_err(|_| fatal("path_unavailable", "parent directory is unavailable"))?;
    if !canonical_existing.starts_with(root) {
        return Err(fatal(
            "scope_denied",
            "parent directory is outside the project",
        ));
    }
    fs::create_dir_all(parent)
        .map_err(|_| retryable("write_failed", "parent directory could not be created"))?;
    let canonical_parent = parent
        .canonicalize()
        .map_err(|_| fatal("path_unavailable", "parent directory is unavailable"))?;
    if !canonical_parent.starts_with(root) {
        return Err(fatal(
            "scope_denied",
            "parent directory is outside the project",
        ));
    }
    Ok(())
}
