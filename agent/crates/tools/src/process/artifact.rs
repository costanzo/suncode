//! Persisting complete process output as a managed artifact.

use super::capture::CapturedOutput;
use super::OUTPUT_SEQUENCE;
use crate::artifacts;
use crate::error::retryable;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use suncode_common::BusinessError;

pub(super) fn copy_capture(
    destination: &mut fs::File,
    hasher: &mut Sha256,
    output: &CapturedOutput,
) -> std::io::Result<()> {
    if let Some(path) = output.path.as_ref() {
        let mut source = fs::File::open(path)?;
        let mut buffer = [0u8; 8192];
        loop {
            let bytes = source.read(&mut buffer)?;
            if bytes == 0 {
                break;
            }
            destination.write_all(&buffer[..bytes])?;
            hasher.update(&buffer[..bytes]);
        }
    } else {
        destination.write_all(&output.preview)?;
        hasher.update(&output.preview);
    }
    Ok(())
}

pub(super) fn write_process_artifact(
    root: &Path,
    stdout: &CapturedOutput,
    stderr: &CapturedOutput,
) -> Result<String, BusinessError> {
    let directory = artifacts::artifact_directory(root);
    fs::create_dir_all(&directory)
        .map_err(|_| retryable("artifact_failed", "artifact directory could not be created"))?;
    let temporary = directory.join(format!(
        ".process-{}-{}.tmp",
        std::process::id(),
        OUTPUT_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    let mut temporary_guard = TemporaryArtifact::new(temporary.clone());
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .map_err(|_| retryable("artifact_failed", "artifact could not be created"))?;
    let mut hasher = Sha256::new();
    copy_capture(&mut file, &mut hasher, stdout).map_err(|_| {
        retryable(
            "artifact_failed",
            "process output artifact could not be written",
        )
    })?;
    file.write_all(b"\n--- stderr ---\n").map_err(|_| {
        retryable(
            "artifact_failed",
            "process output artifact could not be written",
        )
    })?;
    hasher.update(b"\n--- stderr ---\n");
    copy_capture(&mut file, &mut hasher, stderr).map_err(|_| {
        retryable(
            "artifact_failed",
            "process output artifact could not be written",
        )
    })?;
    file.flush().map_err(|_| {
        retryable(
            "artifact_failed",
            "process output artifact could not be flushed",
        )
    })?;
    drop(file);
    let id = format!("{:x}", hasher.finalize());
    let destination = directory.join(format!("{id}.bin"));
    if destination.exists() {
        let _ = fs::remove_file(&temporary);
        temporary_guard.committed = true;
    } else if let Err(error) = fs::rename(&temporary, &destination) {
        return Err(retryable(
            "artifact_failed",
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                "artifact already exists"
            } else {
                "artifact could not be finalized"
            },
        ));
    } else {
        temporary_guard.committed = true;
    }
    Ok(id)
}

pub(super) struct TemporaryArtifact {
    path: PathBuf,
    committed: bool,
}

impl TemporaryArtifact {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            committed: false,
        }
    }
}

impl Drop for TemporaryArtifact {
    fn drop(&mut self) {
        if !self.committed {
            let _ = fs::remove_file(&self.path);
        }
    }
}
