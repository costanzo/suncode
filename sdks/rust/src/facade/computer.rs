use super::*;

impl AsyncAgentSdk {
    pub fn computer_runtime_info(&self) -> SdkResult<suncode_agent::ComputerRuntimeInfo> {
        Ok(self.state.agent.computer_runtime_info())
    }

    pub fn set_computer_use_enabled(
        &self,
        enabled: bool,
    ) -> SdkResult<suncode_agent::ComputerRuntimeInfo> {
        if enabled {
            apply_computer_enablement(&self.state.agent, true)?;
            persist_computer_enablement(&self.state.store, true)?;
        } else {
            persist_computer_enablement(&self.state.store, false)?;
            apply_computer_enablement(&self.state.agent, false)?;
        }
        Ok(self.state.agent.computer_runtime_info())
    }

    pub fn emergency_stop_computer_use(&self) -> SdkResult<suncode_agent::ComputerRuntimeInfo> {
        persist_computer_enablement(&self.state.store, false)?;
        self.state.agent.emergency_stop_computer_use()?;
        Ok(self.state.agent.computer_runtime_info())
    }

    pub fn request_computer_capture_permission(
        &self,
    ) -> SdkResult<suncode_agent::ComputerRuntimeInfo> {
        Ok(self.state.agent.request_computer_capture_permission())
    }

    pub fn request_computer_input_permission(
        &self,
    ) -> SdkResult<suncode_agent::ComputerRuntimeInfo> {
        let info = self.state.agent.request_computer_input_permission();
        if info.input_permission == "allowed" && computer_enablement_desired(&self.state.store) {
            apply_computer_enablement(&self.state.agent, true)?;
        }
        Ok(self.state.agent.computer_runtime_info())
    }

    pub fn take_computer_control(&self) -> SdkResult<suncode_agent::ComputerRuntimeInfo> {
        self.state.agent.take_computer_control()
    }

    pub fn return_computer_control(&self) -> SdkResult<suncode_agent::ComputerRuntimeInfo> {
        self.state.agent.return_computer_control()
    }
}

fn persist_computer_enablement(store: &Store, enabled: bool) -> SdkResult<()> {
    store.set_setting(
        "global",
        "global",
        "computer_use_enabled",
        &Value::Bool(enabled),
    )?;
    Ok(())
}

fn computer_enablement_desired(store: &Store) -> bool {
    store
        .settings(None, None)
        .ok()
        .and_then(|settings| {
            settings
                .into_iter()
                .find(|setting| setting.key == "computer_use_enabled")
                .and_then(|setting| setting.value.as_bool())
        })
        .unwrap_or(false)
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
