//! Tool providers registered with the agent, and the core's implementations
//! of the provider context: path scopes, live output, and cancellation.
//!
//! Tool sources implement `suncode_common::tool::ToolProvider` in their own
//! crates. The core routes calls to them by name and keeps policy, approval,
//! auditing, and events. Turn-level tools (`question`, `todowrite`, `skill`,
//! `delegate_agent`) stay in the core because they act on the turn itself.

use super::{Agent, Continuation, EventPayload, ToolCall, ToolOutputPayload};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use suncode_common::tool::{
    BoxFuture, Cancellation, CatalogContext, Document, DocumentReader, PathScopes, ScopedPath,
    ToolContext, ToolDefinition, ToolInvocation, ToolOutput, ToolProvider,
};
use suncode_common::BusinessError;
use suncode_data::Store;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Default)]
pub(super) struct ToolProviders(Vec<Arc<dyn ToolProvider>>);

impl ToolProviders {
    pub(super) fn new(providers: Vec<Arc<dyn ToolProvider>>) -> Self {
        Self(providers)
    }

    /// The provider that handles `name`, if any.
    pub(super) fn find(&self, name: &str) -> Option<&dyn ToolProvider> {
        self.0
            .iter()
            .find(|provider| provider.owns(name))
            .map(Arc::as_ref)
    }

    /// Tools advertised by every provider, in registration order.
    pub(super) async fn catalog(&self, context: &CatalogContext<'_>) -> Vec<ToolDefinition> {
        let mut definitions = Vec::new();
        for provider in &self.0 {
            definitions.extend(provider.catalog(context).await);
        }
        definitions
    }
}

/// Adapts the turn's Tokio token to the runtime-neutral provider interface.
pub(super) struct TokenCancellation(pub(super) CancellationToken);

impl Cancellation for TokenCancellation {
    fn is_cancelled(&self) -> bool {
        self.0.is_cancelled()
    }

    fn cancelled(&self) -> BoxFuture<'_, ()> {
        Box::pin(self.0.cancelled())
    }
}

const DEPENDENCY_PREFIX: &str = "dependency:";

/// Splits `dependency:<id>/<relative>` into its parts; the relative part
/// defaults to `.`. `None` for paths without the prefix or without an id.
pub(super) fn dependency_path(path: &str) -> Option<(&str, &str)> {
    let value = path.strip_prefix(DEPENDENCY_PREFIX)?;
    let (dependency_id, relative_path) = value.split_once('/').unwrap_or((value, "."));
    if dependency_id.is_empty() {
        return None;
    }
    Some((dependency_id, relative_path))
}

/// Resolves `dependency:` aliases against the project's registered
/// dependencies. Other paths stay project-relative.
pub(super) struct StorePathScopes<'a> {
    pub(super) store: &'a Store,
    pub(super) project_id: &'a str,
    /// Canonical project root, for mapping absolute paths back.
    pub(super) project_root: &'a std::path::Path,
}

impl PathScopes for StorePathScopes<'_> {
    fn resolve(&self, path: &str) -> Result<ScopedPath, BusinessError> {
        if !path.starts_with(DEPENDENCY_PREFIX) {
            return Ok(ScopedPath::Project {
                relative: path.to_string(),
            });
        }
        let (dependency_id, relative) = dependency_path(path).ok_or_else(|| {
            BusinessError::new(
                "invalid_arguments",
                "dependency path must include a dependency ID",
            )
        })?;
        let dependency = self
            .store
            .project_dependency_by_id(self.project_id, dependency_id)?
            .ok_or_else(|| BusinessError::new("dependency_not_found", "dependency not found"))?;
        Ok(ScopedPath::Dependency {
            dependency_id: dependency_id.to_string(),
            root: dependency.canonical_root.into(),
            relative: relative.to_string(),
        })
    }

    fn display_path(&self, path: &std::path::Path) -> Option<String> {
        if let Ok(relative) = path.strip_prefix(self.project_root) {
            return Some(relative.to_string_lossy().replace('\\', "/"));
        }
        let dependencies = self.store.project_dependencies(self.project_id).ok()?;
        dependencies.iter().find_map(|dependency| {
            path.strip_prefix(&dependency.canonical_root)
                .ok()
                .map(|relative| {
                    format!(
                        "{DEPENDENCY_PREFIX}{}/{}",
                        dependency.dependency_id,
                        relative.to_string_lossy().replace('\\', "/")
                    )
                })
        })
    }
}

/// Reads documents through the audited `tool/read` operation, so providers
/// get file content without their own file system access.
pub(super) struct OperationsDocumentReader<'a> {
    pub(super) agent: &'a Agent,
    pub(super) paths: &'a StorePathScopes<'a>,
    pub(super) token: CancellationToken,
}

impl DocumentReader for OperationsDocumentReader<'_> {
    fn read_text<'a>(
        &'a self,
        path: &'a str,
        max_bytes: u64,
    ) -> BoxFuture<'a, Result<Document, BusinessError>> {
        Box::pin(async move {
            let (root, relative) = match self.paths.resolve(path)? {
                ScopedPath::Project { relative } => {
                    (self.paths.project_root.to_path_buf(), relative)
                }
                ScopedPath::Dependency { root, relative, .. } => (root, relative),
            };
            let read = self
                .agent
                .operation_in_project(
                    &root.to_string_lossy(),
                    "tool/read",
                    serde_json::json!({"path": relative, "offset": 1, "max_bytes": max_bytes}),
                    self.token.clone(),
                )
                .await?;
            if read.get("truncated").and_then(Value::as_bool) == Some(true) {
                return Err(BusinessError::new(
                    "file_too_large",
                    format!("document exceeds {max_bytes} bytes"),
                ));
            }
            let bytes = read
                .get("data_base64")
                .and_then(Value::as_str)
                .ok_or_else(|| BusinessError::new("read_failed", "file content is unavailable"))
                .and_then(|encoded| {
                    STANDARD.decode(encoded).map_err(|_| {
                        BusinessError::new("read_failed", "file content could not be decoded")
                    })
                })?;
            let text = String::from_utf8(bytes).map_err(|_| {
                BusinessError::new("encoding_unsupported", "document is not UTF-8 text")
            })?;
            let document = root
                .join(&relative)
                .canonicalize()
                .map_err(|_| BusinessError::new("path_unavailable", "path is unavailable"))?;
            Ok(Document {
                path: document,
                display_path: path.to_string(),
                text,
            })
        })
    }
}

/// Publishes live tool output as session events.
pub(super) struct EventOutput {
    pub(super) agent: Agent,
    pub(super) session_id: String,
    pub(super) turn_id: String,
    pub(super) call_id: Option<String>,
    pub(super) tool_call_id: String,
}

impl ToolOutput for EventOutput {
    fn chunk(&self, stream: &str, bytes: &[u8]) {
        self.agent.emit_live(
            &self.session_id,
            EventPayload::ToolOutput(ToolOutputPayload {
                turn_id: self.turn_id.clone(),
                call_id: self.call_id.clone(),
                tool_call_id: self.tool_call_id.clone(),
                stream: stream.to_owned(),
                chunk_base64: STANDARD.encode(bytes),
            }),
        );
    }
}

impl Agent {
    /// Runs `call` through `provider` with this turn's context.
    pub(super) async fn execute_provider(
        &self,
        provider: &dyn ToolProvider,
        context: &Continuation,
        call: &ToolCall,
        token: CancellationToken,
        approval_pin: Option<Value>,
    ) -> Result<suncode_common::tool::ToolResult, BusinessError> {
        let call_key = format!("{}:{}", context.turn_id, call.call_id);
        let project_root = std::path::Path::new(&context.project_root);
        let paths = StorePathScopes {
            store: &self.store,
            project_id: &context.project_id,
            project_root,
        };
        let documents = OperationsDocumentReader {
            agent: self,
            paths: &paths,
            token: token.clone(),
        };
        let output: Arc<dyn ToolOutput> = Arc::new(EventOutput {
            agent: self.clone(),
            session_id: context.session_id.clone(),
            turn_id: context.turn_id.clone(),
            call_id: context.active_call_id.clone(),
            tool_call_id: call.call_id.clone(),
        });
        let cancellation = TokenCancellation(token);
        provider
            .execute(
                ToolInvocation {
                    name: &super::tools::tool_name(call),
                    arguments: call.arguments.clone(),
                    approval_pin,
                },
                ToolContext {
                    project_id: &context.project_id,
                    project_root,
                    call_key: &call_key,
                    timeout_ceiling: context.bash_timeout_ms.map(Duration::from_millis),
                    paths: &paths,
                    documents: &documents,
                    output,
                    cancellation: &cancellation,
                },
            )
            .await
    }
}
