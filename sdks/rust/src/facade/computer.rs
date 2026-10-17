use super::*;

impl AsyncAgentSdk {
    pub(super) fn ensure_computer_host_available(&self) -> SdkResult<()> {
        if self.state.host_capabilities.computer_use {
            Ok(())
        } else {
            Err(BusinessError::new(
                "computer_host_unavailable",
                "Computer Use is unavailable in this host",
            ))
        }
    }

    pub fn computer_runtime_info(&self) -> SdkResult<suncode_core::ComputerRuntimeInfo> {
        Ok(self.state.agent.computer_runtime_info())
    }

    pub fn set_computer_use_enabled(
        &self,
        enabled: bool,
    ) -> SdkResult<suncode_core::ComputerRuntimeInfo> {
        self.ensure_computer_host_available()?;
        set_computer_enablement(&self.state.agent, &self.state.store, enabled)?;
        Ok(self.state.agent.computer_runtime_info())
    }

    pub fn emergency_stop_computer_use(&self) -> SdkResult<suncode_core::ComputerRuntimeInfo> {
        self.ensure_computer_host_available()?;
        // Stop first: a failed settings write must not keep the agent in
        // control. The stored setting is then turned off as well.
        self.state.agent.emergency_stop_computer_use()?;
        persist_computer_enablement(&self.state.store, false)?;
        Ok(self.state.agent.computer_runtime_info())
    }

    pub fn request_computer_capture_permission(
        &self,
    ) -> SdkResult<suncode_core::ComputerRuntimeInfo> {
        self.ensure_computer_host_available()?;
        self.state.agent.request_computer_capture_permission()
    }

    pub fn request_computer_input_permission(
        &self,
    ) -> SdkResult<suncode_core::ComputerRuntimeInfo> {
        self.ensure_computer_host_available()?;
        let info = self.state.agent.request_computer_input_permission()?;
        if info.input_permission == "allowed" && computer_enablement_desired(&self.state.store) {
            prepare_computer_enablement(&self.state.agent)?;
            self.state.agent.set_computer_use_enabled(true)?;
        }
        Ok(self.state.agent.computer_runtime_info())
    }

    pub fn take_computer_control(&self) -> SdkResult<suncode_core::ComputerRuntimeInfo> {
        self.ensure_computer_host_available()?;
        self.state.agent.take_computer_control()
    }

    pub fn return_computer_control(&self) -> SdkResult<suncode_core::ComputerRuntimeInfo> {
        self.ensure_computer_host_available()?;
        self.state.agent.return_computer_control()
    }
}

/// Turns Computer Use on or off. The runtime changes only after the setting
/// is stored, so a failed write leaves both unchanged. Enabling first checks
/// the input permission and installs the backend, so an unusable setting is
/// never stored as enabled.
pub(super) fn set_computer_enablement(
    agent: &AgentRuntime,
    store: &Store,
    enabled: bool,
) -> SdkResult<()> {
    if enabled {
        prepare_computer_enablement(agent)?;
    }
    persist_computer_enablement(store, enabled)?;
    agent.set_computer_use_enabled(enabled)
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
        .global_settings()
        .is_ok_and(|settings| settings.computer_use_enabled)
}

/// Ensures the input permission and an installed backend, without enabling
/// the runtime.
fn prepare_computer_enablement(agent: &AgentRuntime) -> SdkResult<()> {
    if !agent.computer_runtime_info().backend_available {
        let runtime = agent.computer_runtime_info();
        if runtime.input_permission == "denied" {
            let permission = agent.request_computer_input_permission()?;
            if permission.input_permission == "denied" {
                return Err(computer_input_permission_required());
            }
        }
        agent.install_computer_backend(Box::new(
            suncode_computer::EnigoBackend::new().map_err(|error| {
                let message = error.to_string();
                if message.contains("permission to simulate input") {
                    computer_input_permission_required()
                } else {
                    BusinessError::unavailable(message)
                }
            })?,
        ))?;
    }
    Ok(())
}

fn computer_input_permission_required() -> BusinessError {
    BusinessError::new(
        "computer_input_permission_required",
        "Computer Use requires input control permission. Grant SunCode access in the operating system privacy settings, then try again.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn denied_input_permission_has_a_stable_sdk_error() {
        let error = computer_input_permission_required();

        assert_eq!(error.code, "computer_input_permission_required");
        assert!(error.message.contains("input control permission"));
    }
}
