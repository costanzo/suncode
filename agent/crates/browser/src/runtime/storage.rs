//! Managed browser data under the agent data directory: profiles,
//! and screenshot artifacts.

use super::BrowserRuntime;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use suncode_common::BusinessError;

impl BrowserRuntime {
    pub async fn clear_profile(&self, project_id: &str) -> Result<(), BusinessError> {
        if self
            .inner
            .projects
            .lock()
            .await
            .get(project_id)
            .is_some_and(|slot| slot.session.is_some())
        {
            return Err(BusinessError::new(
                "browser_runtime_active",
                "Stop the project browser before clearing browser data",
            ));
        }
        let profile = self.profile_path(project_id);
        match fs::symlink_metadata(&profile) {
            Ok(metadata) => {
                let removal = if metadata.file_type().is_symlink() || metadata.is_file() {
                    fs::remove_file(&profile)
                } else {
                    fs::remove_dir_all(&profile)
                };
                removal.map_err(|error| {
                    BusinessError::new(
                        "browser_profile_clear_failed",
                        format!("browser profile could not be removed: {error}"),
                    )
                })?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(BusinessError::new(
                    "browser_profile_clear_failed",
                    format!("browser profile could not be inspected: {error}"),
                ));
            }
        }
        Ok(())
    }

    pub(super) fn profile_path(&self, project_id: &str) -> PathBuf {
        self.cef_root()
            .join(format!("browser-{}", project_directory_name(project_id)))
    }
    
    pub(super) fn cef_root(&self) -> PathBuf {
        self.inner.data_dir.join("cef")
    }

    pub(super) fn retain_screenshot(&self, result: Value) -> Result<Value, BusinessError> {
        let encoded = result
            .get("base64")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                BusinessError::new(
                    "browser_screenshot_invalid",
                    "Browser screenshot data is missing",
                )
            })?;
        let bytes = STANDARD.decode(encoded).map_err(|_| {
            BusinessError::new(
                "browser_screenshot_invalid",
                "Browser screenshot data is invalid",
            )
        })?;
        if bytes.is_empty() || bytes.len() > 5 * 1024 * 1024 {
            return Err(BusinessError::new(
                "browser_screenshot_too_large",
                "Browser screenshot must be between 1 byte and 5 MiB",
            ));
        }
        let artifact_id = suncode_common::id::with_prefix("browser");
        let directory = self.inner.data_dir.join("browser").join("artifacts");
        prepare_managed_directory(&directory, &self.inner.data_dir.join("browser"))
            .map_err(|error| BusinessError::new("browser_artifact_failed", error.message))?;
        fs::write(directory.join(format!("{artifact_id}.png")), &bytes).map_err(|error| {
            BusinessError::new(
                "browser_artifact_failed",
                format!("Browser screenshot could not be stored: {error}"),
            )
        })?;
        Ok(json!({
            "pageId": result.get("pageId").cloned().unwrap_or(Value::Null),
            "artifact_id": artifact_id,
            "mime_type": "image/png",
            "bytes": bytes.len(),
        }))
    }
}

pub(super) fn project_directory_name(project_id: &str) -> String {
    let digest = Sha256::digest(project_id.as_bytes());
    digest[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(super) fn prepare_managed_directory(path: &Path, root: &Path) -> Result<(), BusinessError> {
    fs::create_dir_all(root).map_err(|error| {
        BusinessError::new(
            "browser_runtime_unavailable",
            format!("Browser data root could not be created: {error}"),
        )
    })?;
    if fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
    {
        return Err(BusinessError::new(
            "browser_data_path_invalid",
            "Browser data directory must not be a symbolic link",
        ));
    }
    fs::create_dir_all(path).map_err(|error| {
        BusinessError::new(
            "browser_runtime_unavailable",
            format!("Browser data directory could not be created: {error}"),
        )
    })?;
    let canonical_root = fs::canonicalize(root).map_err(|error| {
        BusinessError::new(
            "browser_runtime_unavailable",
            format!("Browser data root could not be resolved: {error}"),
        )
    })?;
    let canonical_path = fs::canonicalize(path).map_err(|error| {
        BusinessError::new(
            "browser_runtime_unavailable",
            format!("Browser data directory could not be resolved: {error}"),
        )
    })?;
    if !canonical_path.starts_with(canonical_root) {
        return Err(BusinessError::new(
            "browser_data_path_invalid",
            "Browser data directory escaped its managed root",
        ));
    }
    Ok(())
}

pub(super) fn directory_size(path: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| {
            let Ok(file_type) = entry.file_type() else {
                return 0;
            };
            if file_type.is_dir() {
                directory_size(&entry.path())
            } else if file_type.is_file() {
                entry.metadata().map(|metadata| metadata.len()).unwrap_or(0)
            } else {
                0
            }
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_profile_directory_does_not_disclose_project_identifier() {
        let value = project_directory_name("project/with/private/path");
        assert_eq!(value.len(), 32);
        assert!(!value.contains("project"));
        assert!(value.chars().all(|character| character.is_ascii_hexdigit()));
    }

    #[cfg(unix)]
    #[test]
    fn managed_browser_directories_reject_symlinks() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let profiles = root.path().join("profiles");
        fs::create_dir_all(&profiles).unwrap();
        let project = profiles.join("project");
        symlink(outside.path(), &project).unwrap();

        assert_eq!(
            prepare_managed_directory(&project, &profiles)
                .unwrap_err()
                .code,
            "browser_data_path_invalid"
        );
    }
}
