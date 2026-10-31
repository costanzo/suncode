//! Persistence only: executable discovery, validation and installation belong to the host.
use super::*;
impl AsyncAgentSdk {
    pub fn save_shell_preferences(
        &self,
        request: ShellPreferencesRequest,
    ) -> SdkResult<serde_json::Value> {
        let mut values = Vec::new();
        for (key, configuration) in [
            ("agent_shell", request.agent),
            ("terminal_shell", request.terminal),
        ] {
            if let Some(configuration) = configuration {
                let value = serde_json::to_value(configuration)?;
                validate_setting("global", key, &value)?;
                values.push((key.to_string(), value));
            }
        }
        if values.is_empty() {
            return Err(BusinessError::invalid("A shell preference is required"));
        }
        self.state.store.set_global_settings(&values)?;
        Ok(serde_json::json!({"saved": true}))
    }
    /// Trusted in-process host transport settings. Contains proxy credentials: never log it.
    pub fn download_network_configuration(&self) -> SdkResult<DownloadNetworkConfiguration> {
        let settings = self.state.store.global_settings()?;
        Ok(DownloadNetworkConfiguration {
            verify_certificates: settings.tls.verify_https_certificates,
            use_system_certificates: settings.tls.use_system_certificates,
            certificate_path: settings
                .tls
                .certificate_path
                .map(|p| p.to_string_lossy().into_owned()),
            proxy: serde_json::json!({"mode":settings.proxy.mode.as_str(), "url":settings.proxy.url, "username":settings.proxy.username, "password":settings.proxy.password, "bypass":settings.proxy.bypass}),
        })
    }
}
