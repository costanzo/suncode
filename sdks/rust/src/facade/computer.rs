use super::*;

impl AsyncAgentSdk {
    pub fn computer_runtime_info(&self) -> SdkResult<suncode_agent::ComputerRuntimeInfo> {
        Ok(self.state.agent.computer_runtime_info())
    }

    pub fn set_computer_use_enabled(
        &self,
        enabled: bool,
    ) -> SdkResult<suncode_agent::ComputerRuntimeInfo> {
        self.state.store.set_setting(
            "global",
            "global",
            "computer_use_enabled",
            &Value::Bool(enabled),
        )?;
        apply_computer_enablement(&self.state.agent, enabled)?;
        Ok(self.state.agent.computer_runtime_info())
    }

    pub fn emergency_stop_computer_use(&self) -> SdkResult<suncode_agent::ComputerRuntimeInfo> {
        self.state.store.set_setting(
            "global",
            "global",
            "computer_use_enabled",
            &Value::Bool(false),
        )?;
        self.state.agent.emergency_stop_computer_use()?;
        Ok(self.state.agent.computer_runtime_info())
    }
}

pub(super) fn apply_computer_enablement(agent: &Agent, enabled: bool) -> SdkResult<()> {
    if enabled && !agent.computer_runtime_info().backend_available {
        agent.install_computer_backend(Box::new(
            suncode_computer::EnigoBackend::new()
                .map_err(|error| BusinessError::unavailable(error.to_string()))?,
        ))?;
    }
    agent.set_computer_use_enabled(enabled)
}
