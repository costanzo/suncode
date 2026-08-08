use super::{sha256_hex, CoreFailure};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub(super) fn checkpoint_root_from_env() -> Option<PathBuf> {
    std::env::var_os("SUNCODE_CORE_DATA_DIR")
        .map(PathBuf::from)
        .map(|path| path.join("checkpoints"))
}
pub(super) fn artifact_directory(root: &Path) -> PathBuf {
    root.parent().unwrap_or(root).join("artifacts")
}
pub(super) fn valid_opaque_id(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit())
}
pub(super) fn write_artifact(root: &Path, bytes: &[u8]) -> Result<String, CoreFailure> {
    let id = sha256_hex(bytes);
    let directory = artifact_directory(root);
    fs::create_dir_all(&directory).map_err(|_| CoreFailure {
        code: "artifact_failed",
        message: "artifact directory could not be created",
        retryable: true,
    })?;
    let path = directory.join(format!("{}.bin", id));
    if !path.exists() {
        fs::write(path, bytes).map_err(|_| CoreFailure {
            code: "artifact_failed",
            message: "artifact could not be written",
            retryable: true,
        })?;
    }
    Ok(id)
}
pub(super) fn read(root: Option<&Path>, params: &Value) -> Result<Value, CoreFailure> {
    let root = root.ok_or(CoreFailure {
        code: "artifact_unavailable",
        message: "artifact storage is not configured",
        retryable: false,
    })?;
    let id = params
        .get("artifact_id")
        .and_then(Value::as_str)
        .ok_or(CoreFailure {
            code: "invalid_arguments",
            message: "artifact_id is required",
            retryable: false,
        })?;
    if !valid_opaque_id(id) {
        return Err(CoreFailure {
            code: "invalid_arguments",
            message: "artifact_id is invalid",
            retryable: false,
        });
    }
    let bytes = fs::read(artifact_directory(root).join(format!("{}.bin", id))).map_err(|_| {
        CoreFailure {
            code: "artifact_unavailable",
            message: "artifact is unavailable",
            retryable: false,
        }
    })?;
    let max_bytes = params
        .get("max_bytes")
        .and_then(Value::as_u64)
        .unwrap_or(1024 * 1024)
        .clamp(1, 16 * 1024 * 1024) as usize;
    Ok(
        json!({"artifact_id": id, "bytes": bytes.len(), "data_base64": STANDARD.encode(&bytes[..bytes.len().min(max_bytes)]), "truncated": bytes.len() > max_bytes, "sha256": sha256_hex(&bytes)}),
    )
}
pub(super) fn sweep(root: Option<&Path>, params: &Value) -> Result<Value, CoreFailure> {
    let root = root.ok_or(CoreFailure {
        code: "artifact_unavailable",
        message: "artifact storage is not configured",
        retryable: false,
    })?;
    let ids = params
        .get("artifact_ids")
        .and_then(Value::as_array)
        .ok_or(CoreFailure {
            code: "invalid_arguments",
            message: "artifact_ids is required",
            retryable: false,
        })?;
    let mut deleted = Vec::new();
    let mut missing = Vec::new();
    for id_value in ids {
        let Some(id) = id_value.as_str() else {
            return Err(CoreFailure {
                code: "invalid_arguments",
                message: "artifact id is invalid",
                retryable: false,
            });
        };
        if !valid_opaque_id(id) {
            return Err(CoreFailure {
                code: "invalid_arguments",
                message: "artifact id is invalid",
                retryable: false,
            });
        }
        match fs::remove_file(artifact_directory(root).join(format!("{}.bin", id))) {
            Ok(()) => deleted.push(id),
            Err(error) if error.kind() == ErrorKind::NotFound => missing.push(id),
            Err(_) => {
                return Err(CoreFailure {
                    code: "artifact_sweep_failed",
                    message: "artifact could not be deleted",
                    retryable: true,
                })
            }
        }
    }
    Ok(json!({"deleted": deleted, "missing": missing}))
}
