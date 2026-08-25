use super::{sha256_hex, BusinessError};
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn checkpoint_root_from_env() -> Option<PathBuf> {
    std::env::var_os("SUNCODE_CORE_DATA_DIR")
        .map(PathBuf::from)
        .map(|path| path.join("checkpoints"))
}
pub(super) fn artifact_directory(root: &Path) -> PathBuf {
    root.parent().unwrap_or(root).join("artifacts")
}
pub(super) fn write_artifact(root: &Path, bytes: &[u8]) -> Result<String, BusinessError> {
    let id = sha256_hex(bytes);
    let directory = artifact_directory(root);
    fs::create_dir_all(&directory).map_err(|_| {
        BusinessError::new("artifact_failed", "artifact directory could not be created")
            .with_retryable(true)
    })?;
    let path = directory.join(format!("{}.bin", id));
    if !path.exists() {
        fs::write(path, bytes).map_err(|_| {
            BusinessError::new("artifact_failed", "artifact could not be written")
                .with_retryable(true)
        })?;
    }
    Ok(id)
}
