use super::computer::apply_computer_enablement;
use super::*;

impl AsyncAgentSdk {
    pub fn list_settings(
        &self,
        project_id: Option<&str>,
        session_id: Option<&str>,
    ) -> SdkResult<SettingsResult> {
        if let Some(project_id) = project_id {
            self.project_for_user(project_id)?;
        }
        if let Some(session_id) = session_id {
            self.session_for_user(session_id)?;
        }
        let mut settings = self.state.store.settings(project_id, session_id)?;
        let password_configured = settings
            .iter()
            .find(|setting| setting.key == "proxy_password")
            .and_then(|setting| setting.value.as_str())
            .is_some_and(|value| !value.is_empty());
        settings.retain(|setting| setting.key != "proxy_password");
        settings.retain(|setting| setting.key != "proxy_password_configured");
        settings.push(suncode_data::SettingRecord {
            key: "proxy_password_configured".into(),
            value: Value::Bool(password_configured),
            scope: "global".into(),
            scope_id: "global".into(),
        });
        settings.sort_by(|left, right| left.key.cmp(&right.key));
        Ok(SettingsResult { settings })
    }

    pub async fn set_setting(
        &self,
        scope: &str,
        project_id: Option<&str>,
        session_id: Option<&str>,
        key: &str,
        value: &Value,
    ) -> SdkResult<SettingUpdate> {
        validate_setting(scope, key, value)?;
        if scope == "global" && key == "browser_use_enabled" {
            self.ensure_browser_host_available()?;
        }
        if scope == "global" && key == "computer_use_enabled" {
            self.ensure_computer_host_available()?;
        }
        if let Some(project_id) = project_id {
            self.project_for_user(project_id)?;
        }
        if let Some(session_id) = session_id {
            self.session_for_user(session_id)?;
        }
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
        if scope == "global" && key == "browser_use_enabled" {
            self.state
                .agent
                .set_browser_use_enabled(value.as_bool().unwrap_or(false))
                .await;
        }
        if scope == "global" && key == "computer_use_enabled" {
            apply_computer_enablement(&self.state.agent, value.as_bool().unwrap_or(false))?;
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

    pub async fn set_proxy_configuration(
        &self,
        request: ProxyConfigurationRequest,
    ) -> SdkResult<ProxyConfigurationResult> {
        let current = self
            .state
            .proxy_configuration
            .read()
            .map(|configuration| configuration.clone())
            .unwrap_or_default();
        let configuration = validate_proxy_configuration(request, &current)?;
        self.state.store.set_global_settings(&[
            (
                "proxy_mode".into(),
                Value::String(configuration.mode.as_str().into()),
            ),
            ("proxy_url".into(), Value::String(configuration.url.clone())),
            (
                "proxy_username".into(),
                Value::String(configuration.username.clone()),
            ),
            (
                "proxy_password".into(),
                Value::String(configuration.password.clone()),
            ),
            (
                "proxy_bypass".into(),
                Value::Array(
                    configuration
                        .bypass
                        .iter()
                        .cloned()
                        .map(Value::String)
                        .collect(),
                ),
            ),
        ])?;
        if let Ok(mut current) = self.state.proxy_configuration.write() {
            *current = configuration.clone();
        }
        self.state
            .operations
            .set_proxy_configuration(configuration.clone());
        self.state
            .agent
            .reconcile_mcp_network_configuration()
            .await?;
        self.state
            .agent
            .set_browser_proxy_configuration(configuration.clone());
        Ok(proxy_configuration_result(&configuration))
    }
}

fn proxy_configuration_result(configuration: &HttpProxyConfiguration) -> ProxyConfigurationResult {
    ProxyConfigurationResult {
        mode: configuration.mode.as_str().into(),
        url: configuration.url.clone(),
        username: configuration.username.clone(),
        password_configured: !configuration.password.is_empty(),
        bypass: configuration.bypass.clone(),
    }
}

fn validate_proxy_configuration(
    request: ProxyConfigurationRequest,
    current: &HttpProxyConfiguration,
) -> SdkResult<HttpProxyConfiguration> {
    let mode = HttpProxyMode::parse(request.mode.trim())
        .ok_or_else(|| BusinessError::invalid("proxy mode must be no_proxy, system, or custom"))?;
    if request.clear_password && request.password.is_some() {
        return Err(BusinessError::invalid(
            "proxy password cannot be replaced and removed together",
        ));
    }
    let url = normalize_proxy_url(&request.url, mode)?;
    let username = request.username.trim().to_string();
    if username.chars().count() > 1024 || username.chars().any(char::is_control) {
        return Err(BusinessError::invalid("proxy username is invalid"));
    }
    let password = if request.clear_password {
        String::new()
    } else if let Some(password) = request.password {
        if password.chars().count() > 8192 || password.contains('\0') {
            return Err(BusinessError::invalid("proxy password is invalid"));
        }
        password
    } else {
        current.password.clone()
    };
    if mode == HttpProxyMode::Custom && username.is_empty() && !password.is_empty() {
        return Err(BusinessError::invalid(
            "proxy username is required when a password is configured",
        ));
    }
    let bypass = normalize_bypass_rules(request.bypass)?;
    Ok(HttpProxyConfiguration {
        mode,
        url,
        username,
        password,
        bypass,
    })
}

fn normalize_proxy_url(value: &str, mode: HttpProxyMode) -> SdkResult<String> {
    let value = value.trim();
    if value.is_empty() {
        if mode == HttpProxyMode::Custom {
            return Err(BusinessError::invalid(
                "proxy URL is required for custom proxy mode",
            ));
        }
        return Ok(String::new());
    }
    let url = url::Url::parse(value)
        .map_err(|_| BusinessError::invalid("proxy URL must be a valid HTTP or HTTPS URL"))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(BusinessError::invalid(
            "proxy URL must be a valid HTTP or HTTPS URL",
        ));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(BusinessError::invalid(
            "proxy URL must not contain embedded credentials",
        ));
    }
    if url.query().is_some() || url.fragment().is_some() || url.path() != "/" {
        return Err(BusinessError::invalid(
            "proxy URL must not contain a path, query, or fragment",
        ));
    }
    Ok(value.trim_end_matches('/').to_string())
}

fn normalize_bypass_rules(values: Vec<String>) -> SdkResult<Vec<String>> {
    if values.len() > 256 {
        return Err(BusinessError::invalid(
            "proxy bypass supports at most 256 rules",
        ));
    }
    let mut normalized = Vec::new();
    for value in values {
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        if value.chars().count() > 512 || !valid_bypass_rule(value) {
            return Err(BusinessError::invalid(format!(
                "proxy bypass rule is invalid: {value}"
            )));
        }
        let value = value.to_ascii_lowercase();
        if !normalized.iter().any(|existing| existing == &value) {
            normalized.push(value);
        }
    }
    Ok(normalized)
}

fn valid_bypass_rule(value: &str) -> bool {
    if value == "*" || value.parse::<std::net::IpAddr>().is_ok() {
        return true;
    }
    if let Some((address, prefix)) = value.rsplit_once('/') {
        let Ok(address) = address.parse::<std::net::IpAddr>() else {
            return false;
        };
        let Ok(prefix) = prefix.parse::<u8>() else {
            return false;
        };
        return match address {
            std::net::IpAddr::V4(_) => prefix <= 32,
            std::net::IpAddr::V6(_) => prefix <= 128,
        };
    }
    let domain = value.strip_prefix('.').unwrap_or(value);
    !domain.is_empty()
        && domain.chars().count() <= 253
        && domain.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '-')
        })
}
