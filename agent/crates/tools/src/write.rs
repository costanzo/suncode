use super::arguments::WriteArguments;
use super::{
    checkpoint, journal_finish, journal_id, journal_intent, load_journal, safe_relative_path,
    CoreFailure,
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
) -> Result<Value, CoreFailure> {
    let root = project_root.ok_or(CoreFailure {
        code: "project_unconfigured",
        message: "project root is not configured",
        retryable: false,
    })?;
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
                return Err(CoreFailure {
                    code: "unknown_completion",
                    message: "operation completion is unknown and must be reconciled",
                    retryable: false,
                });
            }
        }
    }
    let content = args.content_base64.as_str();
    let bytes = STANDARD.decode(content).map_err(|_| CoreFailure {
        code: "invalid_arguments",
        message: "content_base64 is invalid",
        retryable: false,
    })?;
    if bytes.len() > 1024 * 1024 {
        return Err(CoreFailure {
            code: "output_limit",
            message: "file content exceeds the write limit",
            retryable: false,
        });
    }
    let candidate = root.join(safe_relative_path(path)?);
    let candidate_metadata = match fs::symlink_metadata(&candidate) {
        Ok(metadata) => Some(metadata),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(_) => {
            return Err(CoreFailure {
                code: "path_unavailable",
                message: "path is unavailable",
                retryable: false,
            })
        }
    };
    if candidate_metadata
        .as_ref()
        .is_some_and(|metadata| metadata.file_type().is_symlink())
    {
        return Err(CoreFailure {
            code: "scope_denied",
            message: "writes through symbolic links are not allowed",
            retryable: false,
        });
    }
    let existed = candidate_metadata.is_some();
    let current = if existed {
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
        Some(fs::read(canonical).map_err(|_| CoreFailure {
            code: "read_failed",
            message: "file could not be read",
            retryable: true,
        })?)
    } else {
        None
    };
    let expected_bytes = match (current.as_ref(), args.expected_base64.as_deref()) {
        (Some(_), Some(value)) => Some(STANDARD.decode(value).map_err(|_| CoreFailure {
            code: "invalid_arguments",
            message: "expected_base64 is invalid",
            retryable: false,
        })?),
        (Some(_), None) => {
            return Err(CoreFailure {
                code: "precondition_required",
                message: "expected_base64 is required for an existing file",
                retryable: false,
            })
        }
        (None, Some(_)) => {
            return Err(CoreFailure {
                code: "conflict",
                message: "file appeared before the write",
                retryable: false,
            })
        }
        (None, None) => None,
    };
    if let (Some(actual), Some(expected)) = (current.as_ref(), expected_bytes.as_ref()) {
        if actual != expected {
            return Err(CoreFailure {
                code: "conflict",
                message: "file changed since it was read",
                retryable: false,
            });
        }
    }
    if let Some(parent) = candidate.parent() {
        ensure_parent_directory(root, parent)?;
    }
    let checkpoint_root = checkpoint_root.ok_or(CoreFailure {
        code: "checkpoint_unavailable",
        message: "checkpoint storage is not configured",
        retryable: false,
    })?;
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
    fs::write(&candidate, &bytes).map_err(|_| CoreFailure {
        code: "write_failed",
        message: "file could not be written",
        retryable: true,
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

fn ensure_parent_directory(root: &Path, parent: &Path) -> Result<(), CoreFailure> {
    let mut existing = parent.to_path_buf();
    while !existing.exists() {
        if !existing.pop() {
            return Err(CoreFailure {
                code: "path_unavailable",
                message: "parent directory is unavailable",
                retryable: false,
            });
        }
    }
    let canonical_existing = existing.canonicalize().map_err(|_| CoreFailure {
        code: "path_unavailable",
        message: "parent directory is unavailable",
        retryable: false,
    })?;
    if !canonical_existing.starts_with(root) {
        return Err(CoreFailure {
            code: "scope_denied",
            message: "parent directory is outside the project",
            retryable: false,
        });
    }
    fs::create_dir_all(parent).map_err(|_| CoreFailure {
        code: "write_failed",
        message: "parent directory could not be created",
        retryable: true,
    })?;
    let canonical_parent = parent.canonicalize().map_err(|_| CoreFailure {
        code: "path_unavailable",
        message: "parent directory is unavailable",
        retryable: false,
    })?;
    if !canonical_parent.starts_with(root) {
        return Err(CoreFailure {
            code: "scope_denied",
            message: "parent directory is outside the project",
            retryable: false,
        });
    }
    Ok(())
}
