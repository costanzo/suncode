//! Agent integration for Browser Use. The runtime itself lives in
//! `suncode-browser`; this module wires it to settings, the tool catalog, and
//! the public `Agent` API.

use super::*;
use suncode_browser::BrowserRuntime;

pub(super) use suncode_browser::tools::{is_browser_tool, validate_browser_arguments};
pub use suncode_browser::{
    BrowserHostCallback, BrowserInstallationState, BrowserRuntimeInfo, BrowserRuntimeState,
    BrowserVisibilityCapability,
};

const ENABLED_SETTING: &str = "browser_use_enabled";

pub(super) fn new_runtime(
    store: &Store,
    data_dir: PathBuf,
    host_available: bool,
) -> BrowserRuntime {
    let enabled = store
        .settings(None, None)
        .ok()
        .and_then(|settings| {
            settings
                .into_iter()
                .find(|setting| setting.key == ENABLED_SETTING)
                .and_then(|setting| setting.value.as_bool())
        })
        .unwrap_or(false);
    BrowserRuntime::new(data_dir, enabled, host_available)
}

pub(super) async fn catalog(runtime: &BrowserRuntime) -> Vec<suncode_llm::ToolDefinition> {
    runtime.catalog().await
}

impl Agent {
    pub fn set_browser_host_callback(&self, callback: Option<BrowserHostCallback>) {
        self.browser.set_host_callback(callback);
    }

    pub async fn browser_runtime_info(
        &self,
        project_id: Option<&str>,
    ) -> Result<BrowserRuntimeInfo, BusinessError> {
        self.browser.info(project_id).await
    }

    pub async fn set_browser_use_enabled(&self, enabled: bool) {
        self.browser.set_enabled(enabled).await;
    }

    pub async fn verify_browser_runtime(&self) -> Result<(), BusinessError> {
        self.browser.verify().await
    }

    pub async fn start_browser_project(&self, project_id: &str) -> Result<(), BusinessError> {
        self.browser.start_project(project_id).await
    }

    pub async fn take_browser_control(&self, project_id: &str) -> Result<(), BusinessError> {
        self.browser.take_control(project_id).await
    }

    pub async fn return_browser_control(&self, project_id: &str) -> Result<(), BusinessError> {
        self.browser.return_control(project_id).await
    }

    pub async fn restart_browser_runtime(&self, project_id: &str) -> Result<(), BusinessError> {
        self.browser.restart(project_id).await
    }

    pub async fn stop_browser_runtime(&self, project_id: &str) -> Result<(), BusinessError> {
        self.browser.stop(project_id).await
    }

    pub async fn clear_browser_profile(&self, project_id: &str) -> Result<(), BusinessError> {
        self.browser.clear_profile(project_id).await
    }
}
