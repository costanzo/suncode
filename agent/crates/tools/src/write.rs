use super::arguments::WriteArguments;
use super::{
    checkpoint, journal_finish, journal_id, journal_intent, load_journal, safe_relative_path,
    BusinessError,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

pub(super) fn write(
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
    args: &WriteArguments,
) -> Result<Value, BusinessError> {
    let root = project_root.ok_or(
        BusinessError::new("project_unconfigured", "project root is not configured")
            .with_retryable(false),
    )?;
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
                return Err(BusinessError::new(
                    "unknown_completion",
                    "operation completion is unknown and must be reconciled",
                )
                .with_retryable(false));
            }
        }
    }
    let content = args.content_base64.as_str();
    let bytes = STANDARD.decode(content).map_err(|_| {
        BusinessError::new("invalid_arguments", "content_base64 is invalid").with_retryable(false)
    })?;
    if bytes.len() > 1024 * 1024 {
        return Err(
            BusinessError::new("output_limit", "file content exceeds the write limit")
                .with_retryable(false),
        );
    }
    let candidate = root.join(safe_relative_path(path)?);
    let candidate_metadata = match fs::symlink_metadata(&candidate) {
        Ok(metadata) => Some(metadata),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(_) => {
            return Err(
                BusinessError::new("path_unavailable", "path is unavailable").with_retryable(false),
            )
        }
    };
    if candidate_metadata
        .as_ref()
        .is_some_and(|metadata| metadata.file_type().is_symlink())
    {
        return Err(BusinessError::new(
            "scope_denied",
            "writes through symbolic links are not allowed",
        )
        .with_retryable(false));
    }
    let existed = candidate_metadata.is_some();
    let current = if existed {
        let canonical = candidate.canonicalize().map_err(|_| {
            BusinessError::new("path_unavailable", "path is unavailable").with_retryable(false)
        })?;
        if !canonical.starts_with(root) {
            return Err(
                BusinessError::new("scope_denied", "path is outside the project")
                    .with_retryable(false),
            );
        }
        let metadata = fs::metadata(&canonical).map_err(|_| {
            BusinessError::new("path_unavailable", "path is unavailable").with_retryable(false)
        })?;
        if !metadata.is_file() {
            return Err(
                BusinessError::new("not_a_file", "path is not a regular file")
                    .with_retryable(false),
            );
        }
        Some(fs::read(canonical).map_err(|_| {
            BusinessError::new("read_failed", "file could not be read").with_retryable(true)
        })?)
    } else {
        None
    };
    let expected_bytes = match (current.as_ref(), args.expected_base64.as_deref()) {
        (Some(_), Some(value)) => Some(STANDARD.decode(value).map_err(|_| {
            BusinessError::new("invalid_arguments", "expected_base64 is invalid")
                .with_retryable(false)
        })?),
        (Some(_), None) => {
            return Err(BusinessError::new(
                "precondition_required",
                "expected_base64 is required for an existing file",
            )
            .with_retryable(false))
        }
        (None, Some(_)) => {
            return Err(
                BusinessError::new("conflict", "file appeared before the write")
                    .with_retryable(false),
            )
        }
        (None, None) => None,
    };
    if let (Some(actual), Some(expected)) = (current.as_ref(), expected_bytes.as_ref()) {
        if actual != expected {
            return Err(
                BusinessError::new("conflict", "file changed since it was read")
                    .with_retryable(false),
            );
        }
    }
    if let Some(parent) = candidate.parent() {
        ensure_parent_directory(root, parent)?;
    }
    let checkpoint_root = checkpoint_root.ok_or(
        BusinessError::new(
            "checkpoint_unavailable",
            "checkpoint storage is not configured",
        )
        .with_retryable(false),
    )?;
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
    fs::write(&candidate, &bytes).map_err(|_| {
        BusinessError::new("write_failed", "file could not be written").with_retryable(true)
    })?;
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
            return Err(
                BusinessError::new("path_unavailable", "parent directory is unavailable")
                    .with_retryable(false),
            );
        }
    }
    let canonical_existing = existing.canonicalize().map_err(|_| {
        BusinessError::new("path_unavailable", "parent directory is unavailable")
            .with_retryable(false)
    })?;
    if !canonical_existing.starts_with(root) {
        return Err(
            BusinessError::new("scope_denied", "parent directory is outside the project")
                .with_retryable(false),
        );
    }
    fs::create_dir_all(parent).map_err(|_| {
        BusinessError::new("write_failed", "parent directory could not be created")
            .with_retryable(true)
    })?;
    let canonical_parent = parent.canonicalize().map_err(|_| {
        BusinessError::new("path_unavailable", "parent directory is unavailable")
            .with_retryable(false)
    })?;
    if !canonical_parent.starts_with(root) {
        return Err(
            BusinessError::new("scope_denied", "parent directory is outside the project")
                .with_retryable(false),
        );
    }
    Ok(())
}
