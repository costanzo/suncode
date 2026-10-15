use crate::error::retryable;
use crate::support::sha256_hex;
use std::fs;
use std::path::{Path, PathBuf};
use suncode_common::BusinessError;

pub(crate) fn artifact_directory(root: &Path) -> PathBuf {
    root.parent().unwrap_or(root).join("artifacts")
}
pub(crate) fn write_artifact(root: &Path, bytes: &[u8]) -> Result<String, BusinessError> {
    let id = sha256_hex(bytes);
    let directory = artifact_directory(root);
    fs::create_dir_all(&directory)
        .map_err(|_| retryable("artifact_failed", "artifact directory could not be created"))?;
    let path = directory.join(format!("{}.bin", id));
    if !path.exists() {
        fs::write(path, bytes)
            .map_err(|_| retryable("artifact_failed", "artifact could not be written"))?;
    }
    Ok(id)
}
