use super::*;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use suncode_data::{McpServerRecord, McpTransportConfig, McpWorkingDirectory};
use suncode_mcp::{Connection, ConnectionConfig, TlsConfig};

const MAX_PROJECT_TOOLS: usize = 256;
const MAX_EXPOSED_NAME_BYTES: usize = 64;
const MAX_CONCURRENT_CONNECTIONS: usize = 4;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum McpRuntimeState {
    NotStarted,
    Disabled,
    Connecting,
    Connected,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct McpRuntimeStatus {
    pub state: McpRuntimeState,
    pub tool_count: usize,
    pub error: Option<String>,
}

#[derive(Clone)]
pub(super) struct McpManager {
    inner: Arc<Inner>,
}

struct Inner {
    store: Store,
    application_data: PathBuf,
    projects: AsyncMutex<HashMap<String, ProjectRuntime>>,
}

struct ProjectRuntime {
    root: PathBuf,
    slots: HashMap<String, RuntimeSlot>,
}

struct RuntimeSlot {
    generation: u64,
    revision: u64,
    state: McpRuntimeState,
    error: Option<String>,
    cancellation: CancellationToken,
    connection: Option<Arc<dyn Connection>>,
    tools: Vec<CatalogTool>,
}

#[derive(Clone)]
struct CatalogTool {
    exposed_name: String,
    remote_name: String,
    description: String,
    parameters: Value,
    server_id: String,
    server_display_name: String,
    generation: u64,
}

pub(super) struct McpApprovalTarget {
    pub label: String,
    pub server_id: String,
    pub remote_name: String,
    pub generation: u64,
}

impl McpManager {
    pub(super) fn new(store: Store, application_data: PathBuf) -> Self {
        Self {
            inner: Arc::new(Inner {
                store,
                application_data,
                projects: AsyncMutex::new(HashMap::new()),
            }),
        }
    }

    pub(super) async fn activate_project(
        &self,
        project_id: &str,
        root: &Path,
    ) -> Result<(), BusinessError> {
        let created = {
            let mut projects = self.inner.projects.lock().await;
            if let Some(runtime) = projects.get_mut(project_id) {
                runtime.root = root.to_path_buf();
                false
            } else {
                projects.insert(
                    project_id.to_string(),
                    ProjectRuntime {
                        root: root.to_path_buf(),
                        slots: HashMap::new(),
                    },
                );
                true
            }
        };
        if !created {
            return Ok(());
        }
        let servers = self.inner.store.mcp_servers()?;
        let results = stream::iter(servers)
            .map(|server| self.reconcile_project_server(project_id.to_string(), Some(server)))
            .buffer_unordered(MAX_CONCURRENT_CONNECTIONS)
            .collect::<Vec<_>>()
            .await;
        for result in results {
            result?;
        }
        Ok(())
    }

    pub(super) async fn reconcile_all(&self, server_id: &str) -> Result<(), BusinessError> {
        let desired = self.inner.store.mcp_server_by_id(server_id)?;
        let projects = self
            .inner
            .projects
            .lock()
            .await
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        let futures = projects
            .into_iter()
            .map(|project_id| self.reconcile_project_server(project_id, desired.clone()));
        let results = join_all(futures).await;
        for result in results {
            result?;
        }
        Ok(())
    }

    pub(super) async fn retry(
        &self,
        project_id: &str,
        server_id: &str,
    ) -> Result<(), BusinessError> {
        let desired = self
            .inner
            .store
            .mcp_server_by_id(server_id)?
            .ok_or_else(|| BusinessError::missing("mcp_server"))?;
        if !desired.enabled {
            return Err(BusinessError::invalid(
                "disabled MCP server cannot be retried",
            ));
        }
        if !self.inner.projects.lock().await.contains_key(project_id) {
            return Err(BusinessError::missing("project_runtime"));
        }
        self.reconcile_project_server(project_id.to_string(), Some(desired))
            .await
    }

    async fn reconcile_project_server(
        &self,
        project_id: String,
        desired: Option<McpServerRecord>,
    ) -> Result<(), BusinessError> {
        let server_id = desired
            .as_ref()
            .map(|server| server.mcp_server_id.clone())
            .ok_or_else(|| {
                BusinessError::invalid("MCP server id is required for reconciliation")
            })?;
        self.reconcile_project_server_with_id(project_id, server_id, desired)
            .await
    }

    async fn reconcile_project_server_with_id(
        &self,
        project_id: String,
        server_id: String,
        desired: Option<McpServerRecord>,
    ) -> Result<(), BusinessError> {
        let (prepared, retired) = {
            let mut projects = self.inner.projects.lock().await;
            let project = projects
                .get_mut(&project_id)
                .ok_or_else(|| BusinessError::missing("project_runtime"))?;
            let previous_generation = project
                .slots
                .get(&server_id)
                .map(|slot| slot.generation)
                .unwrap_or(0);
            let retired = project.slots.get_mut(&server_id).and_then(|slot| {
                slot.cancellation.cancel();
                slot.tools.clear();
                slot.connection.take()
            });
            if desired.is_none() {
                project.slots.remove(&server_id);
                (None, retired)
            } else {
                let desired = desired.as_ref().expect("checked above");
                let generation = previous_generation.saturating_add(1);
                let cancellation = CancellationToken::new();
                project.slots.insert(
                    server_id.clone(),
                    RuntimeSlot {
                        generation,
                        revision: desired.revision,
                        state: if desired.enabled {
                            McpRuntimeState::Connecting
                        } else {
                            McpRuntimeState::Disabled
                        },
                        error: None,
                        cancellation: cancellation.clone(),
                        connection: None,
                        tools: Vec::new(),
                    },
                );
                (
                    Some((generation, cancellation, project.root.clone())),
                    retired,
                )
            }
        };
        if let Some(retired) = retired {
            tokio::spawn(async move { retired.close().await });
        }
        let Some((generation, cancellation, root)) = prepared else {
            return Ok(());
        };
        let desired = desired.expect("checked above");
        if !desired.enabled {
            return Ok(());
        }
        let config = self.connection_config(&desired, &root)?;
        let connected = tokio::select! {
            _ = cancellation.cancelled() => return Ok(()),
            result = suncode_mcp::connect(config) => result,
        };
        let mut connected = match connected {
            Ok(connected) => connected,
            Err(error) => {
                self.install_failure(&project_id, &server_id, generation, error.message)
                    .await;
                return Ok(());
            }
        };
        let definitions = match connected.connection.list_tools().await {
            Ok(tools) => build_catalog(&desired, generation, tools),
            Err(error) => Err(error),
        };
        let definitions = match definitions {
            Ok(definitions) => definitions,
            Err(error) => {
                connected.connection.close().await;
                self.install_failure(&project_id, &server_id, generation, error.message)
                    .await;
                return Ok(());
            }
        };
        let connection = connected.connection.clone();
        let installed = {
            let mut projects = self.inner.projects.lock().await;
            if let Some(slot) = projects
                .get_mut(&project_id)
                .and_then(|project| project.slots.get_mut(&server_id))
                .filter(|slot| slot.generation == generation && slot.revision == desired.revision)
            {
                slot.connection = Some(connection);
                slot.tools = definitions;
                slot.state = McpRuntimeState::Connected;
                slot.error = None;
                true
            } else {
                false
            }
        };
        if !installed {
            connected.connection.close().await;
            return Ok(());
        }
        let manager = self.clone();
        tokio::spawn(async move {
            while connected.tool_list_changes.recv().await.is_some() {
                manager
                    .refresh_tools(&project_id, &server_id, generation)
                    .await;
            }
            manager
                .mark_disconnected(&project_id, &server_id, generation)
                .await;
        });
        Ok(())
    }

    async fn refresh_tools(&self, project_id: &str, server_id: &str, generation: u64) {
        let (connection, desired) = {
            let projects = self.inner.projects.lock().await;
            let Some(slot) = projects
                .get(project_id)
                .and_then(|project| project.slots.get(server_id))
                .filter(|slot| slot.generation == generation)
            else {
                return;
            };
            let Some(connection) = slot.connection.clone() else {
                return;
            };
            let Ok(Some(desired)) = self.inner.store.mcp_server_by_id(server_id) else {
                return;
            };
            (connection, desired)
        };
        match connection
            .list_tools()
            .await
            .and_then(|tools| build_catalog(&desired, generation, tools))
        {
            Ok(tools) => {
                let mut projects = self.inner.projects.lock().await;
                if let Some(slot) = projects
                    .get_mut(project_id)
                    .and_then(|project| project.slots.get_mut(server_id))
                    .filter(|slot| slot.generation == generation)
                {
                    slot.tools = tools;
                    slot.state = McpRuntimeState::Connected;
                    slot.error = None;
                }
            }
            Err(error) => {
                self.install_failure(project_id, server_id, generation, error.message)
                    .await;
            }
        }
    }

    async fn install_failure(
        &self,
        project_id: &str,
        server_id: &str,
        generation: u64,
        message: String,
    ) {
        let mut projects = self.inner.projects.lock().await;
        if let Some(slot) = projects
            .get_mut(project_id)
            .and_then(|project| project.slots.get_mut(server_id))
            .filter(|slot| slot.generation == generation)
        {
            slot.connection = None;
            slot.tools.clear();
            slot.state = McpRuntimeState::Failed;
            slot.error = Some(message.chars().take(500).collect());
        }
    }

    async fn mark_disconnected(&self, project_id: &str, server_id: &str, generation: u64) {
        let mut projects = self.inner.projects.lock().await;
        if let Some(slot) = projects
            .get_mut(project_id)
            .and_then(|project| project.slots.get_mut(server_id))
            .filter(|slot| slot.generation == generation)
        {
            slot.connection = None;
            slot.tools.clear();
            slot.state = McpRuntimeState::Failed;
            slot.error = Some("MCP server connection closed".into());
        }
    }

    fn connection_config(
        &self,
        server: &McpServerRecord,
        project_root: &Path,
    ) -> Result<ConnectionConfig, BusinessError> {
        match &server.transport {
            McpTransportConfig::Stdio {
                command,
                arguments,
                working_directory,
                environment,
                startup_timeout_seconds,
                request_timeout_seconds,
                ..
            } => Ok(ConnectionConfig::Stdio {
                command: command.clone(),
                arguments: arguments.clone(),
                working_directory: match working_directory {
                    McpWorkingDirectory::Project => project_root.to_path_buf(),
                    McpWorkingDirectory::ApplicationData => self.inner.application_data.clone(),
                },
                environment: environment.clone(),
                startup_timeout: Duration::from_secs(*startup_timeout_seconds),
                request_timeout: Duration::from_secs(*request_timeout_seconds),
            }),
            McpTransportConfig::StreamableHttp {
                url,
                headers,
                startup_timeout_seconds,
                request_timeout_seconds,
                ..
            } => Ok(ConnectionConfig::StreamableHttp {
                url: url.clone(),
                headers: headers.clone(),
                startup_timeout: Duration::from_secs(*startup_timeout_seconds),
                request_timeout: Duration::from_secs(*request_timeout_seconds),
                tls: self.tls_config()?,
            }),
        }
    }

    fn tls_config(&self) -> Result<TlsConfig, BusinessError> {
        let settings = self.inner.store.settings(None, None)?;
        let value = |key: &str| {
            settings
                .iter()
                .find(|setting| setting.key == key)
                .map(|setting| &setting.value)
        };
        Ok(TlsConfig {
            verify_certificates: value("verify_https_certificates")
                .and_then(Value::as_bool)
                .unwrap_or(true),
            use_system_certificates: value("use_system_certificates")
                .and_then(Value::as_bool)
                .unwrap_or(true),
            certificate_path: value("certificate_path")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .map(PathBuf::from),
        })
    }

    pub(super) async fn catalog(&self, project_id: &str) -> Vec<suncode_llm::ToolDefinition> {
        let projects = self.inner.projects.lock().await;
        let mut tools = projects
            .get(project_id)
            .into_iter()
            .flat_map(|project| project.slots.values())
            .filter(|slot| slot.state == McpRuntimeState::Connected)
            .flat_map(|slot| slot.tools.iter())
            .cloned()
            .collect::<Vec<_>>();
        drop(projects);
        tools.sort_by(|left, right| left.exposed_name.cmp(&right.exposed_name));
        tools
            .into_iter()
            .take(MAX_PROJECT_TOOLS)
            .map(|tool| suncode_llm::ToolDefinition {
                name: tool.exposed_name,
                description: tool.description,
                parameters: tool.parameters,
            })
            .collect()
    }

    pub(super) async fn status(
        &self,
        project_id: Option<&str>,
        server: &McpServerRecord,
    ) -> McpRuntimeStatus {
        if !server.enabled {
            return McpRuntimeStatus {
                state: McpRuntimeState::Disabled,
                tool_count: 0,
                error: None,
            };
        }
        let Some(project_id) = project_id else {
            return McpRuntimeStatus {
                state: McpRuntimeState::NotStarted,
                tool_count: 0,
                error: None,
            };
        };
        let projects = self.inner.projects.lock().await;
        let Some(slot) = projects
            .get(project_id)
            .and_then(|project| project.slots.get(&server.mcp_server_id))
        else {
            return McpRuntimeStatus {
                state: McpRuntimeState::NotStarted,
                tool_count: 0,
                error: None,
            };
        };
        McpRuntimeStatus {
            state: slot.state.clone(),
            tool_count: slot.tools.len(),
            error: slot.error.clone(),
        }
    }

    async fn resolve(
        &self,
        project_id: &str,
        exposed_name: &str,
    ) -> Option<(CatalogTool, Arc<dyn Connection>)> {
        let projects = self.inner.projects.lock().await;
        projects
            .get(project_id)?
            .slots
            .values()
            .filter(|slot| slot.state == McpRuntimeState::Connected)
            .find_map(|slot| {
                let tool = slot
                    .tools
                    .iter()
                    .find(|tool| tool.exposed_name == exposed_name)?;
                let connection = slot.connection.clone()?;
                (tool.generation == slot.generation && !connection.is_closed())
                    .then(|| (tool.clone(), connection))
            })
    }

    pub(super) async fn approval_target(
        &self,
        project_id: &str,
        exposed_name: &str,
        arguments: &Value,
    ) -> Result<Option<McpApprovalTarget>, BusinessError> {
        self.resolve(project_id, exposed_name)
            .await
            .map(|(tool, _)| {
                validate_tool_arguments(&tool.parameters, arguments)?;
                Ok(McpApprovalTarget {
                    label: format!("{} / {}", tool.server_display_name, tool.remote_name),
                    server_id: tool.server_id,
                    remote_name: tool.remote_name,
                    generation: tool.generation,
                })
            })
            .transpose()
    }

    pub(super) async fn call(
        &self,
        project_id: &str,
        exposed_name: &str,
        arguments: Value,
        cancellation: CancellationToken,
        expected_generation: Option<u64>,
    ) -> Result<Value, BusinessError> {
        let (tool, connection) = self
            .resolve(project_id, exposed_name)
            .await
            .ok_or_else(|| {
                BusinessError::new(
                    "mcp_tool_unavailable",
                    "MCP tool is no longer available for this project",
                )
            })?;
        if expected_generation.is_some_and(|generation| generation != tool.generation) {
            return Err(BusinessError::new(
                "mcp_tool_unavailable",
                "MCP tool changed after approval and must be requested again",
            ));
        }
        validate_tool_arguments(&tool.parameters, &arguments)?;
        let result = connection
            .call_tool(tool.remote_name, arguments, cancellation)
            .await?;
        Ok(suncode_mcp::tool_result_value(result))
    }
}

pub(super) fn is_mcp_tool(name: &str) -> bool {
    name.starts_with("mcp__")
}

fn build_catalog(
    server: &McpServerRecord,
    generation: u64,
    tools: Vec<suncode_mcp::ToolDefinition>,
) -> Result<Vec<CatalogTool>, BusinessError> {
    let mut names = HashSet::new();
    let mut result = Vec::with_capacity(tools.len());
    for tool in tools {
        jsonschema::validator_for(&tool.input_schema).map_err(|_| {
            BusinessError::new(
                "mcp_schema_invalid",
                "MCP tool input schema is not a valid JSON Schema",
            )
        })?;
        let exposed_name = exposed_name(&server.tool_prefix, &tool.remote_name);
        if !names.insert(exposed_name.to_ascii_lowercase()) {
            return Err(BusinessError::new(
                "mcp_tool_name_conflict",
                "MCP server exposes conflicting tool names",
            ));
        }
        result.push(CatalogTool {
            exposed_name,
            remote_name: tool.remote_name,
            description: if tool.description.trim().is_empty() {
                format!("MCP tool provided by {}", server.display_name)
            } else {
                tool.description
            },
            parameters: tool.input_schema,
            server_id: server.mcp_server_id.clone(),
            server_display_name: server.display_name.clone(),
            generation,
        });
    }
    Ok(result)
}

fn validate_tool_arguments(schema: &Value, arguments: &Value) -> Result<(), BusinessError> {
    let validator = jsonschema::validator_for(schema).map_err(|_| {
        BusinessError::new(
            "mcp_schema_invalid",
            "MCP tool input schema is not a valid JSON Schema",
        )
    })?;
    validator.validate(arguments).map_err(|_| {
        BusinessError::new(
            "mcp_arguments_invalid",
            "MCP tool arguments do not match the advertised schema",
        )
    })
}

fn exposed_name(prefix: &str, remote_name: &str) -> String {
    let remote = remote_name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '_' | '-') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    let candidate = format!("mcp__{prefix}__{remote}");
    if candidate.len() <= MAX_EXPOSED_NAME_BYTES {
        return candidate;
    }
    let digest = Sha256::digest(candidate.as_bytes());
    let suffix = format!(
        "_{:02x}{:02x}{:02x}{:02x}",
        digest[0], digest[1], digest[2], digest[3]
    );
    let mut boundary = MAX_EXPOSED_NAME_BYTES - suffix.len();
    while !candidate.is_char_boundary(boundary) {
        boundary -= 1;
    }
    format!("{}{}", &candidate[..boundary], suffix)
}

impl Agent {
    pub async fn activate_mcp_project(
        &self,
        project_id: &str,
        project_root: &Path,
    ) -> Result<(), BusinessError> {
        self.mcp.activate_project(project_id, project_root).await
    }

    pub async fn reconcile_mcp_server(&self, server_id: &str) -> Result<(), BusinessError> {
        if self.store.mcp_server_by_id(server_id)?.is_none() {
            let projects = self
                .mcp
                .inner
                .projects
                .lock()
                .await
                .keys()
                .cloned()
                .collect::<Vec<_>>();
            for project_id in projects {
                self.mcp
                    .reconcile_project_server_with_id(project_id, server_id.to_string(), None)
                    .await?;
            }
            return Ok(());
        }
        self.mcp.reconcile_all(server_id).await
    }

    pub async fn retry_mcp_server(
        &self,
        project_id: &str,
        server_id: &str,
    ) -> Result<(), BusinessError> {
        self.mcp.retry(project_id, server_id).await
    }

    pub async fn mcp_runtime_status(
        &self,
        project_id: Option<&str>,
        server: &McpServerRecord,
    ) -> McpRuntimeStatus {
        self.mcp.status(project_id, server).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_names_are_namespaced_and_bounded() {
        assert_eq!(
            exposed_name("github", "search_code"),
            "mcp__github__search_code"
        );
        let long = exposed_name("a_very_long_server_prefix", &"x".repeat(100));
        assert!(long.len() <= MAX_EXPOSED_NAME_BYTES);
        assert_eq!(
            long,
            exposed_name("a_very_long_server_prefix", &"x".repeat(100))
        );
    }

    #[test]
    fn advertised_schema_validates_tool_arguments() {
        let schema = json!({
            "type": "object",
            "properties": { "query": { "type": "string" } },
            "required": ["query"],
            "additionalProperties": false
        });
        assert!(validate_tool_arguments(&schema, &json!({"query": "rust"})).is_ok());
        assert_eq!(
            validate_tool_arguments(&schema, &json!({"query": 42}))
                .unwrap_err()
                .code,
            "mcp_arguments_invalid"
        );
    }
}
