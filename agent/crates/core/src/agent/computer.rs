//! Agent integration for Computer Use. The runtime lives in
//! `suncode-computer`; this module reads its persisted setting and exposes the
//! public `Agent` API.

use super::*;
use suncode_computer::{ComputerBackend, EnigoBackend};

pub use suncode_computer::ComputerRuntimeInfo;

/// The persisted Computer Use setting; disabled when unset or unreadable.
pub(super) fn enabled_setting(store: &Store) -> bool {
    store
        .global_settings()
        .is_ok_and(|settings| settings.computer_use_enabled)
}

impl AgentRuntime {
    pub fn computer_runtime_info(&self) -> ComputerRuntimeInfo {
        self.computer.info()
    }

    pub fn set_computer_use_enabled(&self, enabled: bool) -> Result<(), BusinessError> {
        self.computer.set_enabled(enabled)
    }

    pub fn emergency_stop_computer_use(&self) -> Result<(), BusinessError> {
        self.computer.emergency_stop()
    }

    pub fn take_computer_control(&self) -> Result<ComputerRuntimeInfo, BusinessError> {
        self.computer.take_user_control()?;
        Ok(self.computer.info())
    }

    pub fn return_computer_control(&self) -> Result<ComputerRuntimeInfo, BusinessError> {
        self.computer.return_agent_control()?;
        Ok(self.computer.info())
    }

    pub fn request_computer_capture_permission(
        &self,
    ) -> Result<ComputerRuntimeInfo, BusinessError> {
        self.computer.ensure_host_available()?;
        let _ = EnigoBackend::request_capture_permission();
        Ok(self.computer.info())
    }

    pub fn request_computer_input_permission(&self) -> Result<ComputerRuntimeInfo, BusinessError> {
        self.computer.ensure_host_available()?;
        let _ = EnigoBackend::request_input_permission();
        Ok(self.computer.info())
    }

    /// Installs the input backend and, where the platform has one, the
    /// read-only accessibility backend.
    pub fn install_computer_backend(
        &self,
        backend: Box<dyn ComputerBackend>,
    ) -> Result<(), BusinessError> {
        self.computer.install_backend(backend)?;
        if let Some(accessibility) = suncode_computer::platform_accessibility_backend() {
            self.computer.install_accessibility_backend(accessibility)?;
        }
        Ok(())
    }
}
