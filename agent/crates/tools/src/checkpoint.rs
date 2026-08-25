use super::arguments::CheckpointRestoreArguments;
use super::{safe_relative_path, sha256_hex, BusinessError, CHECKPOINT_SEQUENCE};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize)]
struct CheckpointRecord {
    project_root_sha256: String,
    path: String,
    pre_image_base64: Option<String>,
    post_image_sha256: String,
    #[serde(default)]
    post_image_base64: Option<String>,
}

pub(super) fn capture(
    checkpoint_root: &Path,
    project_root: &Path,
    path: &str,
    pre_image: Option<&[u8]>,
    post_image: &[u8],
) -> Result<String, BusinessError> {
    let mut hasher = Sha256::new();
    hasher.update(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            .to_le_bytes(),
    );
    hasher.update(std::process::id().to_le_bytes());
    hasher.update(
        CHECKPOINT_SEQUENCE
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            .to_le_bytes(),
    );
    hasher.update(project_root.to_string_lossy().as_bytes());
    hasher.update(path.as_bytes());
    let checkpoint_id = format!("{:x}", hasher.finalize());
    fs::create_dir_all(checkpoint_root).map_err(|_| {
        BusinessError::new(
            "checkpoint_failed",
            "checkpoint directory could not be created",
        )
        .with_retryable(true)
    })?;
    let record = CheckpointRecord {
        project_root_sha256: sha256_hex(project_root.to_string_lossy().as_bytes()),
        path: path.to_string(),
        pre_image_base64: pre_image.map(|bytes| STANDARD.encode(bytes)),
        post_image_sha256: sha256_hex(post_image),
        post_image_base64: Some(STANDARD.encode(post_image)),
    };
    fs::write(
        checkpoint_root.join(format!("{}.json", checkpoint_id)),
        serde_json::to_vec(&record).map_err(|_| {
            BusinessError::new("checkpoint_failed", "checkpoint could not be encoded")
                .with_retryable(false)
        })?,
    )
    .map_err(|_| {
        BusinessError::new("checkpoint_failed", "checkpoint could not be written")
            .with_retryable(true)
    })?;
    Ok(checkpoint_id)
}

pub(super) fn restore(
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
    args: &CheckpointRestoreArguments,
) -> Result<Value, BusinessError> {
    let root = project_root.ok_or(
        BusinessError::new("project_unconfigured", "project root is not configured")
            .with_retryable(false),
    )?;
    let checkpoint_id = args.checkpoint_id.as_str();
    if checkpoint_id.len() != 64 || !checkpoint_id.chars().all(|value| value.is_ascii_hexdigit()) {
        return Err(
            BusinessError::new("invalid_arguments", "checkpoint_id is invalid")
                .with_retryable(false),
        );
    }
    let checkpoint_root = checkpoint_root.ok_or(
        BusinessError::new(
            "checkpoint_unavailable",
            "checkpoint storage is not configured",
        )
        .with_retryable(false),
    )?;
    let record_path = checkpoint_root.join(format!("{}.json", checkpoint_id));
    let record: CheckpointRecord =
        serde_json::from_slice(&fs::read(&record_path).map_err(|_| {
            BusinessError::new("checkpoint_unavailable", "checkpoint is unavailable")
                .with_retryable(false)
        })?)
        .map_err(|_| {
            BusinessError::new("checkpoint_corrupt", "checkpoint is corrupt").with_retryable(false)
        })?;
    if record.project_root_sha256 != sha256_hex(root.to_string_lossy().as_bytes()) {
        return Err(
            BusinessError::new("scope_denied", "checkpoint belongs to another project")
                .with_retryable(false),
        );
    }
    let target = root.join(safe_relative_path(&record.path)?);
    let metadata = fs::symlink_metadata(&target).ok();
    let target_was_deleted = record.post_image_base64.is_none();
    if metadata
        .as_ref()
        .is_some_and(|value| value.file_type().is_symlink() || !value.is_file())
    {
        return Err(BusinessError::new(
            "restore_conflict",
            "target type changed after checkpoint capture",
        )
        .with_retryable(false));
    }
    if metadata.is_none() && !target_was_deleted {
        return Err(BusinessError::new(
            "restore_conflict",
            "target changed or disappeared after checkpoint capture",
        )
        .with_retryable(false));
    }
    let current = if metadata.is_some() {
        let canonical = target.canonicalize().map_err(|_| {
            BusinessError::new(
                "restore_conflict",
                "target changed or disappeared after checkpoint capture",
            )
            .with_retryable(false)
        })?;
        if !canonical.starts_with(root) {
            return Err(BusinessError::new(
                "restore_conflict",
                "target moved outside the project after checkpoint capture",
            )
            .with_retryable(false));
        }
        fs::read(&canonical).map_err(|_| {
            BusinessError::new(
                "restore_conflict",
                "target changed or disappeared after checkpoint capture",
            )
            .with_retryable(false)
        })?
    } else {
        Vec::new()
    };
    if sha256_hex(&current) != record.post_image_sha256 {
        return Err(BusinessError::new(
            "restore_conflict",
            "target changed after checkpoint capture",
        )
        .with_retryable(false));
    }
    let removed = record.pre_image_base64.is_none();
    if let Some(pre_image) = record.pre_image_base64 {
        fs::write(
            &target,
            STANDARD.decode(pre_image).map_err(|_| {
                BusinessError::new("checkpoint_corrupt", "checkpoint pre-image is corrupt")
                    .with_retryable(false)
            })?,
        )
        .map_err(|_| {
            BusinessError::new("restore_failed", "checkpoint could not be restored")
                .with_retryable(true)
        })?;
    } else if target.exists() {
        fs::remove_file(&target).map_err(|_| {
            BusinessError::new("restore_failed", "created file could not be removed")
                .with_retryable(true)
        })?;
    }
    fs::remove_file(record_path).map_err(|_| {
        BusinessError::new("restore_failed", "checkpoint could not be consumed")
            .with_retryable(true)
    })?;
    Ok(json!({"path": record.path, "restored": true, "removed": removed}))
}
