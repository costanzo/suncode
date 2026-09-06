use super::*;

impl AgentSdk {
    pub fn list_settings(
        &self,
        project_id: Option<&str>,
        session_id: Option<&str>,
    ) -> SdkResult<SettingsResult> {
        Ok(SettingsResult {
            settings: self.state.store.settings(project_id, session_id)?,
        })
    }

    pub fn set_setting(
        &self,
        scope: &str,
        project_id: Option<&str>,
        session_id: Option<&str>,
        key: &str,
        value: &Value,
    ) -> SdkResult<SettingUpdate> {
        validate_setting(scope, key, value)?;
        let scope_id = match scope {
            "global" => "global",
            "project" => {
                project_id.ok_or_else(|| BusinessError::invalid("project_id is required"))?
            }
            "session" => {
                session_id.ok_or_else(|| BusinessError::invalid("session_id is required"))?
            }
            _ => {
                return Err(BusinessError::invalid(
                    "scope must be global, project, or session",
                ))
            }
        };
        self.state.store.set_setting(scope, scope_id, key, value)?;
        if scope == "global" && key == "verify_https_certificates" {
            self.state
                .verify_https_certificates
                .store(value.as_bool().unwrap_or(true), Ordering::SeqCst);
        }
        if scope == "global" && key == "use_system_certificates" {
            self.state
                .use_system_certificates
                .store(value.as_bool().unwrap_or(true), Ordering::SeqCst);
        }
        if scope == "global" && key == "certificate_path" {
            if let Ok(mut path) = self.state.certificate_path.write() {
                *path = value
                    .as_str()
                    .filter(|s| !s.trim().is_empty())
                    .map(PathBuf::from);
            }
        }
        if scope == "global" && (key == "use_system_certificates" || key == "certificate_path") {
            self.state.operations.set_certificate_configuration(
                self.state.use_system_certificates.load(Ordering::SeqCst),
                self.state
                    .certificate_path
                    .read()
                    .ok()
                    .and_then(|path| path.clone()),
            );
        }
        if scope == "global"
            && matches!(
                key,
                "log_level" | "log_directory" | "log_max_bytes" | "log_retention"
            )
        {
            configure_logging(&self.state.store, &self.data_dir)?;
        }
        Ok(SettingUpdate {
            saved: true,
            key: key.to_string(),
            scope: scope.to_string(),
            scope_id: scope_id.to_string(),
        })
    }
}
