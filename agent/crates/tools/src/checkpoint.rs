use crate::arguments::CheckpointRestoreArguments;
use crate::error::{fatal, retryable};
use crate::scope::safe_relative_path;
use crate::support::sha256_hex;
use std::sync::atomic::AtomicU64;
use suncode_common::BusinessError;

static CHECKPOINT_SEQUENCE: AtomicU64 = AtomicU64::new(1);
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

pub(crate) fn capture(
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
        retryable(
            "checkpoint_failed",
            "checkpoint directory could not be created",
        )
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
        serde_json::to_vec(&record)
            .map_err(|_| fatal("checkpoint_failed", "checkpoint could not be encoded"))?,
    )
    .map_err(|_| retryable("checkpoint_failed", "checkpoint could not be written"))?;
    Ok(checkpoint_id)
}

pub(crate) fn restore(
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
    args: &CheckpointRestoreArguments,
) -> Result<Value, BusinessError> {
    let root = project_root.ok_or(fatal(
        "project_unconfigured",
        "project root is not configured",
    ))?;
    let checkpoint_id = args.checkpoint_id.as_str();
    if checkpoint_id.len() != 64 || !checkpoint_id.chars().all(|value| value.is_ascii_hexdigit()) {
        return Err(fatal("invalid_arguments", "checkpoint_id is invalid"));
    }
    let checkpoint_root = checkpoint_root.ok_or(fatal(
        "checkpoint_unavailable",
        "checkpoint storage is not configured",
    ))?;
    let record_path = checkpoint_root.join(format!("{}.json", checkpoint_id));
    let record: CheckpointRecord = serde_json::from_slice(
        &fs::read(&record_path)
            .map_err(|_| fatal("checkpoint_unavailable", "checkpoint is unavailable"))?,
    )
    .map_err(|_| fatal("checkpoint_corrupt", "checkpoint is corrupt"))?;
    if record.project_root_sha256 != sha256_hex(root.to_string_lossy().as_bytes()) {
        return Err(fatal(
            "scope_denied",
            "checkpoint belongs to another project",
        ));
    }
    let target = root.join(safe_relative_path(&record.path)?);
    let metadata = fs::symlink_metadata(&target).ok();
    let target_was_deleted = record.post_image_base64.is_none();
    if metadata
        .as_ref()
        .is_some_and(|value| value.file_type().is_symlink() || !value.is_file())
    {
        return Err(fatal(
            "restore_conflict",
            "target type changed after checkpoint capture",
        ));
    }
    if metadata.is_none() && !target_was_deleted {
        return Err(fatal(
            "restore_conflict",
            "target changed or disappeared after checkpoint capture",
        ));
    }
    let current = if metadata.is_some() {
        let canonical = target.canonicalize().map_err(|_| {
            fatal(
                "restore_conflict",
                "target changed or disappeared after checkpoint capture",
            )
        })?;
        if !canonical.starts_with(root) {
            return Err(fatal(
                "restore_conflict",
                "target moved outside the project after checkpoint capture",
            ));
        }
        fs::read(&canonical).map_err(|_| {
            fatal(
                "restore_conflict",
                "target changed or disappeared after checkpoint capture",
            )
        })?
    } else {
        Vec::new()
    };
    if sha256_hex(&current) != record.post_image_sha256 {
        return Err(fatal(
            "restore_conflict",
            "target changed after checkpoint capture",
        ));
    }
    let removed = record.pre_image_base64.is_none();
    if let Some(pre_image) = record.pre_image_base64 {
        fs::write(
            &target,
            STANDARD
                .decode(pre_image)
                .map_err(|_| fatal("checkpoint_corrupt", "checkpoint pre-image is corrupt"))?,
        )
        .map_err(|_| retryable("restore_failed", "checkpoint could not be restored"))?;
    } else if target.exists() {
        fs::remove_file(&target)
            .map_err(|_| retryable("restore_failed", "created file could not be removed"))?;
    }
    fs::remove_file(record_path)
        .map_err(|_| retryable("restore_failed", "checkpoint could not be consumed"))?;
    Ok(json!({"path": record.path, "restored": true, "removed": removed}))
}
