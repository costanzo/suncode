use super::*;
use serde_json::{json, Map};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicBool, Ordering},
};
use suncode_data::LanguageServerRecord;
use suncode_lsp::{Client, ConnectionConfig, PositionEncoding};
use url::Url;

const MAX_NORMALIZED_ITEMS: usize = 500;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LanguageServerRuntimeState {
    NotStarted,
    Disabled,
    Starting,
    Indexing,
    Ready,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LanguageServerRuntimeStatus {
    pub state: LanguageServerRuntimeState,
    pub capability_count: usize,
    pub error: Option<String>,
}

#[derive(Clone)]
pub(super) struct LanguageServerManager {
    inner: Arc<Inner>,
}

struct Inner {
    store: Store,
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
    state: LanguageServerRuntimeState,
    error: Option<String>,
    client: Option<Client>,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum SemanticOperation {
    Diagnostics,
    Definition,
    References,
    Hover,
    Symbols,
}

pub(super) struct SemanticRequest<'a> {
    pub project_id: &'a str,
    pub project_root: &'a Path,
    pub scope_root: &'a Path,
    pub display_path: &'a str,
    pub relative_path: &'a str,
    pub language_id: &'a str,
    pub text: &'a str,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub operation: SemanticOperation,
}

impl LanguageServerManager {
    pub(super) fn new(store: Store) -> Self {
        Self {
            inner: Arc::new(Inner {
                store,
                projects: AsyncMutex::new(HashMap::new()),
                closed: AtomicBool::new(false),
            }),
        }
    }

    pub(super) async fn activate_project(
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
        for server in self
            .inner
            .store
            .language_servers()?
            .into_iter()
            .filter(|server| server.enabled)
        {
            let manager = self.clone();
            let project_id = project_id.to_string();
            tokio::spawn(async move {
                let _ = manager.start_server(&project_id, server).await;
            });
        }
        Ok(())
    }

    pub(super) async fn reconcile_all(
        &self,
        language_server_id: &str,
    ) -> Result<(), BusinessError> {
        let desired = self.inner.store.language_server_by_id(language_server_id)?;
        let projects = self
            .inner
            .projects
            .lock()
            .await
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        for project_id in projects {
            match desired.clone() {
                Some(server) if server.enabled => {
                    // Persistence is already committed before reconciliation. A launch failure is
                    // retained on the runtime slot and surfaced through status instead of turning
                    // a successful configuration mutation into an ambiguous SDK failure.
                    let _ = self.start_server(&project_id, server).await;
                }
                _ => self.retire_server(&project_id, language_server_id).await,
            }
        }
        Ok(())
    }

    pub(super) async fn retry(
        &self,
        project_id: &str,
        language_server_id: &str,
    ) -> Result<(), BusinessError> {
        let server = self
            .inner
            .store
            .language_server_by_id(language_server_id)?
            .ok_or_else(|| BusinessError::missing("language_server"))?;
        if !server.enabled {
            return Err(BusinessError::new(
                "language_server_disabled",
                "Language server is disabled",
            ));
        }
        self.start_server(project_id, server).await
    }

    pub(super) async fn status(
        &self,
        project_id: Option<&str>,
        server: &LanguageServerRecord,
    ) -> LanguageServerRuntimeStatus {
        if !server.enabled {
            return LanguageServerRuntimeStatus {
                state: LanguageServerRuntimeState::Disabled,
                capability_count: 0,
                error: None,
            };
        }
        let Some(project_id) = project_id else {
            return LanguageServerRuntimeStatus {
                state: LanguageServerRuntimeState::NotStarted,
                capability_count: 0,
                error: None,
            };
        };
        let (state, error, client) = {
            let projects = self.inner.projects.lock().await;
            let Some(slot) = projects
                .get(project_id)
                .and_then(|runtime| runtime.slots.get(&server.language_server_id))
            else {
                return LanguageServerRuntimeStatus {
                    state: LanguageServerRuntimeState::NotStarted,
                    capability_count: 0,
                    error: None,
                };
            };
            (slot.state.clone(), slot.error.clone(), slot.client.clone())
        };
        let capability_count = match client {
            Some(client) => capability_count(&client.capabilities().await),
            None => 0,
        };
        LanguageServerRuntimeStatus {
            state,
            capability_count,
            error,
        }
    }

    async fn start_server(
        &self,
        project_id: &str,
        server: LanguageServerRecord,
    ) -> Result<(), BusinessError> {
        self.ensure_open()?;
        let prepared = {
            let mut projects = self.inner.projects.lock().await;
            let runtime = projects
                .get_mut(project_id)
                .ok_or_else(|| BusinessError::missing("project_runtime"))?;
            let marker_matches = server.config.root_markers.is_empty()
                || server
                    .config
                    .root_markers
                    .iter()
                    .any(|marker| runtime.root.join(marker).exists());
            let previous = runtime.slots.remove(&server.language_server_id);
            let generation = previous
                .as_ref()
                .map(|slot| slot.generation.saturating_add(1))
                .unwrap_or(1);
            let old_client = previous.and_then(|slot| slot.client);
            runtime.slots.insert(
                server.language_server_id.clone(),
                RuntimeSlot {
                    generation,
                    revision: server.revision,
                    state: if marker_matches {
                        LanguageServerRuntimeState::Starting
                    } else {
                        LanguageServerRuntimeState::NotStarted
                    },
                    error: None,
                    client: None,
                },
            );
            marker_matches.then(|| (runtime.root.clone(), generation, old_client))
        };
        let Some((root, generation, old_client)) = prepared else {
            return Ok(());
        };
        if let Some(client) = old_client {
            client.close().await;
        }
        let result = suncode_lsp::connect(
            ConnectionConfig {
                command: server.config.command.clone(),
                arguments: server.config.arguments.clone(),
                working_directory: root.clone(),
                environment: server.config.environment.clone(),
                initialization_options: server.config.initialization_options.clone(),
                startup_timeout: Duration::from_secs(server.config.startup_timeout_seconds),
                request_timeout: Duration::from_secs(server.config.request_timeout_seconds),
            },
            &root,
        )
        .await;
        let stale_client = {
            let mut projects = self.inner.projects.lock().await;
            if let Some(slot) = projects
                .get_mut(project_id)
                .and_then(|runtime| runtime.slots.get_mut(&server.language_server_id))
            {
                if slot.generation != generation || slot.revision != server.revision {
                    result.ok()
                } else {
                    match result {
                        Ok(client) => {
                            slot.state = LanguageServerRuntimeState::Indexing;
                            slot.error = None;
                            slot.client = Some(client);
                            None
                        }
                        Err(error) => {
                            slot.state = LanguageServerRuntimeState::Failed;
                            slot.error = Some(error.message.clone());
                            return Err(error);
                        }
                    }
                }
            } else {
                result.ok()
            }
        };
        if let Some(client) = stale_client {
            client.close().await;
        }
        Ok(())
    }

    pub(super) async fn shutdown(&self) {
        if self.inner.closed.swap(true, Ordering::AcqRel) {
            return;
        }
        let clients = {
            let mut projects = self.inner.projects.lock().await;
            projects
                .drain()
                .flat_map(|(_, project)| project.slots.into_values())
                .filter_map(|slot| slot.client)
                .collect::<Vec<_>>()
        };
        for client in clients {
            client.close().await;
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

    async fn retire_server(&self, project_id: &str, language_server_id: &str) {
        let client = self
            .inner
            .projects
            .lock()
            .await
            .get_mut(project_id)
            .and_then(|runtime| runtime.slots.remove(language_server_id))
            .and_then(|slot| slot.client);
        if let Some(client) = client {
            client.close().await;
        }
    }

    pub(super) async fn execute(
        &self,
        request: SemanticRequest<'_>,
        cancellation: CancellationToken,
    ) -> Result<Value, BusinessError> {
        self.activate_project(request.project_id, request.project_root)
            .await?;
        let server = self
            .inner
            .store
            .language_servers()?
            .into_iter()
            .find(|server| {
                server.enabled
                    && root_markers_match(request.project_root, &server.config.root_markers)
                    && server
                        .config
                        .language_ids
                        .iter()
                        .any(|value| value.eq_ignore_ascii_case(request.language_id))
            })
            .ok_or_else(|| {
                BusinessError::new(
                    "lsp_server_unavailable",
                    format!(
                        "No enabled language server is configured for `{}`",
                        request.language_id
                    ),
                )
            })?;
        let needs_start = {
            let projects = self.inner.projects.lock().await;
            projects
                .get(request.project_id)
                .and_then(|runtime| runtime.slots.get(&server.language_server_id))
                .is_none_or(|slot| {
                    slot.client.as_ref().is_none_or(Client::is_closed)
                        || slot.revision != server.revision
                })
        };
        if needs_start {
            self.start_server(request.project_id, server.clone())
                .await?;
        }
        let client = {
            let projects = self.inner.projects.lock().await;
            projects
                .get(request.project_id)
                .and_then(|runtime| runtime.slots.get(&server.language_server_id))
                .and_then(|slot| slot.client.clone())
                .ok_or_else(|| {
                    BusinessError::new(
                        "lsp_server_unavailable",
                        "Language server is not ready for this project",
                    )
                })?
        };
        let canonical = request
            .scope_root
            .join(request.relative_path)
            .canonicalize()
            .map_err(|_| BusinessError::new("path_unavailable", "path is unavailable"))?;
        if !canonical.starts_with(request.scope_root) {
            return Err(BusinessError::new(
                "scope_denied",
                "path is outside the authorized source root",
            ));
        }
        let uri = Url::from_file_path(&canonical)
            .map_err(|_| BusinessError::invalid("path could not be converted to a file URI"))?
            .to_string();
        client
            .sync_document(&uri, request.language_id, request.text)
            .await?;
        let capabilities = client.capabilities().await;
        let (line, character) = match (request.line, request.column) {
            (Some(line), Some(column)) => {
                position(request.text, line, column, capabilities.position_encoding)?
            }
            _ => (0, 0),
        };
        let raw = match request.operation {
            SemanticOperation::Diagnostics => client.diagnostics(&uri, cancellation).await?,
            SemanticOperation::Definition if capabilities.definition => {
                client
                    .definition(&uri, line, character, cancellation)
                    .await?
            }
            SemanticOperation::References if capabilities.references => {
                client
                    .references(&uri, line, character, cancellation)
                    .await?
            }
            SemanticOperation::Hover if capabilities.hover => {
                client.hover(&uri, line, character, cancellation).await?
            }
            SemanticOperation::Symbols if capabilities.document_symbols => {
                client.document_symbols(&uri, cancellation).await?
            }
            _ => {
                return Err(BusinessError::new(
                    "lsp_capability_unavailable",
                    "Language server does not advertise the requested capability",
                ))
            }
        };
        {
            let mut projects = self.inner.projects.lock().await;
            if let Some(slot) = projects
                .get_mut(request.project_id)
                .and_then(|runtime| runtime.slots.get_mut(&server.language_server_id))
            {
                slot.state = LanguageServerRuntimeState::Ready;
                slot.error = None;
            }
        }
        self.normalize_result(request.project_id, request.display_path, raw)
    }

    fn normalize_result(
        &self,
        project_id: &str,
        display_path: &str,
        mut value: Value,
    ) -> Result<Value, BusinessError> {
        let project = self
            .inner
            .store
            .project_by_id(project_id)?
            .ok_or_else(|| BusinessError::missing("project"))?;
        let dependencies = self.inner.store.project_dependencies(project_id)?;
        normalize_value(
            &mut value,
            Path::new(&project.canonical_root),
            &dependencies,
        );
        if let Some(items) = value.as_array_mut() {
            items.truncate(MAX_NORMALIZED_ITEMS);
        }
        Ok(json!({"path":display_path,"result":value}))
    }
}

fn root_markers_match(project_root: &Path, root_markers: &[String]) -> bool {
    root_markers.is_empty()
        || root_markers
            .iter()
            .any(|marker| project_root.join(marker).exists())
}

fn capability_count(capabilities: &suncode_lsp::ServerCapabilities) -> usize {
    [
        capabilities.pull_diagnostics,
        capabilities.definition,
        capabilities.references,
        capabilities.hover,
        capabilities.document_symbols,
    ]
    .into_iter()
    .filter(|value| *value)
    .count()
}

fn position(
    text: &str,
    line: u32,
    column: u32,
    encoding: PositionEncoding,
) -> Result<(u32, u32), BusinessError> {
    if line == 0 || column == 0 {
        return Err(BusinessError::invalid("line and column must be one-based"));
    }
    let line_index =
        usize::try_from(line - 1).map_err(|_| BusinessError::invalid("line invalid"))?;
    let source_line = text
        .split('\n')
        .nth(line_index)
        .ok_or_else(|| BusinessError::invalid("line is beyond the end of the file"))?
        .trim_end_matches('\r');
    let character_count =
        usize::try_from(column - 1).map_err(|_| BusinessError::invalid("column invalid"))?;
    let prefix = source_line
        .chars()
        .take(character_count)
        .collect::<String>();
    if prefix.chars().count() != character_count {
        return Err(BusinessError::invalid(
            "column is beyond the end of the line",
        ));
    }
    let character = match encoding {
        PositionEncoding::Utf8 => prefix.len(),
        PositionEncoding::Utf16 => prefix.encode_utf16().count(),
        PositionEncoding::Utf32 => prefix.chars().count(),
    };
    Ok((line - 1, u32::try_from(character).unwrap_or(u32::MAX)))
}

fn normalize_value(
    value: &mut Value,
    project_root: &Path,
    dependencies: &[suncode_data::ProjectDependencyRecord],
) {
    match value {
        Value::Array(values) => {
            values.truncate(MAX_NORMALIZED_ITEMS);
            for value in values {
                normalize_value(value, project_root, dependencies);
            }
        }
        Value::Object(object) => {
            normalize_uri_key(object, "uri", project_root, dependencies);
            normalize_uri_key(object, "targetUri", project_root, dependencies);
            for key in [
                "range",
                "selectionRange",
                "targetRange",
                "targetSelectionRange",
            ] {
                if let Some(range) = object.get_mut(key) {
                    normalize_range(range);
                }
            }
            for value in object.values_mut() {
                normalize_value(value, project_root, dependencies);
            }
        }
        _ => {}
    }
}

fn normalize_uri_key(
    object: &mut Map<String, Value>,
    key: &str,
    project_root: &Path,
    dependencies: &[suncode_data::ProjectDependencyRecord],
) {
    let Some(uri) = object
        .remove(key)
        .and_then(|value| value.as_str().map(str::to_string))
    else {
        return;
    };
    let path = Url::parse(&uri)
        .ok()
        .and_then(|uri| uri.to_file_path().ok());
    let normalized = path
        .as_deref()
        .and_then(|path| scoped_display_path(path, project_root, dependencies));
    match normalized {
        Some(path) => {
            object.insert("path".into(), json!(path));
        }
        None => {
            object.insert("external".into(), json!(true));
        }
    }
}

fn scoped_display_path(
    path: &Path,
    project_root: &Path,
    dependencies: &[suncode_data::ProjectDependencyRecord],
) -> Option<String> {
    if let Ok(relative) = path.strip_prefix(project_root) {
        return Some(relative.to_string_lossy().replace('\\', "/"));
    }
    dependencies.iter().find_map(|dependency| {
        path.strip_prefix(&dependency.canonical_root)
            .ok()
            .map(|relative| {
                format!(
                    "dependency:{}/{}",
                    dependency.dependency_id,
                    relative.to_string_lossy().replace('\\', "/")
                )
            })
    })
}

fn normalize_range(value: &mut Value) {
    let Some(object) = value.as_object_mut() else {
        return;
    };
    for endpoint in ["start", "end"] {
        let Some(position) = object.get_mut(endpoint).and_then(Value::as_object_mut) else {
            continue;
        };
        if let Some(line) = position.get_mut("line") {
            if let Some(value) = line.as_u64() {
                *line = json!(value.saturating_add(1));
            }
        }
        if let Some(character) = position.get_mut("character") {
            if let Some(value) = character.as_u64() {
                *character = json!(value.saturating_add(1));
            }
        }
    }
}

pub(super) fn is_lsp_tool(name: &str) -> bool {
    matches!(
        name,
        "lsp_diagnostics" | "lsp_definition" | "lsp_references" | "lsp_hover" | "lsp_symbols"
    )
}

pub(super) fn operation(name: &str) -> Option<SemanticOperation> {
    match name {
        "lsp_diagnostics" => Some(SemanticOperation::Diagnostics),
        "lsp_definition" => Some(SemanticOperation::Definition),
        "lsp_references" => Some(SemanticOperation::References),
        "lsp_hover" => Some(SemanticOperation::Hover),
        "lsp_symbols" => Some(SemanticOperation::Symbols),
        _ => None,
    }
}

pub(super) fn infer_language_id(path: &str) -> Option<&'static str> {
    let path = path.to_ascii_lowercase();
    let extension = path.rsplit('.').next().unwrap_or_default();
    match extension {
        "rs" => Some("rust"),
        "go" => Some("go"),
        "py" | "pyi" => Some("python"),
        "ts" => Some("typescript"),
        "tsx" => Some("typescriptreact"),
        "js" | "mjs" | "cjs" => Some("javascript"),
        "jsx" => Some("javascriptreact"),
        "cs" => Some("csharp"),
        "c" | "h" => Some("c"),
        "cc" | "cpp" | "cxx" | "hpp" => Some("cpp"),
        "java" => Some("java"),
        "kt" | "kts" => Some("kotlin"),
        "swift" => Some("swift"),
        "rb" => Some("ruby"),
        "php" => Some("php"),
        "lua" => Some("lua"),
        "zig" => Some("zig"),
        "ex" | "exs" => Some("elixir"),
        "vue" => Some("vue"),
        "svelte" => Some("svelte"),
        "json" => Some("json"),
        "yaml" | "yml" => Some("yaml"),
        "toml" => Some("toml"),
        _ => None,
    }
}

impl Agent {
    pub async fn activate_language_server_project(
        &self,
        project_id: &str,
        project_root: &Path,
    ) -> Result<(), BusinessError> {
        self.lsp.activate_project(project_id, project_root).await
    }

    pub async fn reconcile_language_server(
        &self,
        language_server_id: &str,
    ) -> Result<(), BusinessError> {
        self.lsp.reconcile_all(language_server_id).await
    }

    pub async fn retry_language_server(
        &self,
        project_id: &str,
        language_server_id: &str,
    ) -> Result<(), BusinessError> {
        self.lsp.retry(project_id, language_server_id).await
    }

    pub async fn language_server_runtime_status(
        &self,
        project_id: Option<&str>,
        server: &LanguageServerRecord,
    ) -> LanguageServerRuntimeStatus {
        self.lsp.status(project_id, server).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_one_based_unicode_positions() {
        assert_eq!(
            position("a😀b", 1, 3, PositionEncoding::Utf8).unwrap(),
            (0, 5)
        );
        assert_eq!(
            position("a😀b", 1, 3, PositionEncoding::Utf16).unwrap(),
            (0, 3)
        );
        assert_eq!(
            position("a😀b", 1, 3, PositionEncoding::Utf32).unwrap(),
            (0, 2)
        );
    }

    #[test]
    fn maps_common_extensions_to_language_ids() {
        assert_eq!(infer_language_id("src/lib.rs"), Some("rust"));
        assert_eq!(infer_language_id("src/App.tsx"), Some("typescriptreact"));
        assert_eq!(infer_language_id("README"), None);
    }

    #[test]
    fn root_markers_limit_server_selection() {
        let root =
            std::env::temp_dir().join(format!("suncode-lsp-root-markers-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("Cargo.toml"), "[package]\nname = \"sample\"\n").unwrap();

        assert!(root_markers_match(&root, &[]));
        assert!(root_markers_match(&root, &["Cargo.toml".into()]));
        assert!(!root_markers_match(&root, &["package.json".into()]));

        std::fs::remove_dir_all(root).unwrap();
    }
}
