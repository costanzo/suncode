use super::*;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use suncode_data::{LanguageServerConfig, LanguageServerInput, LanguageServerRecord};

const JDTLS_KIND: &str = "jdtls";
const JDTLS_DISPLAY_NAME: &str = "Eclipse JDT Language Server";
const JDTLS_DESCRIPTION: &str =
    "Java diagnostics, definitions, references, hover details, and symbols.";
const DEFAULT_ROOT_MARKERS: &[&str] = &["pom.xml", "build.gradle", "build.gradle.kts"];
const VTSLS_KIND: &str = "vtsls";
const VTSLS_DISPLAY_NAME: &str = "vtsls";
const VTSLS_DESCRIPTION: &str =
    "JavaScript and TypeScript diagnostics, definitions, references, hover details, and symbols.";
const VTSLS_ROOT_MARKERS: &[&str] = &["package.json", "tsconfig.json", "jsconfig.json"];
const VTSLS_LANGUAGE_IDS: &[&str] = &[
    "javascript",
    "javascriptreact",
    "typescript",
    "typescriptreact",
];
const BASEDPYRIGHT_KIND: &str = "basedpyright";
const BASEDPYRIGHT_DISPLAY_NAME: &str = "basedpyright";
const BASEDPYRIGHT_DESCRIPTION: &str =
    "Python diagnostics, definitions, references, hover details, and symbols.";
const BASEDPYRIGHT_ROOT_MARKERS: &[&str] = &[
    "pyproject.toml",
    "pyrightconfig.json",
    "setup.py",
    "setup.cfg",
    "requirements.txt",
];

impl AsyncAgentSdk {
    pub async fn start_language_server_project(
        &self,
        project_id: &str,
    ) -> SdkResult<LanguageServerProjectResult> {
        let project = self.project_for_user(project_id)?;
        self.state
            .agent
            .activate_language_server_project(project_id, Path::new(&project.canonical_root))
            .await?;
        Ok(LanguageServerProjectResult {
            project_id: project_id.to_string(),
            started: true,
        })
    }

    pub async fn list_language_servers(
        &self,
        project_id: Option<&str>,
    ) -> SdkResult<LanguageServersResult> {
        let Some(project_id) = project_id else {
            return Ok(LanguageServersResult {
                servers: vec![
                    self.unscoped_jdtls_dto(),
                    self.unscoped_vtsls_dto(),
                    self.unscoped_basedpyright_dto(),
                ],
            });
        };
        let project = self.project_for_user(project_id)?;
        let record = self
            .state
            .store
            .language_server_for_project(project_id, JDTLS_KIND)?;
        let jdtls = self
            .jdtls_dto(project_id, &project.canonical_root, record)
            .await?;
        let vtsls_record = self
            .state
            .store
            .language_server_for_project(project_id, VTSLS_KIND)?;
        let vtsls = self
            .vtsls_dto(project_id, &project.canonical_root, vtsls_record)
            .await?;
        let basedpyright_record = self
            .state
            .store
            .language_server_for_project(project_id, BASEDPYRIGHT_KIND)?;
        let basedpyright = self
            .basedpyright_dto(project_id, &project.canonical_root, basedpyright_record)
            .await?;
        Ok(LanguageServersResult {
            servers: vec![jdtls, vtsls, basedpyright],
        })
    }

    pub async fn configure_jdtls_language_server(
        &self,
        project_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
        request: JdtlsLanguageServerWriteRequest,
    ) -> SdkResult<LanguageServerDto> {
        let project = self.project_for_user(project_id)?;
        require_idempotency_key(idempotency_key)?;
        validate_jdtls_request(&request)?;
        let current = self
            .state
            .store
            .language_server_for_project(project_id, JDTLS_KIND)?;
        let environment = apply_environment_changes(
            current
                .as_ref()
                .map(|record| record.config.environment.clone())
                .unwrap_or_default(),
            request.environment,
        )?;
        let data_path_mode = match request.data_path_mode.trim() {
            "" | "managed" => "managed",
            "custom" => "custom",
            _ => {
                return Err(BusinessError::invalid(
                    "jdtls data path mode must be managed or custom",
                ))
            }
        };
        let data_path = if data_path_mode == "managed" {
            self.managed_jdtls_data_path(project_id)
        } else {
            let value = request
                .data_path
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| BusinessError::invalid("custom jdtls data path is required"))?;
            let path = PathBuf::from(value);
            if !path.is_absolute() {
                return Err(BusinessError::invalid(
                    "custom jdtls data path must be absolute",
                ));
            }
            path
        };
        let mut environment = environment;
        if let Some(java_home) = request
            .java_home
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            environment.insert("JAVA_HOME".into(), java_home.into());
        } else {
            environment.remove("JAVA_HOME");
        }
        let input = LanguageServerInput {
            // The legacy table keeps a global uniqueness constraint. The SDK never exposes this
            // storage-only name; catalog identity remains the stable `jdtls` kind.
            display_name: format!("{JDTLS_DISPLAY_NAME} [{project_id}]"),
            config: LanguageServerConfig {
                version: 2,
                project_id: Some(project_id.to_string()),
                server_kind: Some(JDTLS_KIND.into()),
                command: request.executable.trim().to_string(),
                arguments: vec!["-data".into(), data_path.to_string_lossy().into_owned()],
                language_ids: vec!["java".into()],
                root_markers: if request.root_markers.is_empty() {
                    DEFAULT_ROOT_MARKERS
                        .iter()
                        .map(|value| (*value).into())
                        .collect()
                } else {
                    request.root_markers
                },
                initialization_options: json!({}),
                configuration: json!({}),
                environment,
                startup_timeout_seconds: request.startup_timeout_seconds,
                request_timeout_seconds: request.request_timeout_seconds,
                java_home: request.java_home.filter(|value| !value.trim().is_empty()),
                data_path_mode: Some(data_path_mode.into()),
                data_path: Some(data_path.to_string_lossy().into_owned()),
                vtsls_executable: None,
                node_path: None,
                typescript_sdk_mode: None,
                typescript_sdk_path: None,
                max_ts_server_memory: None,
                basedpyright_executable: None,
                python_path: None,
                diagnostic_mode: None,
                type_checking_mode: None,
                config_path_mode: None,
                config_path: None,
            },
            enabled: request.enabled,
            sort_order: 0,
        };
        let record = match current {
            Some(current) => self.state.store.update_language_server(
                &current.language_server_id,
                expected_revision,
                &input,
            )?,
            None if expected_revision == 0 => self.state.store.create_language_server(
                &project_language_server_id(project_id, JDTLS_KIND),
                &input,
            )?,
            None => return Err(BusinessError::missing("language_server")),
        };
        self.state
            .agent
            .activate_language_server_project(project_id, Path::new(&project.canonical_root))
            .await?;
        self.state
            .agent
            .reconcile_language_server(project_id, JDTLS_KIND)
            .await?;
        self.jdtls_dto(project_id, &project.canonical_root, Some(record))
            .await
    }

    pub async fn configure_vtsls_language_server(
        &self,
        project_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
        request: VtslsLanguageServerWriteRequest,
    ) -> SdkResult<LanguageServerDto> {
        let project = self.project_for_user(project_id)?;
        require_idempotency_key(idempotency_key)?;
        validate_vtsls_request(&request)?;
        let current = self
            .state
            .store
            .language_server_for_project(project_id, VTSLS_KIND)?;
        let environment = apply_environment_changes(
            current
                .as_ref()
                .map(|record| record.config.environment.clone())
                .unwrap_or_default(),
            request.environment,
        )?;
        let sdk_mode = normalize_typescript_sdk_mode(&request.typescript_sdk_mode)?;
        let sdk_path = request
            .typescript_sdk_path
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let executable = request.executable.trim().to_string();
        let node_path = request.node_path.trim().to_string();
        let server_script =
            resolve_vtsls_script(&executable).unwrap_or_else(|| PathBuf::from(&executable));
        let configuration =
            vtsls_configuration(sdk_mode, sdk_path.as_deref(), request.max_ts_server_memory);
        let input = LanguageServerInput {
            display_name: format!("{VTSLS_DISPLAY_NAME} [{project_id}]"),
            config: LanguageServerConfig {
                version: 3,
                project_id: Some(project_id.to_string()),
                server_kind: Some(VTSLS_KIND.into()),
                command: node_path.clone(),
                arguments: vec![
                    server_script.to_string_lossy().into_owned(),
                    "--stdio".into(),
                ],
                language_ids: VTSLS_LANGUAGE_IDS
                    .iter()
                    .map(|value| (*value).into())
                    .collect(),
                root_markers: if request.root_markers.is_empty() {
                    VTSLS_ROOT_MARKERS
                        .iter()
                        .map(|value| (*value).into())
                        .collect()
                } else {
                    request.root_markers
                },
                initialization_options: json!({}),
                configuration,
                environment,
                startup_timeout_seconds: request.startup_timeout_seconds,
                request_timeout_seconds: request.request_timeout_seconds,
                java_home: None,
                data_path_mode: None,
                data_path: None,
                vtsls_executable: Some(executable),
                node_path: Some(node_path),
                typescript_sdk_mode: Some(sdk_mode.into()),
                typescript_sdk_path: sdk_path,
                max_ts_server_memory: Some(request.max_ts_server_memory),
                basedpyright_executable: None,
                python_path: None,
                diagnostic_mode: None,
                type_checking_mode: None,
                config_path_mode: None,
                config_path: None,
            },
            enabled: request.enabled,
            sort_order: 10,
        };
        let record = match current {
            Some(current) => self.state.store.update_language_server(
                &current.language_server_id,
                expected_revision,
                &input,
            )?,
            None if expected_revision == 0 => self.state.store.create_language_server(
                &project_language_server_id(project_id, VTSLS_KIND),
                &input,
            )?,
            None => return Err(BusinessError::missing("language_server")),
        };
        self.state
            .agent
            .activate_language_server_project(project_id, Path::new(&project.canonical_root))
            .await?;
        self.state
            .agent
            .reconcile_language_server(project_id, VTSLS_KIND)
            .await?;
        self.vtsls_dto(project_id, &project.canonical_root, Some(record))
            .await
    }

    pub async fn configure_basedpyright_language_server(
        &self,
        project_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
        request: BasedpyrightLanguageServerWriteRequest,
    ) -> SdkResult<LanguageServerDto> {
        let project = self.project_for_user(project_id)?;
        require_idempotency_key(idempotency_key)?;
        validate_basedpyright_request(&request)?;
        let current = self
            .state
            .store
            .language_server_for_project(project_id, BASEDPYRIGHT_KIND)?;
        let environment = apply_environment_changes(
            current
                .as_ref()
                .map(|record| record.config.environment.clone())
                .unwrap_or_default(),
            request.environment,
        )?;
        let diagnostic_mode = normalize_diagnostic_mode(&request.diagnostic_mode)?;
        let type_checking_mode = normalize_type_checking_mode(&request.type_checking_mode)?;
        let config_path_mode = normalize_config_path_mode(&request.config_path_mode)?;
        let python_path = optional_trimmed(request.python_path);
        let config_path = (config_path_mode == "custom")
            .then(|| optional_trimmed(request.config_path))
            .flatten();
        let executable = request.executable.trim().to_string();
        let configuration = basedpyright_configuration(
            python_path.as_deref(),
            diagnostic_mode,
            type_checking_mode,
            config_path_mode,
            config_path.as_deref(),
        );
        let input = LanguageServerInput {
            display_name: format!("{BASEDPYRIGHT_DISPLAY_NAME} [{project_id}]"),
            config: LanguageServerConfig {
                version: 4,
                project_id: Some(project_id.to_string()),
                server_kind: Some(BASEDPYRIGHT_KIND.into()),
                command: executable.clone(),
                arguments: vec!["--stdio".into()],
                language_ids: vec!["python".into()],
                root_markers: if request.root_markers.is_empty() {
                    BASEDPYRIGHT_ROOT_MARKERS
                        .iter()
                        .map(|value| (*value).into())
                        .collect()
                } else {
                    request.root_markers
                },
                initialization_options: json!({}),
                configuration,
                environment,
                startup_timeout_seconds: request.startup_timeout_seconds,
                request_timeout_seconds: request.request_timeout_seconds,
                java_home: None,
                data_path_mode: None,
                data_path: None,
                vtsls_executable: None,
                node_path: None,
                typescript_sdk_mode: None,
                typescript_sdk_path: None,
                max_ts_server_memory: None,
                basedpyright_executable: Some(executable),
                python_path,
                diagnostic_mode: Some(diagnostic_mode.into()),
                type_checking_mode: Some(type_checking_mode.into()),
                config_path_mode: Some(config_path_mode.into()),
                config_path,
            },
            enabled: request.enabled,
            sort_order: 20,
        };
        let record = match current {
            Some(current) => self.state.store.update_language_server(
                &current.language_server_id,
                expected_revision,
                &input,
            )?,
            None if expected_revision == 0 => self.state.store.create_language_server(
                &project_language_server_id(project_id, BASEDPYRIGHT_KIND),
                &input,
            )?,
            None => return Err(BusinessError::missing("language_server")),
        };
        self.state
            .agent
            .activate_language_server_project(project_id, Path::new(&project.canonical_root))
            .await?;
        self.state
            .agent
            .reconcile_language_server(project_id, BASEDPYRIGHT_KIND)
            .await?;
        self.basedpyright_dto(project_id, &project.canonical_root, Some(record))
            .await
    }

    pub async fn set_language_server_enabled(
        &self,
        project_id: Option<&str>,
        language_server_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
        enabled: bool,
    ) -> SdkResult<LanguageServerDto> {
        let project_id =
            project_id.ok_or_else(|| BusinessError::invalid("project_id is required"))?;
        let project = self.project_for_user(project_id)?;
        require_idempotency_key(idempotency_key)?;
        let current = self
            .state
            .store
            .language_server_for_project(project_id, language_server_id)?;
        match language_server_id {
            JDTLS_KIND => {
                let request = match current.as_ref() {
                    Some(record) => jdtls_request_from_record(record, enabled),
                    None => self.default_jdtls_request(project_id, enabled),
                };
                self.configure_jdtls_language_server(
                    project_id,
                    expected_revision,
                    idempotency_key,
                    request,
                )
                .await
            }
            VTSLS_KIND => {
                let request = match current.as_ref() {
                    Some(record) => vtsls_request_from_record(record, enabled),
                    None => self.default_vtsls_request(enabled),
                };
                self.configure_vtsls_language_server(
                    project_id,
                    expected_revision,
                    idempotency_key,
                    request,
                )
                .await
            }
            BASEDPYRIGHT_KIND => {
                let request = match current.as_ref() {
                    Some(record) => basedpyright_request_from_record(record, enabled),
                    None => self.default_basedpyright_request(
                        project_id,
                        &project.canonical_root,
                        enabled,
                    ),
                };
                self.configure_basedpyright_language_server(
                    project_id,
                    expected_revision,
                    idempotency_key,
                    request,
                )
                .await
            }
            _ => Err(BusinessError::missing("language_server")),
        }
    }

    pub async fn retry_language_server(
        &self,
        project_id: &str,
        language_server_id: &str,
    ) -> SdkResult<LanguageServerDto> {
        let project = self.project_for_user(project_id)?;
        if !matches!(
            language_server_id,
            JDTLS_KIND | VTSLS_KIND | BASEDPYRIGHT_KIND
        ) {
            return Err(BusinessError::missing("language_server"));
        }
        self.state
            .agent
            .retry_language_server(project_id, language_server_id)
            .await?;
        let record = self
            .state
            .store
            .language_server_for_project(project_id, language_server_id)?;
        match language_server_id {
            JDTLS_KIND => {
                self.jdtls_dto(project_id, &project.canonical_root, record)
                    .await
            }
            VTSLS_KIND => {
                self.vtsls_dto(project_id, &project.canonical_root, record)
                    .await
            }
            BASEDPYRIGHT_KIND => {
                self.basedpyright_dto(project_id, &project.canonical_root, record)
                    .await
            }
            _ => unreachable!(),
        }
    }

    // Compatibility entry points remain callable for existing bindings while the product surface
    // moves to the typed project catalog. They accept only the built-in jdtls shape.
    pub async fn create_language_server(
        &self,
        project_id: Option<&str>,
        idempotency_key: &str,
        request: LanguageServerWriteRequest,
    ) -> SdkResult<LanguageServerDto> {
        let project_id =
            project_id.ok_or_else(|| BusinessError::invalid("project_id is required"))?;
        self.configure_jdtls_language_server(
            project_id,
            0,
            idempotency_key,
            legacy_request(request),
        )
        .await
    }

    pub async fn update_language_server(
        &self,
        project_id: Option<&str>,
        language_server_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
        request: LanguageServerWriteRequest,
    ) -> SdkResult<LanguageServerDto> {
        let project_id =
            project_id.ok_or_else(|| BusinessError::invalid("project_id is required"))?;
        if language_server_id != JDTLS_KIND {
            return Err(BusinessError::missing("language_server"));
        }
        self.configure_jdtls_language_server(
            project_id,
            expected_revision,
            idempotency_key,
            legacy_request(request),
        )
        .await
    }

    pub async fn delete_language_server(
        &self,
        language_server_id: &str,
        _expected_revision: u64,
        idempotency_key: &str,
    ) -> SdkResult<LanguageServerDeleteResult> {
        require_idempotency_key(idempotency_key)?;
        Ok(LanguageServerDeleteResult {
            language_server_id: language_server_id.to_string(),
            removed: false,
        })
    }

    async fn jdtls_dto(
        &self,
        project_id: &str,
        project_root: &str,
        record: Option<LanguageServerRecord>,
    ) -> SdkResult<LanguageServerDto> {
        let detected_executable = detect_jdtls();
        let detected_command = match record.as_ref() {
            Some(record) => {
                command_exists(&record.config.command).then(|| record.config.command.clone())
            }
            None => detected_executable.clone(),
        }
        .or_else(|| detected_executable.clone());
        let detected = detected_command.is_some();
        let detected_java_home = detect_java_home();
        let default = self.default_jdtls_record(project_id);
        let record = record.unwrap_or(default);
        let runtime = self
            .state
            .agent
            .language_server_runtime_status(Some(project_id), &record)
            .await;
        let data_path = record.config.data_path.clone().unwrap_or_else(|| {
            self.managed_jdtls_data_path(project_id)
                .to_string_lossy()
                .into()
        });
        Ok(LanguageServerDto {
            language_server_id: JDTLS_KIND.into(),
            kind: JDTLS_KIND.into(),
            display_name: JDTLS_DISPLAY_NAME.into(),
            description: JDTLS_DESCRIPTION.into(),
            language: "Java".into(),
            detected,
            detected_version: None,
            detected_executable: detected_executable.clone(),
            detected_java_home: detected_java_home.clone(),
            vtsls: None,
            basedpyright: None,
            project_configurable: true,
            command: if record.revision == 0 {
                detected_command.unwrap_or_else(|| "jdtls".into())
            } else {
                record.config.command.clone()
            },
            arguments: record.config.arguments.clone(),
            language_ids: vec!["java".into()],
            root_markers: record.config.root_markers.clone(),
            initialization_options: json!({}),
            environment_keys: record.config.environment.keys().cloned().collect(),
            startup_timeout_seconds: record.config.startup_timeout_seconds,
            request_timeout_seconds: record.config.request_timeout_seconds,
            java_home: record.config.java_home.clone().or(detected_java_home),
            data_path_mode: record
                .config
                .data_path_mode
                .clone()
                .unwrap_or_else(|| "managed".into()),
            data_path,
            matched_root_marker: record
                .config
                .root_markers
                .iter()
                .find(|marker| Path::new(project_root).join(marker).exists())
                .cloned(),
            enabled: record.enabled,
            sort_order: 0,
            revision: record.revision,
            runtime_status: runtime.state,
            capability_count: runtime.capability_count,
            error: runtime.error,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }

    fn default_jdtls_record(&self, project_id: &str) -> LanguageServerRecord {
        let request = self.default_jdtls_request(project_id, false);
        let data_path = self.managed_jdtls_data_path(project_id);
        LanguageServerRecord {
            language_server_id: project_language_server_id(project_id, JDTLS_KIND),
            display_name: format!("{JDTLS_DISPLAY_NAME} [{project_id}]"),
            config: LanguageServerConfig {
                version: 2,
                project_id: Some(project_id.into()),
                server_kind: Some(JDTLS_KIND.into()),
                command: request.executable,
                arguments: vec!["-data".into(), data_path.to_string_lossy().into_owned()],
                language_ids: vec!["java".into()],
                root_markers: request.root_markers,
                initialization_options: json!({}),
                configuration: json!({}),
                environment: BTreeMap::new(),
                startup_timeout_seconds: request.startup_timeout_seconds,
                request_timeout_seconds: request.request_timeout_seconds,
                java_home: request.java_home,
                data_path_mode: Some("managed".into()),
                data_path: Some(data_path.to_string_lossy().into_owned()),
                vtsls_executable: None,
                node_path: None,
                typescript_sdk_mode: None,
                typescript_sdk_path: None,
                max_ts_server_memory: None,
                basedpyright_executable: None,
                python_path: None,
                diagnostic_mode: None,
                type_checking_mode: None,
                config_path_mode: None,
                config_path: None,
            },
            enabled: false,
            sort_order: 0,
            revision: 0,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn unscoped_jdtls_dto(&self) -> LanguageServerDto {
        let detected_executable = detect_jdtls();
        let detected_java_home = detect_java_home();
        LanguageServerDto {
            language_server_id: JDTLS_KIND.into(),
            kind: JDTLS_KIND.into(),
            display_name: JDTLS_DISPLAY_NAME.into(),
            description: JDTLS_DESCRIPTION.into(),
            language: "Java".into(),
            detected: detected_executable.is_some(),
            detected_version: None,
            detected_executable: detected_executable.clone(),
            detected_java_home: detected_java_home.clone(),
            vtsls: None,
            basedpyright: None,
            project_configurable: false,
            command: detected_executable.unwrap_or_else(|| "jdtls".into()),
            arguments: Vec::new(),
            language_ids: vec!["java".into()],
            root_markers: DEFAULT_ROOT_MARKERS
                .iter()
                .map(|value| (*value).into())
                .collect(),
            initialization_options: json!({}),
            environment_keys: Vec::new(),
            startup_timeout_seconds: 120,
            request_timeout_seconds: 30,
            java_home: detected_java_home,
            data_path_mode: "managed".into(),
            data_path: String::new(),
            matched_root_marker: None,
            enabled: false,
            sort_order: 0,
            revision: 0,
            runtime_status: suncode_core::LanguageServerRuntimeState::NotStarted,
            capability_count: 0,
            error: None,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn default_jdtls_request(
        &self,
        project_id: &str,
        enabled: bool,
    ) -> JdtlsLanguageServerWriteRequest {
        JdtlsLanguageServerWriteRequest {
            executable: detect_jdtls().unwrap_or_else(|| "jdtls".into()),
            java_home: detect_java_home(),
            data_path_mode: "managed".into(),
            data_path: Some(
                self.managed_jdtls_data_path(project_id)
                    .to_string_lossy()
                    .into(),
            ),
            root_markers: DEFAULT_ROOT_MARKERS
                .iter()
                .map(|value| (*value).into())
                .collect(),
            environment: None,
            startup_timeout_seconds: 120,
            request_timeout_seconds: 30,
            enabled,
        }
    }

    async fn vtsls_dto(
        &self,
        project_id: &str,
        project_root: &str,
        record: Option<LanguageServerRecord>,
    ) -> SdkResult<LanguageServerDto> {
        let detected_executable = detect_vtsls();
        let detected_node_path = detect_node();
        let detected_node_version = detected_node_path.as_deref().and_then(detect_node_version);
        let detected = detected_executable.is_some()
            && detected_node_version
                .as_deref()
                .and_then(node_major_version)
                .is_some_and(|major| major >= 18);
        let default = self.default_vtsls_record(project_id);
        let record = record.unwrap_or(default);
        let runtime = self
            .state
            .agent
            .language_server_runtime_status(Some(project_id), &record)
            .await;
        let executable = record.config.vtsls_executable.clone().unwrap_or_else(|| {
            detected_executable
                .clone()
                .unwrap_or_else(|| "vtsls".into())
        });
        let node_path = record
            .config
            .node_path
            .clone()
            .unwrap_or_else(|| detected_node_path.clone().unwrap_or_else(|| "node".into()));
        Ok(LanguageServerDto {
            language_server_id: VTSLS_KIND.into(),
            kind: VTSLS_KIND.into(),
            display_name: VTSLS_DISPLAY_NAME.into(),
            description: VTSLS_DESCRIPTION.into(),
            language: "JavaScript / TypeScript".into(),
            detected,
            detected_version: None,
            detected_executable,
            detected_java_home: None,
            vtsls: Some(VtslsLanguageServerConfigurationDto {
                executable: executable.clone(),
                node_path,
                detected_node_path,
                detected_node_version,
                typescript_sdk_mode: record
                    .config
                    .typescript_sdk_mode
                    .clone()
                    .unwrap_or_else(|| "workspace-preferred".into()),
                typescript_sdk_path: record.config.typescript_sdk_path.clone(),
                max_ts_server_memory: record.config.max_ts_server_memory.unwrap_or(3072),
            }),
            basedpyright: None,
            project_configurable: true,
            command: executable,
            arguments: record.config.arguments.clone(),
            language_ids: VTSLS_LANGUAGE_IDS
                .iter()
                .map(|value| (*value).into())
                .collect(),
            root_markers: record.config.root_markers.clone(),
            initialization_options: json!({}),
            environment_keys: record.config.environment.keys().cloned().collect(),
            startup_timeout_seconds: record.config.startup_timeout_seconds,
            request_timeout_seconds: record.config.request_timeout_seconds,
            java_home: None,
            data_path_mode: String::new(),
            data_path: String::new(),
            matched_root_marker: record
                .config
                .root_markers
                .iter()
                .find(|marker| Path::new(project_root).join(marker).exists())
                .cloned(),
            enabled: record.enabled,
            sort_order: 10,
            revision: record.revision,
            runtime_status: runtime.state,
            capability_count: runtime.capability_count,
            error: runtime.error,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }

    fn default_vtsls_record(&self, project_id: &str) -> LanguageServerRecord {
        let request = self.default_vtsls_request(false);
        let executable = request.executable.clone();
        let server_script =
            resolve_vtsls_script(&executable).unwrap_or_else(|| PathBuf::from(&executable));
        LanguageServerRecord {
            language_server_id: project_language_server_id(project_id, VTSLS_KIND),
            display_name: format!("{VTSLS_DISPLAY_NAME} [{project_id}]"),
            config: LanguageServerConfig {
                version: 3,
                project_id: Some(project_id.into()),
                server_kind: Some(VTSLS_KIND.into()),
                command: request.node_path.clone(),
                arguments: vec![
                    server_script.to_string_lossy().into_owned(),
                    "--stdio".into(),
                ],
                language_ids: VTSLS_LANGUAGE_IDS
                    .iter()
                    .map(|value| (*value).into())
                    .collect(),
                root_markers: request.root_markers.clone(),
                initialization_options: json!({}),
                configuration: vtsls_configuration(
                    "workspace-preferred",
                    None,
                    request.max_ts_server_memory,
                ),
                environment: BTreeMap::new(),
                startup_timeout_seconds: request.startup_timeout_seconds,
                request_timeout_seconds: request.request_timeout_seconds,
                java_home: None,
                data_path_mode: None,
                data_path: None,
                vtsls_executable: Some(executable),
                node_path: Some(request.node_path),
                typescript_sdk_mode: Some("workspace-preferred".into()),
                typescript_sdk_path: None,
                max_ts_server_memory: Some(request.max_ts_server_memory),
                basedpyright_executable: None,
                python_path: None,
                diagnostic_mode: None,
                type_checking_mode: None,
                config_path_mode: None,
                config_path: None,
            },
            enabled: false,
            sort_order: 10,
            revision: 0,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn unscoped_vtsls_dto(&self) -> LanguageServerDto {
        let detected_executable = detect_vtsls();
        let detected_node_path = detect_node();
        let detected_node_version = detected_node_path.as_deref().and_then(detect_node_version);
        let detected = detected_executable.is_some()
            && detected_node_version
                .as_deref()
                .and_then(node_major_version)
                .is_some_and(|major| major >= 18);
        let executable = detected_executable
            .clone()
            .unwrap_or_else(|| "vtsls".into());
        let node_path = detected_node_path.clone().unwrap_or_else(|| "node".into());
        LanguageServerDto {
            language_server_id: VTSLS_KIND.into(),
            kind: VTSLS_KIND.into(),
            display_name: VTSLS_DISPLAY_NAME.into(),
            description: VTSLS_DESCRIPTION.into(),
            language: "JavaScript / TypeScript".into(),
            detected,
            detected_version: None,
            detected_executable,
            detected_java_home: None,
            vtsls: Some(VtslsLanguageServerConfigurationDto {
                executable: executable.clone(),
                node_path,
                detected_node_path,
                detected_node_version,
                typescript_sdk_mode: "workspace-preferred".into(),
                typescript_sdk_path: None,
                max_ts_server_memory: 3072,
            }),
            basedpyright: None,
            project_configurable: false,
            command: executable,
            arguments: vec!["--stdio".into()],
            language_ids: VTSLS_LANGUAGE_IDS
                .iter()
                .map(|value| (*value).into())
                .collect(),
            root_markers: VTSLS_ROOT_MARKERS
                .iter()
                .map(|value| (*value).into())
                .collect(),
            initialization_options: json!({}),
            environment_keys: Vec::new(),
            startup_timeout_seconds: 120,
            request_timeout_seconds: 30,
            java_home: None,
            data_path_mode: String::new(),
            data_path: String::new(),
            matched_root_marker: None,
            enabled: false,
            sort_order: 10,
            revision: 0,
            runtime_status: suncode_core::LanguageServerRuntimeState::NotStarted,
            capability_count: 0,
            error: None,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn default_vtsls_request(&self, enabled: bool) -> VtslsLanguageServerWriteRequest {
        VtslsLanguageServerWriteRequest {
            executable: detect_vtsls().unwrap_or_else(|| "vtsls".into()),
            node_path: detect_node().unwrap_or_else(|| "node".into()),
            typescript_sdk_mode: "workspace-preferred".into(),
            typescript_sdk_path: None,
            root_markers: VTSLS_ROOT_MARKERS
                .iter()
                .map(|value| (*value).into())
                .collect(),
            max_ts_server_memory: 3072,
            environment: None,
            startup_timeout_seconds: 120,
            request_timeout_seconds: 30,
            enabled,
        }
    }

    async fn basedpyright_dto(
        &self,
        project_id: &str,
        project_root: &str,
        record: Option<LanguageServerRecord>,
    ) -> SdkResult<LanguageServerDto> {
        let detected_executable = detect_basedpyright(Some(Path::new(project_root)));
        let detected_python_path = detect_python(Some(Path::new(project_root)));
        let detected_python_version = detected_python_path
            .as_deref()
            .and_then(detect_python_version);
        let default = self.default_basedpyright_record(project_id, project_root);
        let record = record.unwrap_or(default);
        let runtime = self
            .state
            .agent
            .language_server_runtime_status(Some(project_id), &record)
            .await;
        let executable = record
            .config
            .basedpyright_executable
            .clone()
            .unwrap_or_else(|| {
                detected_executable
                    .clone()
                    .unwrap_or_else(|| "basedpyright-langserver".into())
            });
        let python_path = record
            .config
            .python_path
            .clone()
            .or_else(|| detected_python_path.clone());
        Ok(LanguageServerDto {
            language_server_id: BASEDPYRIGHT_KIND.into(),
            kind: BASEDPYRIGHT_KIND.into(),
            display_name: BASEDPYRIGHT_DISPLAY_NAME.into(),
            description: BASEDPYRIGHT_DESCRIPTION.into(),
            language: "Python".into(),
            detected: detected_executable.is_some(),
            detected_version: None,
            detected_executable,
            detected_java_home: None,
            vtsls: None,
            basedpyright: Some(BasedpyrightLanguageServerConfigurationDto {
                executable: executable.clone(),
                python_path,
                detected_python_path,
                detected_python_version,
                diagnostic_mode: record
                    .config
                    .diagnostic_mode
                    .clone()
                    .unwrap_or_else(|| "openFilesOnly".into()),
                type_checking_mode: record
                    .config
                    .type_checking_mode
                    .clone()
                    .unwrap_or_else(|| "recommended".into()),
                config_path_mode: record
                    .config
                    .config_path_mode
                    .clone()
                    .unwrap_or_else(|| "automatic".into()),
                config_path: record.config.config_path.clone(),
            }),
            project_configurable: true,
            command: executable,
            arguments: vec!["--stdio".into()],
            language_ids: vec!["python".into()],
            root_markers: record.config.root_markers.clone(),
            initialization_options: json!({}),
            environment_keys: record.config.environment.keys().cloned().collect(),
            startup_timeout_seconds: record.config.startup_timeout_seconds,
            request_timeout_seconds: record.config.request_timeout_seconds,
            java_home: None,
            data_path_mode: String::new(),
            data_path: String::new(),
            matched_root_marker: record
                .config
                .root_markers
                .iter()
                .find(|marker| Path::new(project_root).join(marker).exists())
                .cloned(),
            enabled: record.enabled,
            sort_order: 20,
            revision: record.revision,
            runtime_status: runtime.state,
            capability_count: runtime.capability_count,
            error: runtime.error,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }

    fn default_basedpyright_record(
        &self,
        project_id: &str,
        project_root: &str,
    ) -> LanguageServerRecord {
        let request = self.default_basedpyright_request(project_id, project_root, false);
        let executable = request.executable.clone();
        LanguageServerRecord {
            language_server_id: project_language_server_id(project_id, BASEDPYRIGHT_KIND),
            display_name: format!("{BASEDPYRIGHT_DISPLAY_NAME} [{project_id}]"),
            config: LanguageServerConfig {
                version: 4,
                project_id: Some(project_id.into()),
                server_kind: Some(BASEDPYRIGHT_KIND.into()),
                command: executable.clone(),
                arguments: vec!["--stdio".into()],
                language_ids: vec!["python".into()],
                root_markers: request.root_markers.clone(),
                initialization_options: json!({}),
                configuration: basedpyright_configuration(
                    request.python_path.as_deref(),
                    "openFilesOnly",
                    "recommended",
                    "automatic",
                    None,
                ),
                environment: BTreeMap::new(),
                startup_timeout_seconds: request.startup_timeout_seconds,
                request_timeout_seconds: request.request_timeout_seconds,
                java_home: None,
                data_path_mode: None,
                data_path: None,
                vtsls_executable: None,
                node_path: None,
                typescript_sdk_mode: None,
                typescript_sdk_path: None,
                max_ts_server_memory: None,
                basedpyright_executable: Some(executable),
                python_path: request.python_path,
                diagnostic_mode: Some("openFilesOnly".into()),
                type_checking_mode: Some("recommended".into()),
                config_path_mode: Some("automatic".into()),
                config_path: None,
            },
            enabled: false,
            sort_order: 20,
            revision: 0,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn unscoped_basedpyright_dto(&self) -> LanguageServerDto {
        let detected_executable = detect_basedpyright(None);
        let detected_python_path = detect_python(None);
        let detected_python_version = detected_python_path
            .as_deref()
            .and_then(detect_python_version);
        let executable = detected_executable
            .clone()
            .unwrap_or_else(|| "basedpyright-langserver".into());
        LanguageServerDto {
            language_server_id: BASEDPYRIGHT_KIND.into(),
            kind: BASEDPYRIGHT_KIND.into(),
            display_name: BASEDPYRIGHT_DISPLAY_NAME.into(),
            description: BASEDPYRIGHT_DESCRIPTION.into(),
            language: "Python".into(),
            detected: detected_executable.is_some(),
            detected_version: None,
            detected_executable,
            detected_java_home: None,
            vtsls: None,
            basedpyright: Some(BasedpyrightLanguageServerConfigurationDto {
                executable: executable.clone(),
                python_path: detected_python_path.clone(),
                detected_python_path,
                detected_python_version,
                diagnostic_mode: "openFilesOnly".into(),
                type_checking_mode: "recommended".into(),
                config_path_mode: "automatic".into(),
                config_path: None,
            }),
            project_configurable: false,
            command: executable,
            arguments: vec!["--stdio".into()],
            language_ids: vec!["python".into()],
            root_markers: BASEDPYRIGHT_ROOT_MARKERS
                .iter()
                .map(|value| (*value).into())
                .collect(),
            initialization_options: json!({}),
            environment_keys: Vec::new(),
            startup_timeout_seconds: 60,
            request_timeout_seconds: 30,
            java_home: None,
            data_path_mode: String::new(),
            data_path: String::new(),
            matched_root_marker: None,
            enabled: false,
            sort_order: 20,
            revision: 0,
            runtime_status: suncode_core::LanguageServerRuntimeState::NotStarted,
            capability_count: 0,
            error: None,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn default_basedpyright_request(
        &self,
        _project_id: &str,
        project_root: &str,
        enabled: bool,
    ) -> BasedpyrightLanguageServerWriteRequest {
        BasedpyrightLanguageServerWriteRequest {
            executable: detect_basedpyright(Some(Path::new(project_root)))
                .unwrap_or_else(|| "basedpyright-langserver".into()),
            python_path: detect_python(Some(Path::new(project_root))),
            root_markers: BASEDPYRIGHT_ROOT_MARKERS
                .iter()
                .map(|value| (*value).into())
                .collect(),
            diagnostic_mode: "openFilesOnly".into(),
            type_checking_mode: "recommended".into(),
            config_path_mode: "automatic".into(),
            config_path: None,
            environment: None,
            startup_timeout_seconds: 60,
            request_timeout_seconds: 30,
            enabled,
        }
    }

    fn managed_jdtls_data_path(&self, project_id: &str) -> PathBuf {
        self.state
            .data_dir
            .join("lsp")
            .join(JDTLS_KIND)
            .join(project_id)
    }
}

fn validate_jdtls_request(request: &JdtlsLanguageServerWriteRequest) -> SdkResult<()> {
    if request.executable.trim().is_empty()
        || request.executable.chars().count() > 2_048
        || request.executable.contains('\0')
    {
        return Err(BusinessError::invalid(
            "jdtls executable is required and must not exceed 2048 characters",
        ));
    }
    if !(1..=120).contains(&request.startup_timeout_seconds)
        || !(1..=600).contains(&request.request_timeout_seconds)
    {
        return Err(BusinessError::invalid(
            "Language server timeout is outside the supported range",
        ));
    }
    Ok(())
}

fn legacy_request(request: LanguageServerWriteRequest) -> JdtlsLanguageServerWriteRequest {
    let java_home = request
        .environment
        .as_ref()
        .and_then(|changes| changes.set.get("JAVA_HOME").cloned());
    let data_path = request
        .arguments
        .windows(2)
        .find(|values| values[0] == "-data")
        .map(|values| values[1].clone());
    JdtlsLanguageServerWriteRequest {
        executable: request.command,
        java_home,
        data_path_mode: if data_path.is_some() {
            "custom"
        } else {
            "managed"
        }
        .into(),
        data_path,
        root_markers: request.root_markers,
        environment: request.environment,
        startup_timeout_seconds: request.startup_timeout_seconds,
        request_timeout_seconds: request.request_timeout_seconds,
        enabled: request.enabled,
    }
}

fn jdtls_request_from_record(
    record: &LanguageServerRecord,
    enabled: bool,
) -> JdtlsLanguageServerWriteRequest {
    JdtlsLanguageServerWriteRequest {
        executable: record.config.command.clone(),
        java_home: record.config.java_home.clone(),
        data_path_mode: record
            .config
            .data_path_mode
            .clone()
            .unwrap_or_else(|| "managed".into()),
        data_path: record.config.data_path.clone(),
        root_markers: record.config.root_markers.clone(),
        environment: None,
        startup_timeout_seconds: record.config.startup_timeout_seconds,
        request_timeout_seconds: record.config.request_timeout_seconds,
        enabled,
    }
}

fn vtsls_request_from_record(
    record: &LanguageServerRecord,
    enabled: bool,
) -> VtslsLanguageServerWriteRequest {
    VtslsLanguageServerWriteRequest {
        executable: record
            .config
            .vtsls_executable
            .clone()
            .unwrap_or_else(|| "vtsls".into()),
        node_path: record
            .config
            .node_path
            .clone()
            .unwrap_or_else(|| record.config.command.clone()),
        typescript_sdk_mode: record
            .config
            .typescript_sdk_mode
            .clone()
            .unwrap_or_else(|| "workspace-preferred".into()),
        typescript_sdk_path: record.config.typescript_sdk_path.clone(),
        root_markers: record.config.root_markers.clone(),
        max_ts_server_memory: record.config.max_ts_server_memory.unwrap_or(3072),
        environment: None,
        startup_timeout_seconds: record.config.startup_timeout_seconds,
        request_timeout_seconds: record.config.request_timeout_seconds,
        enabled,
    }
}

fn basedpyright_request_from_record(
    record: &LanguageServerRecord,
    enabled: bool,
) -> BasedpyrightLanguageServerWriteRequest {
    BasedpyrightLanguageServerWriteRequest {
        executable: record
            .config
            .basedpyright_executable
            .clone()
            .unwrap_or_else(|| record.config.command.clone()),
        python_path: record.config.python_path.clone(),
        root_markers: record.config.root_markers.clone(),
        diagnostic_mode: record
            .config
            .diagnostic_mode
            .clone()
            .unwrap_or_else(|| "openFilesOnly".into()),
        type_checking_mode: record
            .config
            .type_checking_mode
            .clone()
            .unwrap_or_else(|| "recommended".into()),
        config_path_mode: record
            .config
            .config_path_mode
            .clone()
            .unwrap_or_else(|| "automatic".into()),
        config_path: record.config.config_path.clone(),
        environment: None,
        startup_timeout_seconds: record.config.startup_timeout_seconds,
        request_timeout_seconds: record.config.request_timeout_seconds,
        enabled,
    }
}

fn validate_basedpyright_request(
    request: &BasedpyrightLanguageServerWriteRequest,
) -> SdkResult<()> {
    if request.executable.trim().is_empty()
        || request.executable.chars().count() > 2_048
        || request.executable.contains('\0')
    {
        return Err(BusinessError::invalid(
            "basedpyright executable is required and must not exceed 2048 characters",
        ));
    }
    normalize_diagnostic_mode(&request.diagnostic_mode)?;
    normalize_type_checking_mode(&request.type_checking_mode)?;
    let config_path_mode = normalize_config_path_mode(&request.config_path_mode)?;
    if let Some(python_path) = request
        .python_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if python_path.chars().count() > 2_048
            || python_path.contains('\0')
            || !Path::new(python_path).is_absolute()
        {
            return Err(BusinessError::invalid(
                "Python interpreter path must be absolute and must not exceed 2048 characters",
            ));
        }
    }
    if config_path_mode == "custom" {
        let value = request
            .config_path
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                BusinessError::invalid("Custom basedpyright configuration location is required")
            })?;
        if value.chars().count() > 2_048 || value.contains('\0') || !Path::new(value).is_absolute()
        {
            return Err(BusinessError::invalid(
                "Custom basedpyright configuration location must be absolute",
            ));
        }
    }
    if !(1..=120).contains(&request.startup_timeout_seconds)
        || !(1..=600).contains(&request.request_timeout_seconds)
    {
        return Err(BusinessError::invalid(
            "Language server timeout is outside the supported range",
        ));
    }
    Ok(())
}

fn normalize_diagnostic_mode(value: &str) -> SdkResult<&'static str> {
    match value.trim() {
        "" | "openFilesOnly" => Ok("openFilesOnly"),
        "workspace" => Ok("workspace"),
        _ => Err(BusinessError::invalid(
            "basedpyright diagnostic mode must be openFilesOnly or workspace",
        )),
    }
}

fn normalize_type_checking_mode(value: &str) -> SdkResult<&'static str> {
    match value.trim() {
        "" | "recommended" => Ok("recommended"),
        "off" => Ok("off"),
        "basic" => Ok("basic"),
        "standard" => Ok("standard"),
        "strict" => Ok("strict"),
        "all" => Ok("all"),
        _ => Err(BusinessError::invalid(
            "basedpyright type checking mode is unsupported",
        )),
    }
}

fn normalize_config_path_mode(value: &str) -> SdkResult<&'static str> {
    match value.trim() {
        "" | "automatic" => Ok("automatic"),
        "custom" => Ok("custom"),
        _ => Err(BusinessError::invalid(
            "basedpyright configuration location mode must be automatic or custom",
        )),
    }
}

fn optional_trimmed(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn basedpyright_configuration(
    python_path: Option<&str>,
    diagnostic_mode: &str,
    type_checking_mode: &str,
    config_path_mode: &str,
    config_path: Option<&str>,
) -> serde_json::Value {
    let mut settings = json!({
        "basedpyright": {
            "analysis": {
                "diagnosticMode": diagnostic_mode,
                "typeCheckingMode": type_checking_mode
            }
        }
    });
    if let Some(python_path) = python_path {
        settings["python"]["pythonPath"] = serde_json::Value::String(python_path.into());
    }
    if config_path_mode == "custom" {
        settings["basedpyright"]["analysis"]["configFilePath"] =
            serde_json::Value::String(config_path.unwrap_or_default().into());
    }
    settings
}

fn validate_vtsls_request(request: &VtslsLanguageServerWriteRequest) -> SdkResult<()> {
    if request.executable.trim().is_empty()
        || request.executable.chars().count() > 2_048
        || request.executable.contains('\0')
    {
        return Err(BusinessError::invalid(
            "vtsls executable is required and must not exceed 2048 characters",
        ));
    }
    if request.node_path.trim().is_empty()
        || request.node_path.chars().count() > 2_048
        || request.node_path.contains('\0')
    {
        return Err(BusinessError::invalid(
            "Node.js executable is required and must not exceed 2048 characters",
        ));
    }
    let node_version = detect_node_version(request.node_path.trim()).ok_or_else(|| {
        BusinessError::invalid("Node.js executable could not be run to determine its version")
    })?;
    if node_major_version(&node_version).is_none_or(|major| major < 18) {
        return Err(BusinessError::invalid("vtsls requires Node.js 18 or newer"));
    }
    let mode = normalize_typescript_sdk_mode(&request.typescript_sdk_mode)?;
    if mode == "custom" {
        let value = request
            .typescript_sdk_path
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| BusinessError::invalid("Custom TypeScript SDK directory is required"))?;
        let path = Path::new(value);
        if !path.is_absolute() || !path.join("tsserverlibrary.js").is_file() {
            return Err(BusinessError::invalid(
                "Custom TypeScript SDK directory must be absolute and contain tsserverlibrary.js",
            ));
        }
    }
    if !(512..=16_384).contains(&request.max_ts_server_memory) {
        return Err(BusinessError::invalid(
            "TypeScript server memory must be between 512 and 16384 MiB",
        ));
    }
    if !(1..=120).contains(&request.startup_timeout_seconds)
        || !(1..=600).contains(&request.request_timeout_seconds)
    {
        return Err(BusinessError::invalid(
            "Language server timeout is outside the supported range",
        ));
    }
    Ok(())
}

fn normalize_typescript_sdk_mode(value: &str) -> SdkResult<&'static str> {
    match value.trim() {
        "" | "workspace-preferred" => Ok("workspace-preferred"),
        "bundled" => Ok("bundled"),
        "custom" => Ok("custom"),
        _ => Err(BusinessError::invalid(
            "TypeScript SDK mode must be workspace-preferred, bundled, or custom",
        )),
    }
}

fn vtsls_configuration(mode: &str, sdk_path: Option<&str>, memory: u64) -> serde_json::Value {
    let mut settings = json!({
        "vtsls": {
            "autoUseWorkspaceTsdk": mode == "workspace-preferred"
        },
        "typescript": {
            "tsserver": {
                "maxTsServerMemory": memory
            }
        }
    });
    if mode == "custom" {
        settings["vtsls"]["typescript"]["globalTsdk"] =
            serde_json::Value::String(sdk_path.unwrap_or_default().to_string());
    }
    settings
}

fn apply_environment_changes(
    mut current: BTreeMap<String, String>,
    changes: Option<LanguageServerEnvironmentChanges>,
) -> SdkResult<BTreeMap<String, String>> {
    let Some(changes) = changes else {
        return Ok(current);
    };
    let mut seen = HashSet::new();
    for key in changes.set.keys().chain(changes.remove.iter()) {
        let normalized = key.to_ascii_uppercase();
        if !seen.insert(normalized) {
            return Err(BusinessError::invalid(
                "Language server environment changes contain duplicate keys",
            ));
        }
    }
    for key in changes.remove {
        if let Some(existing) = current
            .keys()
            .find(|existing| existing.eq_ignore_ascii_case(&key))
            .cloned()
        {
            current.remove(&existing);
        }
    }
    for (key, value) in changes.set {
        if let Some(existing) = current
            .keys()
            .find(|existing| existing.eq_ignore_ascii_case(&key))
            .cloned()
        {
            current.remove(&existing);
        }
        current.insert(key, value);
    }
    Ok(current)
}

fn project_language_server_id(project_id: &str, kind: &str) -> String {
    format!("{project_id}:{kind}")
}

fn detect_jdtls() -> Option<String> {
    let name = if cfg!(windows) { "jdtls.bat" } else { "jdtls" };
    find_on_path(name)
        .or_else(|| {
            ["/opt/homebrew/bin", "/usr/local/bin"]
                .into_iter()
                .map(|directory| Path::new(directory).join(name))
                .find(|candidate| candidate.is_file())
        })
        .map(|path| path.to_string_lossy().into_owned())
}

fn detect_vtsls() -> Option<String> {
    let name = if cfg!(windows) { "vtsls.cmd" } else { "vtsls" };
    find_on_path(name)
        .or_else(|| {
            ["/opt/homebrew/bin", "/usr/local/bin"]
                .into_iter()
                .map(|directory| Path::new(directory).join(name))
                .find(|candidate| candidate.is_file())
        })
        .or_else(|| detect_nvm_binary(name))
        .map(|path| path.to_string_lossy().into_owned())
}

fn detect_basedpyright(project_root: Option<&Path>) -> Option<String> {
    let name = if cfg!(windows) {
        "basedpyright-langserver.exe"
    } else {
        "basedpyright-langserver"
    };
    project_root
        .into_iter()
        .flat_map(|root| {
            if cfg!(windows) {
                vec![
                    root.join(".venv/Scripts").join(name),
                    root.join("venv/Scripts").join(name),
                ]
            } else {
                vec![
                    root.join(".venv/bin").join(name),
                    root.join("venv/bin").join(name),
                ]
            }
        })
        .find(|candidate| candidate.is_file())
        .or_else(|| find_on_path(name))
        .or_else(|| {
            ["/opt/homebrew/bin", "/usr/local/bin"]
                .into_iter()
                .map(|directory| Path::new(directory).join(name))
                .find(|candidate| candidate.is_file())
        })
        .or_else(|| {
            std::env::var_os("HOME").and_then(|home| {
                [".local/bin", ".pyenv/shims"]
                    .into_iter()
                    .map(|directory| Path::new(&home).join(directory).join(name))
                    .find(|candidate| candidate.is_file())
            })
        })
        .map(|path| path.to_string_lossy().into_owned())
}

fn detect_python(project_root: Option<&Path>) -> Option<String> {
    let name = if cfg!(windows) {
        "python.exe"
    } else {
        "python"
    };
    project_root
        .into_iter()
        .flat_map(|root| {
            if cfg!(windows) {
                vec![
                    root.join(".venv/Scripts").join(name),
                    root.join("venv/Scripts").join(name),
                ]
            } else {
                vec![
                    root.join(".venv/bin").join(name),
                    root.join("venv/bin").join(name),
                ]
            }
        })
        .find(|candidate| candidate.is_file())
        .or_else(|| {
            std::env::var_os("CONDA_PREFIX").and_then(|directory| {
                let bin = if cfg!(windows) { "Scripts" } else { "bin" };
                let candidate = Path::new(&directory).join(bin).join(name);
                candidate.is_file().then_some(candidate)
            })
        })
        .or_else(|| {
            std::env::var_os("VIRTUAL_ENV").and_then(|directory| {
                let bin = if cfg!(windows) { "Scripts" } else { "bin" };
                let candidate = Path::new(&directory).join(bin).join(name);
                candidate.is_file().then_some(candidate)
            })
        })
        .or_else(|| {
            find_on_path(if cfg!(windows) {
                "python.exe"
            } else {
                "python3"
            })
        })
        .or_else(|| find_on_path(name))
        .or_else(|| {
            ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"]
                .into_iter()
                .flat_map(|directory| {
                    let directory = Path::new(directory);
                    [directory.join("python3"), directory.join(name)]
                })
                .find(|candidate| candidate.is_file())
        })
        .or_else(|| {
            std::env::var_os("HOME").and_then(|home| {
                [
                    ".pyenv/shims/python",
                    ".local/bin/python3",
                    ".local/bin/python",
                ]
                .into_iter()
                .map(|relative| Path::new(&home).join(relative))
                .find(|candidate| candidate.is_file())
            })
        })
        .map(|path| path.to_string_lossy().into_owned())
}

fn detect_python_version(python_path: &str) -> Option<String> {
    let output = std::process::Command::new(python_path)
        .arg("--version")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = if output.stdout.is_empty() {
        output.stderr
    } else {
        output.stdout
    };
    String::from_utf8(value)
        .ok()
        .map(|value| value.trim().trim_start_matches("Python ").to_string())
        .filter(|value| !value.is_empty())
}

fn detect_node() -> Option<String> {
    let name = if cfg!(windows) { "node.exe" } else { "node" };
    find_on_path(name)
        .or_else(|| {
            ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"]
                .into_iter()
                .map(|directory| Path::new(directory).join(name))
                .find(|candidate| candidate.is_file())
        })
        .or_else(|| detect_nvm_binary(name))
        .map(|path| path.to_string_lossy().into_owned())
}

fn detect_nvm_binary(name: &str) -> Option<PathBuf> {
    let root = std::env::var_os("HOME")
        .map(PathBuf::from)?
        .join(".nvm")
        .join("versions")
        .join("node");
    std::fs::read_dir(root)
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let candidate = entry.path().join("bin").join(name);
            candidate.is_file().then(|| {
                let version = entry
                    .file_name()
                    .to_string_lossy()
                    .trim_start_matches('v')
                    .split('.')
                    .take(3)
                    .map(|part| part.parse::<u64>().unwrap_or(0))
                    .collect::<Vec<_>>();
                (
                    (
                        *version.first().unwrap_or(&0),
                        *version.get(1).unwrap_or(&0),
                        *version.get(2).unwrap_or(&0),
                    ),
                    candidate,
                )
            })
        })
        .max_by_key(|(version, _)| *version)
        .map(|(_, candidate)| candidate)
}

fn detect_node_version(node_path: &str) -> Option<String> {
    let output = std::process::Command::new(node_path)
        .arg("--version")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().trim_start_matches('v').to_string())
        .filter(|value| !value.is_empty())
}

fn node_major_version(value: &str) -> Option<u64> {
    value.split('.').next()?.parse().ok()
}

fn resolve_vtsls_script(executable: &str) -> Option<PathBuf> {
    let configured = Path::new(executable);
    let executable = if configured.components().count() > 1 {
        configured.to_path_buf()
    } else {
        find_on_path(executable)?
    };
    let canonical = std::fs::canonicalize(&executable).unwrap_or_else(|_| executable.clone());
    if canonical
        .extension()
        .is_some_and(|extension| extension == "js")
    {
        return Some(canonical);
    }
    let parent = executable.parent()?;
    [
        parent.join("../lib/node_modules/@vtsls/language-server/bin/vtsls.js"),
        parent.join("../node_modules/@vtsls/language-server/bin/vtsls.js"),
        parent.join("node_modules/@vtsls/language-server/bin/vtsls.js"),
    ]
    .into_iter()
    .find(|candidate| candidate.is_file())
    .and_then(|candidate| std::fs::canonicalize(&candidate).ok().or(Some(candidate)))
}

fn detect_java_home() -> Option<String> {
    static DETECTED: OnceLock<Option<String>> = OnceLock::new();
    DETECTED.get_or_init(detect_java_home_uncached).clone()
}

fn detect_java_home_uncached() -> Option<String> {
    std::env::var("JAVA_HOME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(detect_macos_java_home)
        .or_else(|| {
            detect_java_home_under(
                Path::new("/Library/Java/JavaVirtualMachines"),
                "Contents/Home",
            )
        })
        .or_else(|| detect_java_home_under(Path::new("/usr/lib/jvm"), ""))
}

fn detect_macos_java_home() -> Option<String> {
    let executable = Path::new("/usr/libexec/java_home");
    if !executable.is_file() {
        return None;
    }
    let output = std::process::Command::new(executable).output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn detect_java_home_under(root: &Path, suffix: &str) -> Option<String> {
    let mut candidates = std::fs::read_dir(root)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path().join(suffix))
        .filter(|candidate| candidate.join("bin").join("java").is_file())
        .collect::<Vec<_>>();
    candidates.sort();
    candidates
        .pop()
        .map(|path| path.to_string_lossy().into_owned())
}

fn command_exists(command: &str) -> bool {
    let path = Path::new(command);
    if path.components().count() > 1 {
        path.is_file()
    } else {
        find_on_path(command).is_some()
    }
}

fn find_on_path(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|value| {
        std::env::split_paths(&value)
            .map(|directory| directory.join(name))
            .find(|candidate| candidate.is_file())
    })
}

fn require_idempotency_key(value: &str) -> SdkResult<()> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 256 {
        return Err(BusinessError::invalid(
            "idempotency key is required and must not exceed 256 characters",
        ));
    }
    Ok(())
}
