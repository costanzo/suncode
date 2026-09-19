use super::*;

impl AgentSdk {
    pub fn browser_runtime_info(
        &self,
        project_id: Option<&str>,
    ) -> SdkResult<suncode_agent::BrowserRuntimeInfo> {
        if let Some(project_id) = project_id {
            self.project_for_user(project_id)?;
        }
        self.runtime
            .block_on(self.state.agent.browser_runtime_info(project_id))
    }

    pub fn set_browser_use_enabled(
        &self,
        enabled: bool,
    ) -> SdkResult<suncode_agent::BrowserRuntimeInfo> {
        self.state.store.set_setting(
            "global",
            "global",
            "browser_use_enabled",
            &Value::Bool(enabled),
        )?;
        self.runtime
            .block_on(self.state.agent.set_browser_use_enabled(enabled));
        self.browser_runtime_info(None)
    }

    pub fn verify_browser_runtime(
        &self,
        project_id: Option<&str>,
    ) -> SdkResult<suncode_agent::BrowserRuntimeInfo> {
        if let Some(project_id) = project_id {
            self.project_for_user(project_id)?;
        }
        self.runtime
            .block_on(self.state.agent.verify_browser_runtime())?;
        self.browser_runtime_info(project_id)
    }

    pub fn start_browser_project(
        &self,
        project_id: &str,
    ) -> SdkResult<suncode_agent::BrowserRuntimeInfo> {
        self.project_for_user(project_id)?;
        self.runtime
            .block_on(self.state.agent.start_browser_project(project_id))?;
        self.browser_runtime_info(Some(project_id))
    }

    pub fn take_browser_control(
        &self,
        project_id: &str,
    ) -> SdkResult<suncode_agent::BrowserRuntimeInfo> {
        self.project_for_user(project_id)?;
        self.runtime
            .block_on(self.state.agent.take_browser_control(project_id))?;
        self.browser_runtime_info(Some(project_id))
    }

    pub fn return_browser_control(
        &self,
        project_id: &str,
    ) -> SdkResult<suncode_agent::BrowserRuntimeInfo> {
        self.project_for_user(project_id)?;
        self.runtime
            .block_on(self.state.agent.return_browser_control(project_id))?;
        self.browser_runtime_info(Some(project_id))
    }

    pub fn restart_browser_runtime(
        &self,
        project_id: &str,
    ) -> SdkResult<suncode_agent::BrowserRuntimeInfo> {
        self.project_for_user(project_id)?;
        self.runtime
            .block_on(self.state.agent.restart_browser_runtime(project_id))?;
        self.browser_runtime_info(Some(project_id))
    }

    pub fn stop_browser_runtime(
        &self,
        project_id: &str,
    ) -> SdkResult<suncode_agent::BrowserRuntimeInfo> {
        self.project_for_user(project_id)?;
        self.runtime
            .block_on(self.state.agent.stop_browser_runtime(project_id))?;
        self.browser_runtime_info(Some(project_id))
    }

    pub fn clear_browser_profile(&self, project_id: &str) -> SdkResult<BrowserProfileClearResult> {
        self.project_for_user(project_id)?;
        self.runtime
            .block_on(self.state.agent.clear_browser_profile(project_id))?;
        Ok(BrowserProfileClearResult {
            project_id: project_id.into(),
            cleared: true,
        })
    }
}
