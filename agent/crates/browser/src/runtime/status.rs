//! Read-only runtime status and the advertised tool catalog.

use super::storage::directory_size;
use super::{
    BrowserInstallationState, BrowserRuntime, BrowserRuntimeInfo, BrowserRuntimeState,
    BrowserVisibilityCapability,
};
use crate::cdp::PROTOCOL_VERSION;
use crate::tools::ToolDefinition;
use std::sync::atomic::Ordering;
use suncode_common::{logging, BusinessError};

impl BrowserRuntime {
    pub async fn info(
        &self,
        project_id: Option<&str>,
    ) -> Result<BrowserRuntimeInfo, BusinessError> {
        let enabled = self.inner.enabled.load(Ordering::SeqCst);
        let (installation_state, error) = self.installation_state(enabled);
        let integrity_verified = self.inner.verified.load(Ordering::SeqCst);
        let installation_ready = matches!(installation_state, BrowserInstallationState::Ready);
        let (runtime_state, control_owner, active_page_count, runtime_error) =
            if let Some(project_id) = project_id {
                let projects = self.inner.projects.lock().await;
                projects
                    .get(project_id)
                    .map(|slot| {
                        (
                            slot.state.clone(),
                            Some(slot.control_owner.clone()),
                            slot.active_page_count,
                            slot.error.clone(),
                        )
                    })
                    .unwrap_or((BrowserRuntimeState::NotStarted, None, 0, None))
            } else {
                (BrowserRuntimeState::NotStarted, None, 0, None)
            };
        let profile = project_id.map(|id| self.profile_path(id));
        Ok(BrowserRuntimeInfo {
            enabled,
            installation_state,
            runtime_state,
            target: format!("cef-cdp:{}", self.inner.cdp_port),
            node_path: String::new(),
            chromium_path: "CEF (embedded)".into(),
            chromium_version: "CEF".into(),
            chromium_revision: String::new(),
            worker_protocol_version: PROTOCOL_VERSION,
            integrity_state: if integrity_verified {
                "verified"
            } else if installation_ready {
                "unverified"
            } else {
                "unavailable"
            }
            .into(),
            control_owner,
            profile_path: profile
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned()),
            profile_size_bytes: profile.as_ref().map(|path| directory_size(path)),
            active_page_count,
            visibility_capability: visibility_capability(),
            error: runtime_error.or(error),
        })
    }

    /// Tool definitions to advertise; empty unless the host and setting allow it.
    pub async fn catalog(&self) -> Vec<ToolDefinition> {
        if !self.inner.host_available {
            logging::warn(
                "browser.catalog",
                "tools_not_advertised reason=host_unavailable",
            );
            return Vec::new();
        }
        let enabled = self.inner.enabled.load(Ordering::SeqCst);
        if !enabled {
            logging::info("browser.catalog", "tools_not_advertised reason=disabled");
            return Vec::new();
        }
        let definitions = crate::tools::definitions();
        logging::debug(
            "browser.catalog",
            format!("tools_advertised count={}", definitions.len()),
        );
        definitions
    }

    /// Host availability takes precedence over the user's enabled setting.
    fn installation_state(&self, enabled: bool) -> (BrowserInstallationState, Option<String>) {
        if !self.inner.host_available {
            (
                BrowserInstallationState::Unsupported,
                Some("Browser Use is unavailable in this host".into()),
            )
        } else if !enabled {
            (BrowserInstallationState::Disabled, None)
        } else {
            (BrowserInstallationState::Ready, None)
        }
    }
}

pub(super) fn visibility_capability() -> BrowserVisibilityCapability {
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        BrowserVisibilityCapability::Full
    } else if cfg!(target_os = "linux") {
        BrowserVisibilityCapability::Limited
    } else {
        BrowserVisibilityCapability::Unsupported
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn unavailable_host_exposes_no_browser_tools_or_runtime_start() {
        let directory = tempfile::tempdir().unwrap();
        let manager = BrowserRuntime::new(directory.path().to_path_buf(), true, false);

        assert!(manager.catalog().await.is_empty());
        let info = manager.info(None).await.unwrap();
        assert!(info.enabled);
        assert_eq!(
            info.installation_state,
            BrowserInstallationState::Unsupported
        );
        assert_eq!(
            manager.start_project("project-1").await.unwrap_err().code,
            "browser_host_unavailable"
        );
    }

    #[tokio::test]
    async fn enabled_host_advertises_browser_tools_before_cef_starts() {
        let directory = tempfile::tempdir().unwrap();
        let manager = BrowserRuntime::new(directory.path().to_path_buf(), true, true);

        let definitions = manager.catalog().await;
        assert_eq!(definitions.len(), 9);
        assert!(definitions
            .iter()
            .any(|definition| definition.name == "browser_open"));
    }
}
