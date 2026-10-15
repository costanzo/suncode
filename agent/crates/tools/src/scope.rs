//! Project roots and project-relative path containment.

use crate::error::{fatal, retryable};
use serde_json::{json, Value};
use std::fs;
use std::path::{Component, Path, PathBuf};
use suncode_common::BusinessError;

pub(crate) fn open_project(path: &Path) -> Result<(PathBuf, Value), BusinessError> {
    let requested = path;
    if !requested.is_absolute() {
        return Err(fatal("invalid_arguments", "project path must be absolute"));
    }
    let root = requested
        .canonicalize()
        .map_err(|_| fatal("project_unavailable", "project root is unavailable"))?;
    if !root.is_dir() {
        return Err(fatal(
            "project_unavailable",
            "project root is not a directory",
        ));
    }
    let canonical_path = root.to_str().ok_or(fatal(
        "project_unavailable",
        "project path is not valid UTF-8",
    ))?;
    let display_name = root
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("Project");
    let value = json!({"canonical_path": canonical_path, "display_name": display_name});
    Ok((root, value))
}

pub(crate) fn existing_file(root: &Path, path: &str) -> Result<(PathBuf, Vec<u8>), BusinessError> {
    let relative = safe_relative_path(path)?;
    let candidate = root.join(relative);
    let metadata = fs::symlink_metadata(&candidate)
        .map_err(|_| fatal("path_unavailable", "path is unavailable"))?;
    if metadata.file_type().is_symlink() {
        return Err(fatal("scope_denied", "symbolic links are not allowed"));
    }
    let canonical = candidate
        .canonicalize()
        .map_err(|_| fatal("path_unavailable", "path is unavailable"))?;
    if !canonical.starts_with(root) {
        return Err(fatal("scope_denied", "path is outside the project"));
    }
    if !metadata.is_file() {
        return Err(fatal("not_a_file", "path is not a regular file"));
    }
    let bytes =
        fs::read(&canonical).map_err(|_| retryable("read_failed", "file could not be read"))?;
    Ok((canonical, bytes))
}

pub(crate) fn require_project(project_root: Option<&Path>) -> Result<&Path, BusinessError> {
    let root = project_root.ok_or(fatal(
        "project_unconfigured",
        "project root is not configured",
    ))?;
    if !root.is_dir() {
        return Err(fatal("project_unavailable", "project root is unavailable"));
    }
    Ok(root)
}

pub(crate) fn safe_relative_path(path: &str) -> Result<PathBuf, BusinessError> {
    let value = Path::new(path);
    if value.as_os_str().is_empty() || value.is_absolute() {
        return Err(fatal(
            "scope_denied",
            "path must be relative to the project",
        ));
    }
    for component in value.components() {
        if matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        ) {
            return Err(fatal("scope_denied", "path must remain inside the project"));
        }
    }
    Ok(value.to_path_buf())
}
