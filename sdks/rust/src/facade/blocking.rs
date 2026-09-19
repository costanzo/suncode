use super::*;
use serde_json::Value;
use std::ops::{Deref, DerefMut};
use suncode_agent::{agent::TurnResponse, BrowserRuntimeInfo};

pub struct AgentSdk {
    inner: AsyncAgentSdk,
    runtime: tokio::runtime::Runtime,
}

impl Deref for AgentSdk {
    type Target = AsyncAgentSdk;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for AgentSdk {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl AgentSdk {
    pub fn version() -> VersionResult {
        AsyncAgentSdk::version()
    }

    pub fn open_default(user_id: &str) -> SdkResult<Self> {
        Self::open_default_with_providers(user_id, |_| Ok(()))
    }

    pub fn open_default_with_providers<F>(user_id: &str, configure_providers: F) -> SdkResult<Self>
    where
        F: FnOnce(&mut ModelProviderRegistry) -> Result<(), BusinessError>,
    {
        let runtime = build_runtime()?;
        let inner = runtime.block_on(AsyncAgentSdk::open_default_with_providers(
            user_id,
            configure_providers,
        ))?;
        Ok(Self { inner, runtime })
    }

    pub fn as_async(&self) -> &AsyncAgentSdk {
        &self.inner
    }

    pub fn shutdown(self) -> SdkResult<()> {
        let Self { inner, runtime } = self;
        runtime.block_on(inner.shutdown())
    }

    pub fn browser_runtime_info(&self, project_id: Option<&str>) -> SdkResult<BrowserRuntimeInfo> {
        self.runtime
            .block_on(self.inner.browser_runtime_info(project_id))
    }

    pub fn set_browser_use_enabled(&self, enabled: bool) -> SdkResult<BrowserRuntimeInfo> {
        self.runtime
            .block_on(self.inner.set_browser_use_enabled(enabled))
    }

    pub fn verify_browser_runtime(
        &self,
        project_id: Option<&str>,
    ) -> SdkResult<BrowserRuntimeInfo> {
        self.runtime
            .block_on(self.inner.verify_browser_runtime(project_id))
    }

    pub fn start_browser_project(&self, project_id: &str) -> SdkResult<BrowserRuntimeInfo> {
        self.runtime
            .block_on(self.inner.start_browser_project(project_id))
    }

    pub fn take_browser_control(&self, project_id: &str) -> SdkResult<BrowserRuntimeInfo> {
        self.runtime
            .block_on(self.inner.take_browser_control(project_id))
    }

    pub fn return_browser_control(&self, project_id: &str) -> SdkResult<BrowserRuntimeInfo> {
        self.runtime
            .block_on(self.inner.return_browser_control(project_id))
    }

    pub fn restart_browser_runtime(&self, project_id: &str) -> SdkResult<BrowserRuntimeInfo> {
        self.runtime
            .block_on(self.inner.restart_browser_runtime(project_id))
    }

    pub fn stop_browser_runtime(&self, project_id: &str) -> SdkResult<BrowserRuntimeInfo> {
        self.runtime
            .block_on(self.inner.stop_browser_runtime(project_id))
    }

    pub fn clear_browser_profile(&self, project_id: &str) -> SdkResult<BrowserProfileClearResult> {
        self.runtime
            .block_on(self.inner.clear_browser_profile(project_id))
    }

    pub fn set_setting(
        &self,
        scope: &str,
        project_id: Option<&str>,
        session_id: Option<&str>,
        key: &str,
        value: &Value,
    ) -> SdkResult<SettingUpdate> {
        self.runtime.block_on(
            self.inner
                .set_setting(scope, project_id, session_id, key, value),
        )
    }

    pub fn set_proxy_configuration(
        &self,
        request: ProxyConfigurationRequest,
    ) -> SdkResult<ProxyConfigurationResult> {
        self.runtime
            .block_on(self.inner.set_proxy_configuration(request))
    }

    pub fn submit_turn(
        &self,
        session_id: &str,
        input: &str,
        idempotency_key: &str,
        model: Option<&str>,
        reasoning_effort: Option<&str>,
    ) -> SdkResult<TurnResponse> {
        self.runtime.block_on(self.inner.submit_turn(
            session_id,
            input,
            idempotency_key,
            model,
            reasoning_effort,
        ))
    }

    pub fn submit_turn_with_attachments(
        &self,
        session_id: &str,
        input: &str,
        idempotency_key: &str,
        model: Option<&str>,
        reasoning_effort: Option<&str>,
        image_ids: &[String],
    ) -> SdkResult<TurnResponse> {
        self.runtime
            .block_on(self.inner.submit_turn_with_attachments(
                session_id,
                input,
                idempotency_key,
                model,
                reasoning_effort,
                image_ids,
            ))
    }

    pub fn retry_last_turn(&self, session_id: &str) -> SdkResult<TurnResponse> {
        self.runtime
            .block_on(self.inner.retry_last_turn(session_id))
    }

    pub fn resolve_approval(
        &self,
        approval_id: &str,
        decision: &str,
    ) -> SdkResult<ApprovalOutcome> {
        self.runtime
            .block_on(self.inner.resolve_approval(approval_id, decision))
    }

    pub fn reply_question(&self, request_id: &str, answers: &Value) -> SdkResult<QuestionOutcome> {
        self.runtime
            .block_on(self.inner.reply_question(request_id, answers))
    }

    pub fn reject_question(&self, request_id: &str) -> SdkResult<QuestionOutcome> {
        self.runtime
            .block_on(self.inner.reject_question(request_id))
    }

    pub fn start_mcp_project(&self, project_id: &str) -> SdkResult<McpLoadProgressResult> {
        self.runtime
            .block_on(self.inner.start_mcp_project(project_id))
    }

    pub fn mcp_load_progress(&self, project_id: &str) -> McpLoadProgressResult {
        self.runtime
            .block_on(self.inner.mcp_load_progress(project_id))
    }

    pub fn list_mcp_servers(&self, project_id: Option<&str>) -> SdkResult<McpServersResult> {
        self.runtime
            .block_on(self.inner.list_mcp_servers(project_id))
    }

    pub fn create_mcp_server(
        &self,
        project_id: Option<&str>,
        idempotency_key: &str,
        request: McpServerWriteRequest,
    ) -> SdkResult<McpServerDto> {
        self.runtime.block_on(
            self.inner
                .create_mcp_server(project_id, idempotency_key, request),
        )
    }

    pub fn update_mcp_server(
        &self,
        project_id: Option<&str>,
        server_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
        request: McpServerWriteRequest,
    ) -> SdkResult<McpServerDto> {
        self.runtime.block_on(self.inner.update_mcp_server(
            project_id,
            server_id,
            expected_revision,
            idempotency_key,
            request,
        ))
    }

    pub fn set_mcp_server_enabled(
        &self,
        project_id: Option<&str>,
        server_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
        enabled: bool,
    ) -> SdkResult<McpServerDto> {
        self.runtime.block_on(self.inner.set_mcp_server_enabled(
            project_id,
            server_id,
            expected_revision,
            idempotency_key,
            enabled,
        ))
    }

    pub fn delete_mcp_server(
        &self,
        server_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> SdkResult<McpServerDeleteResult> {
        self.runtime.block_on(self.inner.delete_mcp_server(
            server_id,
            expected_revision,
            idempotency_key,
        ))
    }

    pub fn retry_mcp_server(&self, project_id: &str, server_id: &str) -> SdkResult<McpServerDto> {
        self.runtime
            .block_on(self.inner.retry_mcp_server(project_id, server_id))
    }

    pub fn start_language_server_project(
        &self,
        project_id: &str,
    ) -> SdkResult<LanguageServerProjectResult> {
        self.runtime
            .block_on(self.inner.start_language_server_project(project_id))
    }

    pub fn list_language_servers(
        &self,
        project_id: Option<&str>,
    ) -> SdkResult<LanguageServersResult> {
        self.runtime
            .block_on(self.inner.list_language_servers(project_id))
    }

    pub fn create_language_server(
        &self,
        project_id: Option<&str>,
        idempotency_key: &str,
        request: LanguageServerWriteRequest,
    ) -> SdkResult<LanguageServerDto> {
        self.runtime.block_on(self.inner.create_language_server(
            project_id,
            idempotency_key,
            request,
        ))
    }

    pub fn update_language_server(
        &self,
        project_id: Option<&str>,
        language_server_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
        request: LanguageServerWriteRequest,
    ) -> SdkResult<LanguageServerDto> {
        self.runtime.block_on(self.inner.update_language_server(
            project_id,
            language_server_id,
            expected_revision,
            idempotency_key,
            request,
        ))
    }

    pub fn set_language_server_enabled(
        &self,
        project_id: Option<&str>,
        language_server_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
        enabled: bool,
    ) -> SdkResult<LanguageServerDto> {
        self.runtime
            .block_on(self.inner.set_language_server_enabled(
                project_id,
                language_server_id,
                expected_revision,
                idempotency_key,
                enabled,
            ))
    }

    pub fn delete_language_server(
        &self,
        language_server_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> SdkResult<LanguageServerDeleteResult> {
        self.runtime.block_on(self.inner.delete_language_server(
            language_server_id,
            expected_revision,
            idempotency_key,
        ))
    }

    pub fn retry_language_server(
        &self,
        project_id: &str,
        language_server_id: &str,
    ) -> SdkResult<LanguageServerDto> {
        self.runtime.block_on(
            self.inner
                .retry_language_server(project_id, language_server_id),
        )
    }

    #[cfg(test)]
    pub(super) fn from_async_for_test(inner: AsyncAgentSdk) -> Self {
        Self {
            inner,
            runtime: build_runtime().unwrap(),
        }
    }

    #[cfg(test)]
    pub(super) fn from_state_for_test(state: AgentState) -> Self {
        Self::from_async_for_test(AsyncAgentSdk::from_state_for_test(state))
    }
}

fn build_runtime() -> SdkResult<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("suncode-sdk-blocking")
        .build()
        .map_err(|error| BusinessError::unavailable(format!("tokio runtime unavailable: {error}")))
}
