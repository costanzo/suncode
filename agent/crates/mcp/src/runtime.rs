//! Per-project MCP connection runtime: reconciliation against the desired
//! server configuration, tool discovery, and calls.
//!
//! Each `(project, server)` pair owns one slot. A slot's generation increases
//! on every reconciliation, so late results from a replaced connection are
//! discarded and approvals are bound to the connection the user saw.

use crate::spec::{ServerSource, ServerSpec, ServerTransport, WorkingDirectory};
use crate::{
    connect, tool_result_value, Connection, ConnectionConfig, ToolDefinition as RemoteTool,
};
use futures_util::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use suncode_common::tool::ToolDefinition;
use suncode_common::BusinessError;
use tokio::sync::Mutex as AsyncMutex;
use tokio_util::sync::CancellationToken;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct McpLoadProgress {
    pub total: usize,
    pub settled: usize,
    pub connected: usize,
    pub failed: usize,
    pub loading: bool,
}

#[derive(Clone)]
pub struct McpRuntime {
    inner: Arc<Inner>,
}

struct Inner {
    source: Arc<dyn ServerSource>,
    projects: AsyncMutex<HashMap<String, ProjectRuntime>>,
    closed: AtomicBool,
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

pub(crate) struct McpApprovalTarget {
    pub label: String,
    pub server_id: String,
    pub remote_name: String,
    pub generation: u64,
}

impl McpRuntime {
    pub fn new(source: Arc<dyn ServerSource>) -> Self {
        Self {
            inner: Arc::new(Inner {
                source,
                projects: AsyncMutex::new(HashMap::new()),
                closed: AtomicBool::new(false),
            }),
        }
    }

    pub async fn activate_project(
        &self,
        project_id: &str,
        root: &Path,
    ) -> Result<(), BusinessError> {
        self.ensure_open()?;
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
        let servers = self.inner.source.servers()?;
        let manager = self.clone();
        let project_id = project_id.to_string();
        tokio::spawn(async move {
            let semaphore = Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_CONNECTIONS));
            let tasks = servers
                .into_iter()
                .filter(|server| server.enabled)
                .map(|server| {
                    let manager = manager.clone();
                    let semaphore = semaphore.clone();
                    let project_id = project_id.clone();
                    let server_id = server.server_id.clone();
                    tokio::spawn(async move {
                        let Ok(_permit) = semaphore.acquire_owned().await else {
                            return;
                        };
                        if let Err(error) = manager
                            .reconcile_project_server(project_id.clone(), Some(server))
                            .await
                        {
                            manager
                                .fail_current(&project_id, &server_id, error.to_string())
                                .await;
                        }
                    })
                })
                .collect::<Vec<_>>();
            for task in tasks {
                let _ = task.await;
            }
        });
        Ok(())
    }

    pub async fn progress(&self, project_id: &str) -> McpLoadProgress {
        let total = self
            .inner
            .source
            .servers()
            .map(|servers| servers.into_iter().filter(|server| server.enabled).count())
            .unwrap_or(0);
        if total == 0 {
            return McpLoadProgress {
                total: 0,
                settled: 0,
                connected: 0,
                failed: 0,
                loading: false,
            };
        }
        let projects = self.inner.projects.lock().await;
        let mut connected = 0;
        let mut failed = 0;
        if let Some(project) = projects.get(project_id) {
            for slot in project.slots.values() {
                match slot.state {
                    McpRuntimeState::Connected => connected += 1,
                    McpRuntimeState::Failed => failed += 1,
                    _ => {}
                }
            }
        }
        let settled = (connected + failed).min(total);
        McpLoadProgress {
            total,
            settled,
            connected,
            failed,
            loading: settled < total,
        }
    }

    /// Applies the current configuration of `server_id` to every active
    /// project, including removal when the server no longer exists.
    pub async fn reconcile_server(&self, server_id: &str) -> Result<(), BusinessError> {
        if self.inner.source.server(server_id)?.is_some() {
            return self.reconcile_all(server_id).await;
        }
        let projects = self
            .inner
            .projects
            .lock()
            .await
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        for project_id in projects {
            self.reconcile_project_server_with_id(project_id, server_id.to_string(), None)
                .await?;
        }
        Ok(())
    }

    async fn reconcile_all(&self, server_id: &str) -> Result<(), BusinessError> {
        let desired = self.inner.source.server(server_id)?;
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

    pub async fn reconcile_remote_servers(&self) -> Result<(), BusinessError> {
        let server_ids = self
            .inner
            .source
            .servers()?
            .into_iter()
            .filter(|server| server.enabled && server.transport.is_remote())
            .map(|server| server.server_id)
            .collect::<Vec<_>>();
        for server_id in server_ids {
            self.reconcile_all(&server_id).await?;
        }
        Ok(())
    }

    pub async fn retry(&self, project_id: &str, server_id: &str) -> Result<(), BusinessError> {
        let desired = self
            .inner
            .source
            .server(server_id)?
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
        desired: Option<ServerSpec>,
    ) -> Result<(), BusinessError> {
        let server_id = desired
            .as_ref()
            .map(|server| server.server_id.clone())
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
        desired: Option<ServerSpec>,
    ) -> Result<(), BusinessError> {
        self.ensure_open()?;
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
            if let Some(desired) = desired.as_ref() {
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
                    Some((
                        generation,
                        cancellation,
                        project.root.clone(),
                        desired.clone(),
                    )),
                    retired,
                )
            } else {
                project.slots.remove(&server_id);
                (None, retired)
            }
        };
        if let Some(retired) = retired {
            retired.close().await;
        }
        let Some((generation, cancellation, root, desired)) = prepared else {
            return Ok(());
        };
        if !desired.enabled {
            return Ok(());
        }
        let config = self.connection_config(&desired, &root)?;
        let connected = tokio::select! {
            _ = cancellation.cancelled() => return Ok(()),
            result = connect(config) => result,
        };
        let mut connected = match connected {
            Ok(connected) => connected,
            Err(error) => {
                self.install_failure(&project_id, &server_id, generation, error.message)
                    .await;
                return Ok(());
            }
        };
        let connection = connected.connection.clone();
        let retained = {
            let mut projects = self.inner.projects.lock().await;
            if self.inner.closed.load(Ordering::Acquire) {
                false
            } else if let Some(slot) = projects
                .get_mut(&project_id)
                .and_then(|project| project.slots.get_mut(&server_id))
                .filter(|slot| slot.generation == generation && slot.revision == desired.revision)
            {
                slot.connection = Some(connection.clone());
                true
            } else {
                false
            }
        };
        if !retained {
            connected.connection.close().await;
            return Ok(());
        }
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

    pub async fn shutdown(&self) {
        if self.inner.closed.swap(true, Ordering::AcqRel) {
            return;
        }
        let connections = {
            let mut projects = self.inner.projects.lock().await;
            projects
                .drain()
                .flat_map(|(_, project)| project.slots.into_values())
                .filter_map(|slot| {
                    slot.cancellation.cancel();
                    slot.connection
                })
                .collect::<Vec<_>>()
        };
        for connection in connections {
            connection.close().await;
        }
    }

    fn ensure_open(&self) -> Result<(), BusinessError> {
        if self.inner.closed.load(Ordering::Acquire) {
            Err(BusinessError::new(
                "agent_shutting_down",
                "agent shutdown is in progress",
            ))
        } else {
            Ok(())
        }
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
            let Ok(Some(desired)) = self.inner.source.server(server_id) else {
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

    async fn fail_current(&self, project_id: &str, server_id: &str, message: String) {
        let mut projects = self.inner.projects.lock().await;
        if let Some(slot) = projects
            .get_mut(project_id)
            .and_then(|project| project.slots.get_mut(server_id))
            .filter(|slot| slot.state == McpRuntimeState::Connecting)
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
        server: &ServerSpec,
        project_root: &Path,
    ) -> Result<ConnectionConfig, BusinessError> {
        Ok(match &server.transport {
            ServerTransport::Stdio {
                command,
                arguments,
                working_directory,
                environment,
                startup_timeout,
                request_timeout,
            } => ConnectionConfig::Stdio {
                command: command.clone(),
                arguments: arguments.clone(),
                working_directory: match working_directory {
                    WorkingDirectory::Project => project_root.to_path_buf(),
                    WorkingDirectory::ApplicationData => self.inner.source.application_data(),
                },
                environment: environment.clone(),
                startup_timeout: *startup_timeout,
                request_timeout: *request_timeout,
            },
            ServerTransport::StreamableHttp {
                url,
                headers,
                startup_timeout,
                request_timeout,
            } => ConnectionConfig::StreamableHttp {
                url: url.clone(),
                headers: headers.clone(),
                startup_timeout: *startup_timeout,
                request_timeout: *request_timeout,
                tls: self.inner.source.network()?,
            },
        })
    }

    pub(crate) async fn catalog(&self, project_id: &str) -> Vec<ToolDefinition> {
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
            .map(|tool| ToolDefinition {
                name: tool.exposed_name,
                description: tool.description,
                parameters: tool.parameters,
            })
            .collect()
    }

    pub async fn status(&self, project_id: Option<&str>, server: &ServerSpec) -> McpRuntimeStatus {
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
            .and_then(|project| project.slots.get(&server.server_id))
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

    pub(crate) async fn approval_target(
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

    pub(crate) async fn call(
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
        Ok(tool_result_value(result))
    }
}

pub fn is_mcp_tool(name: &str) -> bool {
    name.starts_with("mcp__")
}

fn build_catalog(
    server: &ServerSpec,
    generation: u64,
    tools: Vec<RemoteTool>,
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
            server_id: server.server_id.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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

    mod provider {
        use super::*;
        use crate::{ToolDefinition as Remote, ToolResult};
        use futures_util::future::BoxFuture as FutureBox;
        use std::sync::atomic::AtomicUsize;
        use suncode_common::tool::{
            BoxFuture, Cancellation, PathScopes, ScopedPath, ToolContext, ToolInvocation,
            ToolOutput, ToolProvider,
        };

        struct NoSource;
        impl ServerSource for NoSource {
            fn servers(&self) -> Result<Vec<ServerSpec>, BusinessError> {
                Ok(Vec::new())
            }
            fn server(&self, _: &str) -> Result<Option<ServerSpec>, BusinessError> {
                Ok(None)
            }
            fn network(&self) -> Result<crate::TlsConfig, BusinessError> {
                Ok(crate::TlsConfig::default())
            }
            fn application_data(&self) -> PathBuf {
                PathBuf::new()
            }
        }

        struct EchoConnection(AtomicUsize);
        impl Connection for EchoConnection {
            fn list_tools(&self) -> FutureBox<'_, Result<Vec<Remote>, BusinessError>> {
                Box::pin(async { Ok(Vec::new()) })
            }
            fn call_tool(
                &self,
                name: String,
                _arguments: Value,
                _cancellation: CancellationToken,
            ) -> FutureBox<'_, Result<ToolResult, BusinessError>> {
                self.0.fetch_add(1, Ordering::SeqCst);
                Box::pin(async move {
                    Ok(ToolResult {
                        content: vec![format!("called {name}")],
                        structured_content: None,
                        is_error: false,
                        truncated: false,
                    })
                })
            }
            fn close(&self) -> FutureBox<'_, ()> {
                Box::pin(async {})
            }
            fn is_closed(&self) -> bool {
                false
            }
        }

        struct Paths;
        impl PathScopes for Paths {
            fn resolve(&self, path: &str) -> Result<ScopedPath, BusinessError> {
                Ok(ScopedPath::Project {
                    relative: path.into(),
                })
            }
        }
        struct Sink;
        impl ToolOutput for Sink {
            fn chunk(&self, _: &str, _: &[u8]) {}
        }
        struct Never;
        impl Cancellation for Never {
            fn is_cancelled(&self) -> bool {
                false
            }
            fn cancelled(&self) -> BoxFuture<'_, ()> {
                Box::pin(std::future::pending())
            }
        }

        /// A runtime with one connected server exposing `search`.
        async fn connected() -> (McpRuntime, Arc<EchoConnection>) {
            let runtime = McpRuntime::new(Arc::new(NoSource));
            let connection = Arc::new(EchoConnection(AtomicUsize::new(0)));
            let schema = json!({"type":"object","properties":{"q":{"type":"string"}}});
            runtime.inner.projects.lock().await.insert(
                "project".into(),
                ProjectRuntime {
                    root: PathBuf::new(),
                    slots: HashMap::from([(
                        "server".into(),
                        RuntimeSlot {
                            generation: 3,
                            revision: 1,
                            state: McpRuntimeState::Connected,
                            error: None,
                            cancellation: CancellationToken::new(),
                            connection: Some(connection.clone()),
                            tools: vec![CatalogTool {
                                exposed_name: "mcp__gh__search".into(),
                                remote_name: "search".into(),
                                description: "Search".into(),
                                parameters: schema,
                                server_id: "server".into(),
                                server_display_name: "GitHub".into(),
                                generation: 3,
                            }],
                        },
                    )]),
                },
            );
            (runtime, connection)
        }

        async fn execute(runtime: &McpRuntime, pin: Option<Value>) -> Result<Value, BusinessError> {
            ToolProvider::execute(
                runtime,
                ToolInvocation {
                    name: "mcp__gh__search",
                    arguments: json!({"q":"rust"}),
                    approval_pin: pin,
                },
                ToolContext {
                    project_id: "project",
                    project_root: Path::new("/"),
                    call_key: "turn:call",
                    timeout_ceiling: None,
                    paths: &Paths,
                    output: Arc::new(Sink),
                    cancellation: &Never,
                },
            )
            .await
        }

        #[tokio::test]
        async fn approval_target_names_the_server_and_pins_the_generation() {
            let (runtime, _) = connected().await;
            let target = ToolProvider::approval_target(
                &runtime,
                "project",
                "mcp__gh__search",
                &json!({"q":"rust"}),
            )
            .await
            .unwrap();
            assert_eq!(target.operation, "GitHub / search");
            assert_eq!(target.arguments["serverId"], "server");
            assert_eq!(target.arguments["toolName"], "search");
            assert_eq!(target.pin, Some(json!(3)));
            let invalid = ToolProvider::approval_target(
                &runtime,
                "project",
                "mcp__gh__search",
                &json!({"q":1}),
            )
            .await
            .unwrap_err();
            assert_eq!(invalid.code, "mcp_arguments_invalid");
        }

        #[tokio::test]
        async fn execution_honors_the_pin_and_rejects_a_replaced_connection() {
            let (runtime, connection) = connected().await;
            let result = execute(&runtime, Some(json!(3))).await.unwrap();
            assert_eq!(result["content"][0], "called search");
            let stale = execute(&runtime, Some(json!(2))).await.unwrap_err();
            assert_eq!(stale.code, "mcp_tool_unavailable");
            assert_eq!(connection.0.load(Ordering::SeqCst), 1);
            let missing =
                ToolProvider::approval_target(&runtime, "project", "mcp__gh__other", &json!({}))
                    .await
                    .unwrap_err();
            assert_eq!(missing.code, "mcp_tool_unavailable");
        }
    }
}
