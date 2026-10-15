use crate::arguments::WriteArguments;
use crate::checkpoint;
use crate::error::{fatal, retryable};
use crate::journal::{journal_finish, journal_id, journal_intent, load_journal};
use crate::scope::safe_relative_path;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
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
    if let (Some(checkpoint_root), Some(id)) = (
        checkpoint_root,
        journal_id(
            args.idempotency_key.as_deref(),
            args.operation_id.as_deref(),
        ),
    ) {
        if let Some(existing) = load_journal(checkpoint_root, &id) {
            if existing.status == "succeeded" {
                if let Some(result) = existing.result {
                    return Ok(result);
                }
            }
            if existing.status == "pending" {
                return Err(fatal(
                    "unknown_completion",
                    "operation completion is unknown and must be reconciled",
                ));
            }
        }
    }
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
    let candidate = root.join(safe_relative_path(path)?);
    let candidate_metadata = match fs::symlink_metadata(&candidate) {
        Ok(metadata) => Some(metadata),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(_) => return Err(fatal("path_unavailable", "path is unavailable")),
    };
    if candidate_metadata
        .as_ref()
        .is_some_and(|metadata| metadata.file_type().is_symlink())
    {
        return Err(fatal(
            "scope_denied",
            "writes through symbolic links are not allowed",
        ));
    }
    let existed = candidate_metadata.is_some();
    let current = if existed {
        let canonical = candidate
            .canonicalize()
            .map_err(|_| fatal("path_unavailable", "path is unavailable"))?;
        if !canonical.starts_with(root) {
            return Err(fatal("scope_denied", "path is outside the project"));
        }
        let metadata = fs::metadata(&canonical)
            .map_err(|_| fatal("path_unavailable", "path is unavailable"))?;
        if !metadata.is_file() {
            return Err(fatal("not_a_file", "path is not a regular file"));
        }
        Some(fs::read(canonical).map_err(|_| retryable("read_failed", "file could not be read"))?)
    } else {
        None
    };
    let expected_bytes = match (current.as_ref(), args.expected_base64.as_deref()) {
        (Some(_), Some(value)) => Some(
            STANDARD
                .decode(value)
                .map_err(|_| fatal("invalid_arguments", "expected_base64 is invalid"))?,
        ),
        (Some(_), None) => {
            return Err(fatal(
                "precondition_required",
                "expected_base64 is required for an existing file",
            ))
        }
        (None, Some(_)) => return Err(fatal("conflict", "file appeared before the write")),
        (None, None) => None,
    };
    if let (Some(actual), Some(expected)) = (current.as_ref(), expected_bytes.as_ref()) {
        if actual != expected {
            return Err(fatal("conflict", "file changed since it was read"));
        }
    }
    if let Some(parent) = candidate.parent() {
        ensure_parent_directory(root, parent)?;
    }
    let checkpoint_root = checkpoint_root.ok_or(fatal(
        "checkpoint_unavailable",
        "checkpoint storage is not configured",
    ))?;
    let operation_id = journal_intent(
        checkpoint_root,
        args.idempotency_key.as_deref(),
        args.operation_id.as_deref(),
        "write",
        Some(path),
        current.as_deref(),
        Some(&bytes),
    )?;
    let checkpoint_id =
        checkpoint::capture(checkpoint_root, root, path, current.as_deref(), &bytes)?;
    fs::write(&candidate, &bytes)
        .map_err(|_| retryable("write_failed", "file could not be written"))?;
    let result = json!({"path": path, "bytes": bytes.len(), "created": !existed, "checkpoint_id": checkpoint_id});
    journal_finish(
        checkpoint_root,
        operation_id.as_deref(),
        "succeeded",
        Some(result.clone()),
    );
    Ok(result)
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
