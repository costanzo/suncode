use super::*;

impl AsyncAgentSdk {
    pub(super) fn ensure_browser_host_available(&self) -> SdkResult<()> {
        if self.state.host_capabilities.browser_use {
            Ok(())
        } else {
            Err(BusinessError::new(
                "browser_host_unavailable",
                "Browser Use is unavailable in this host",
            ))
        }
    }

    pub async fn browser_runtime_info(
        &self,
        project_id: Option<&str>,
    ) -> SdkResult<suncode_agent::BrowserRuntimeInfo> {
        if let Some(project_id) = project_id {
            self.project_for_user(project_id)?;
        }
        self.state.agent.browser_runtime_info(project_id).await
    }

    pub async fn set_browser_use_enabled(
        &self,
        enabled: bool,
    ) -> SdkResult<suncode_agent::BrowserRuntimeInfo> {
        self.ensure_browser_host_available()?;
        self.state.store.set_setting(
            "global",
            "global",
            "browser_use_enabled",
            &Value::Bool(enabled),
        )?;
        self.state.agent.set_browser_use_enabled(enabled).await;
        self.browser_runtime_info(None).await
    }

    pub async fn verify_browser_runtime(
        &self,
        project_id: Option<&str>,
    ) -> SdkResult<suncode_agent::BrowserRuntimeInfo> {
        self.ensure_browser_host_available()?;
        if let Some(project_id) = project_id {
            self.project_for_user(project_id)?;
        }
        self.state.agent.verify_browser_runtime().await?;
        self.browser_runtime_info(project_id).await
    }

    pub async fn start_browser_project(
        &self,
        project_id: &str,
    ) -> SdkResult<suncode_agent::BrowserRuntimeInfo> {
        self.ensure_browser_host_available()?;
        self.project_for_user(project_id)?;
        self.state.agent.start_browser_project(project_id).await?;
        self.browser_runtime_info(Some(project_id)).await
    }

    pub async fn take_browser_control(
        &self,
        project_id: &str,
    ) -> SdkResult<suncode_agent::BrowserRuntimeInfo> {
        self.ensure_browser_host_available()?;
        self.project_for_user(project_id)?;
        self.state.agent.take_browser_control(project_id).await?;
        self.browser_runtime_info(Some(project_id)).await
    }

    pub async fn return_browser_control(
        &self,
        project_id: &str,
    ) -> SdkResult<suncode_agent::BrowserRuntimeInfo> {
        self.ensure_browser_host_available()?;
        self.project_for_user(project_id)?;
        self.state.agent.return_browser_control(project_id).await?;
        self.browser_runtime_info(Some(project_id)).await
    }

    pub async fn restart_browser_runtime(
        &self,
        project_id: &str,
    ) -> SdkResult<suncode_agent::BrowserRuntimeInfo> {
        self.ensure_browser_host_available()?;
        self.project_for_user(project_id)?;
        self.state.agent.restart_browser_runtime(project_id).await?;
        self.browser_runtime_info(Some(project_id)).await
    }

    pub async fn stop_browser_runtime(
        &self,
        project_id: &str,
    ) -> SdkResult<suncode_agent::BrowserRuntimeInfo> {
        self.ensure_browser_host_available()?;
        self.project_for_user(project_id)?;
        self.state.agent.stop_browser_runtime(project_id).await?;
        self.browser_runtime_info(Some(project_id)).await
    }

    pub async fn clear_browser_profile(
        &self,
        project_id: &str,
    ) -> SdkResult<BrowserProfileClearResult> {
        self.ensure_browser_host_available()?;
        self.project_for_user(project_id)?;
        self.state.agent.clear_browser_profile(project_id).await?;
        Ok(BrowserProfileClearResult {
            project_id: project_id.into(),
            cleared: true,
        })
    }
}
