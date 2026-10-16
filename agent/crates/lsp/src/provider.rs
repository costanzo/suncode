//! [`LspRuntime`] as a tool provider for the five read-only `lsp_*` tools.

use crate::runtime::{infer_language_id, is_lsp_tool, operation, LspRuntime, SemanticRequest};
use serde_json::Value;
use suncode_common::tool::{
    BoxFuture, Risk, ScopedPath, ToolContext, ToolDefinition, ToolInvocation, ToolProvider,
};
use suncode_common::BusinessError;
use tokio_util::sync::CancellationToken;

/// Largest document synchronized to a language server.
const MAX_DOCUMENT_BYTES: u64 = 1024 * 1024;
const MAX_PATH_CHARS: usize = 4_096;
const MAX_LANGUAGE_ID_CHARS: usize = 64;

impl ToolProvider for LspRuntime {
    fn owns(&self, name: &str) -> bool {
        is_lsp_tool(name)
    }

    /// The `lsp_*` definitions are part of the static built-in set.
    fn catalog<'a>(&'a self, _project_id: &'a str) -> BoxFuture<'a, Vec<ToolDefinition>> {
        Box::pin(async { Vec::new() })
    }

    fn validate(&self, name: &str, arguments: &Value) -> Result<(), BusinessError> {
        validate_arguments(name, arguments)
    }

    fn risk(&self, _name: &str) -> Option<Risk> {
        Some(Risk::ReadOnly)
    }

    fn allows_dependency_paths(&self, _name: &str) -> bool {
        true
    }

    /// Calls share one server connection and sync a document first.
    fn concurrent(&self, _name: &str) -> bool {
        false
    }

    fn execute<'a>(
        &'a self,
        invocation: ToolInvocation<'a>,
        context: ToolContext<'a>,
    ) -> BoxFuture<'a, Result<Value, BusinessError>> {
        Box::pin(async move {
            let arguments = &invocation.arguments;
            let display_path = arguments
                .get("path")
                .and_then(Value::as_str)
                .ok_or_else(|| BusinessError::invalid("path is required"))?;
            let scope_root = match context.paths.resolve(display_path)? {
                ScopedPath::Project { .. } => context.project_root.to_path_buf(),
                ScopedPath::Dependency { root, .. } => root,
            };
            let document = context
                .documents
                .read_text(display_path, MAX_DOCUMENT_BYTES)
                .await
                .map_err(|error| {
                    if error.code == "file_too_large" {
                        BusinessError::new(
                            "lsp_document_too_large",
                            "Language server document exceeds the 1 MiB semantic-tool bound",
                        )
                    } else {
                        error
                    }
                })?;
            let language_id = arguments
                .get("languageId")
                .and_then(Value::as_str)
                .or_else(|| infer_language_id(display_path))
                .ok_or_else(|| {
                    BusinessError::new(
                        "lsp_language_unknown",
                        "Language could not be inferred; provide languageId",
                    )
                })?;
            let position = |key: &str| {
                arguments
                    .get(key)
                    .and_then(Value::as_u64)
                    .map(|value| u32::try_from(value).unwrap_or(u32::MAX))
            };
            let operation = operation(invocation.name)
                .ok_or_else(|| BusinessError::new("authorization_denied", "Unknown LSP tool"))?;
            // The client API takes a Tokio token; bridge the call's
            // runtime-neutral cancellation into one.
            let token = CancellationToken::new();
            let request = self.execute(
                SemanticRequest {
                    project_id: context.project_id,
                    project_root: context.project_root,
                    scope_root: &scope_root,
                    display_path,
                    document: &document.path,
                    language_id,
                    text: &document.text,
                    line: position("line"),
                    column: position("column"),
                    operation,
                    paths: context.paths,
                },
                token.clone(),
            );
            tokio::pin!(request);
            tokio::select! {
                result = &mut request => result,
                () = context.cancellation.cancelled() => {
                    token.cancel();
                    request.await
                }
            }
        })
    }
}

fn validate_arguments(name: &str, value: &Value) -> Result<(), BusinessError> {
    let path = value
        .get("path")
        .and_then(Value::as_str)
        .filter(|path| !path.trim().is_empty() && path.chars().count() <= MAX_PATH_CHARS)
        .ok_or_else(|| {
            BusinessError::invalid("path is required and must be at most 4096 characters")
        })?;
    if path.contains('\0') {
        return Err(BusinessError::invalid("path contains an invalid character"));
    }
    if let Some(language_id) = value.get("languageId").and_then(Value::as_str) {
        if language_id.is_empty()
            || language_id.chars().count() > MAX_LANGUAGE_ID_CHARS
            || !language_id
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "_.+-".contains(character))
        {
            return Err(BusinessError::invalid("languageId is invalid"));
        }
    }
    if matches!(name, "lsp_definition" | "lsp_references" | "lsp_hover") {
        for key in ["line", "column"] {
            if value
                .get(key)
                .and_then(Value::as_u64)
                .is_none_or(|value| value == 0)
            {
                return Err(BusinessError::invalid(format!(
                    "{key} must be a one-based positive integer"
                )));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{ServerSource, ServerSpec};
    use serde_json::json;
    use std::path::Path;
    use std::sync::Arc;
    use suncode_common::tool::{Cancellation, Document, DocumentReader, PathScopes, ToolOutput};

    struct NoServers;
    impl ServerSource for NoServers {
        fn servers(&self) -> Result<Vec<ServerSpec>, BusinessError> {
            Ok(Vec::new())
        }
        fn server(&self, _: &str) -> Result<Option<ServerSpec>, BusinessError> {
            Ok(None)
        }
    }

    struct Paths;
    impl PathScopes for Paths {
        fn resolve(&self, path: &str) -> Result<ScopedPath, BusinessError> {
            Ok(ScopedPath::Project {
                relative: path.into(),
            })
        }
        fn display_path(&self, path: &Path) -> Option<String> {
            path.strip_prefix("/project")
                .ok()
                .map(|relative| relative.to_string_lossy().into_owned())
        }
    }

    struct TooLarge;
    impl DocumentReader for TooLarge {
        fn read_text<'a>(
            &'a self,
            _path: &'a str,
            max_bytes: u64,
        ) -> BoxFuture<'a, Result<Document, BusinessError>> {
            assert_eq!(max_bytes, MAX_DOCUMENT_BYTES);
            Box::pin(async { Err(BusinessError::new("file_too_large", "too large")) })
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

    #[tokio::test]
    async fn documents_are_read_through_the_context_with_the_semantic_bound() {
        let runtime = LspRuntime::new(Arc::new(NoServers));
        let error = ToolProvider::execute(
            &runtime,
            ToolInvocation {
                name: "lsp_symbols",
                arguments: json!({"path":"src/lib.rs"}),
                approval_pin: None,
            },
            ToolContext {
                project_id: "project",
                project_root: Path::new("/project"),
                call_key: "turn:call",
                timeout_ceiling: None,
                paths: &Paths,
                documents: &TooLarge,
                output: Arc::new(Sink),
                cancellation: &Never,
            },
        )
        .await
        .unwrap_err();
        assert_eq!(error.code, "lsp_document_too_large");
    }

    #[test]
    fn provider_is_read_only_dependency_capable_and_sequential() {
        let runtime = LspRuntime::new(Arc::new(NoServers));
        assert!(runtime.owns("lsp_hover") && !runtime.owns("read"));
        assert_eq!(runtime.risk("lsp_hover"), Some(Risk::ReadOnly));
        assert!(runtime.allows_dependency_paths("lsp_definition"));
        assert!(!runtime.concurrent("lsp_symbols"));
        assert!(runtime
            .validate("lsp_hover", &json!({"path":"a.rs","line":1,"column":1}))
            .is_ok());
        assert!(runtime
            .validate("lsp_hover", &json!({"path":"a.rs","line":0,"column":1}))
            .is_err());
        assert!(runtime
            .validate("lsp_symbols", &json!({"path":"a.rs","languageId":"bad id"}))
            .is_err());
    }
}
