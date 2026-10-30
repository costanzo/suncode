//! The built-in tools as a [`ToolProvider`]: `read`, `glob`, `grep`, `write`,
//! `edit`, `bash`, and `webfetch`, and the read-only document tools,
//! backed by [`Operations`].
//!
//! The core's turn-level tools (`question`, `todowrite`, `skill`,
//! `delegate_agent`) are not here; they act on the turn itself.

mod shell;
mod translate;

#[cfg(test)]
mod tests;

use crate::{definitions, Operations};
use serde_json::{json, Value};
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use suncode_common::tool::{
    BoxFuture, CatalogContext, Risk, ScopedPath, ToolContext, ToolDefinition, ToolInvocation,
    ToolProvider, ToolResult,
};
use suncode_common::BusinessError;

/// Default bash timeout when the model does not set one.
const DEFAULT_BASH_TIMEOUT_MS: u64 = 120_000;

pub struct BuiltinTools {
    operations: Arc<Operations>,
}

impl BuiltinTools {
    pub fn new(operations: Arc<Operations>) -> Self {
        Self { operations }
    }
}

fn method(name: &str) -> Option<&'static str> {
    match name {
        "read" => Some("tool/read"),
        "glob" => Some("tool/glob"),
        "grep" => Some("tool/grep"),
        "document_inspect" => Some("tool/document_inspect"),
        "document_read" => Some("tool/document_read"),
        "document_search" => Some("tool/document_search"),
        "spreadsheet_read" => Some("tool/spreadsheet_read"),
        "write" => Some("tool/write"),
        "edit" => Some("tool/edit"),
        "bash" => Some("tool/bash"),
        "webfetch" => Some("tool/webfetch"),
        _ => None,
    }
}

fn is_document_tool(name: &str) -> bool {
    matches!(
        name,
        "document_inspect" | "document_read" | "document_search" | "spreadsheet_read"
    )
}

/// Dependencies are read-only, so only read-only file tools may target them.
fn reads_dependencies(name: &str) -> bool {
    matches!(name, "read" | "glob" | "grep") || is_document_tool(name)
}

impl ToolProvider for BuiltinTools {
    fn owns(&self, name: &str) -> bool {
        method(name).is_some()
    }

    fn catalog<'a>(
        &'a self,
        _context: &'a CatalogContext<'a>,
    ) -> BoxFuture<'a, Vec<ToolDefinition>> {
        Box::pin(async {
            definitions::all()
                .into_iter()
                .filter(|definition| method(&definition.name).is_some())
                .collect()
        })
    }

    fn validate(&self, name: &str, arguments: &Value) -> Result<(), BusinessError> {
        match name {
            "bash" => shell::validate_bash_arguments(arguments),
            "webfetch" => translate::validate_webfetch_arguments(arguments),
            _ => Ok(()),
        }
    }

    fn risk(&self, name: &str) -> Option<Risk> {
        match name {
            "read" | "glob" | "grep" => Some(Risk::ReadOnly),
            name if is_document_tool(name) => Some(Risk::ReadOnly),
            "webfetch" => Some(Risk::NetworkAccess),
            "write" | "edit" => Some(Risk::ProjectWrite),
            "bash" => Some(Risk::ProcessExecution),
            _ => None,
        }
    }

    fn allows_dependency_paths(&self, name: &str) -> bool {
        reads_dependencies(name)
    }

    fn execute<'a>(
        &'a self,
        invocation: ToolInvocation<'a>,
        context: ToolContext<'a>,
    ) -> BoxFuture<'a, Result<ToolResult, BusinessError>> {
        Box::pin(async move {
            let method = method(invocation.name)
                .ok_or_else(|| BusinessError::new("authorization_denied", "Unknown tool"))?;
            let (root, params, dependency_id) = prepare(&invocation, &context)?;
            let raw = self.run_blocking(root, method, params, &context).await?;
            Ok(translate::normalize_result(invocation.name, raw, dependency_id.as_deref()).into())
        })
    }
}

/// Resolves the call's root, translates model arguments, and applies the
/// host's timeout ceiling and idempotency key.
fn prepare(
    invocation: &ToolInvocation<'_>,
    context: &ToolContext<'_>,
) -> Result<(std::path::PathBuf, Value, Option<String>), BusinessError> {
    let mut arguments = invocation.arguments.clone();
    let (root, dependency_id) = match arguments.get("path").and_then(Value::as_str) {
        None => (context.project_root.to_path_buf(), None),
        Some(path) => match context.paths.resolve(path)? {
            ScopedPath::Project { relative } => {
                // Authorized absolute paths arrive here already made relative
                arguments["path"] = json!(relative);
                (context.project_root.to_path_buf(), None)
            }
            ScopedPath::Dependency {
                dependency_id,
                root,
                relative,
            } => {
                if !reads_dependencies(invocation.name) {
                    return Err(BusinessError::new(
                        "scope_denied",
                        "dependencies are read-only",
                    ));
                }
                arguments["path"] = json!(relative);
                (root, Some(dependency_id))
            }
        },
    };
    let mut params = translate::translate_arguments(invocation.name, &arguments, Some(&root))?;
    if invocation.name == "bash" {
        if let Some(ceiling) = context.timeout_ceiling {
            let ceiling = u64::try_from(ceiling.as_millis()).unwrap_or(u64::MAX);
            let requested = params
                .get("timeout_ms")
                .and_then(Value::as_u64)
                .unwrap_or(DEFAULT_BASH_TIMEOUT_MS);
            params["timeout_ms"] = json!(requested.min(ceiling));
        }
    }
    params["idempotency_key"] = json!(context.call_key);
    Ok((root, params, dependency_id))
}

impl BuiltinTools {
    /// Runs one blocking operation on its own thread so the caller's async
    /// runtime is not stalled, forwarding cancellation to the operations layer.
    async fn run_blocking(
        &self,
        root: std::path::PathBuf,
        method: &'static str,
        params: Value,
        context: &ToolContext<'_>,
    ) -> Result<Value, BusinessError> {
        let operations = self.operations.clone();
        let cancelled = Arc::new(AtomicBool::new(false));
        let output = context.output.clone();
        let receiver = {
            let cancelled = cancelled.clone();
            suncode_common::blocking::run_blocking("suncode-tool", move || {
                let callback: crate::ProcessOutputCallback =
                    Arc::new(move |stream: &str, bytes: &[u8]| output.chunk(stream, bytes));
                operations.execute_in_project_with_cancellation_and_output(
                    &root,
                    method,
                    params,
                    Some(&cancelled),
                    Some(callback),
                )
            })
        };
        let mut receiver = std::pin::pin!(receiver);
        let mut cancellation = context.cancellation.cancelled();
        // Wait for the result; on cancellation, signal the operation and keep
        // waiting so it can terminate its process tree and report honestly.
        let first = std::future::poll_fn(|cx| {
            if let std::task::Poll::Ready(result) = receiver.as_mut().poll(cx) {
                return std::task::Poll::Ready(Some(result));
            }
            cancellation.as_mut().poll(cx).map(|()| None)
        })
        .await;
        let result = match first {
            Some(result) => result,
            None => {
                cancelled.store(true, Ordering::Relaxed);
                receiver.await
            }
        };
        result.unwrap_or_else(|| {
            Err(BusinessError::new(
                "agent_unavailable",
                "operation task failed",
            ))
        })
    }
}
