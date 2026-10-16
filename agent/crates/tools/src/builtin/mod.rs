//! The built-in tools as a [`ToolProvider`]: `read`, `glob`, `grep`, `write`,
//! `edit`, `bash`, and `webfetch`, backed by [`Operations`].
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
    BoxFuture, Risk, ScopedPath, ToolContext, ToolDefinition, ToolInvocation, ToolProvider,
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
        "write" => Some("tool/write"),
        "edit" => Some("tool/edit"),
        "bash" => Some("tool/bash"),
        "webfetch" => Some("tool/webfetch"),
        _ => None,
    }
}

impl ToolProvider for BuiltinTools {
    fn owns(&self, name: &str) -> bool {
        method(name).is_some()
    }

    fn catalog<'a>(&'a self, _project_id: &'a str) -> BoxFuture<'a, Vec<ToolDefinition>> {
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
            "webfetch" => Some(Risk::NetworkAccess),
            "write" | "edit" => Some(Risk::ProjectWrite),
            "bash" => Some(Risk::ProcessExecution),
            _ => None,
        }
    }

    fn allows_dependency_paths(&self, name: &str) -> bool {
        matches!(name, "read" | "glob" | "grep")
    }

    fn execute<'a>(
        &'a self,
        invocation: ToolInvocation<'a>,
        context: ToolContext<'a>,
    ) -> BoxFuture<'a, Result<Value, BusinessError>> {
        Box::pin(async move {
            let method = method(invocation.name)
                .ok_or_else(|| BusinessError::new("authorization_denied", "Unknown tool"))?;
            let (root, params, dependency_id) = prepare(&invocation, &context)?;
            let raw = self.run_blocking(root, method, params, &context).await?;
            Ok(translate::normalize_result(
                invocation.name,
                raw,
                dependency_id.as_deref(),
            ))
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
            ScopedPath::Project { .. } => (context.project_root.to_path_buf(), None),
            ScopedPath::Dependency {
                dependency_id,
                root,
                relative,
            } => {
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
        let (sender, receiver) = oneshot::channel();
        {
            let cancelled = cancelled.clone();
            std::thread::Builder::new()
                .name("suncode-tool".into())
                .spawn(move || {
                    let callback: crate::ProcessOutputCallback =
                        Arc::new(move |stream: &str, bytes: &[u8]| output.chunk(stream, bytes));
                    let result = operations.execute_in_project_with_cancellation_and_output(
                        &root,
                        method,
                        params,
                        Some(&cancelled),
                        Some(callback),
                    );
                    sender.send(result);
                })
                .map_err(|_| BusinessError::new("agent_unavailable", "operation task failed"))?;
        }
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

/// A minimal std-only one-shot channel. `None` means the sender was dropped,
/// for example because the operation thread panicked.
mod oneshot {
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};
    use std::task::{Context, Poll, Waker};

    struct State<T> {
        value: Option<T>,
        waker: Option<Waker>,
        closed: bool,
    }

    pub(super) struct Sender<T>(Arc<Mutex<State<T>>>);
    pub(super) struct Receiver<T>(Arc<Mutex<State<T>>>);

    pub(super) fn channel<T>() -> (Sender<T>, Receiver<T>) {
        let state = Arc::new(Mutex::new(State {
            value: None,
            waker: None,
            closed: false,
        }));
        (Sender(state.clone()), Receiver(state))
    }

    impl<T> Sender<T> {
        pub(super) fn send(self, value: T) {
            if let Ok(mut state) = self.0.lock() {
                state.value = Some(value);
            }
        }
    }

    impl<T> Drop for Sender<T> {
        fn drop(&mut self) {
            let waker = match self.0.lock() {
                Ok(mut state) => {
                    state.closed = true;
                    state.waker.take()
                }
                Err(_) => None,
            };
            if let Some(waker) = waker {
                waker.wake();
            }
        }
    }

    impl<T> Future for Receiver<T> {
        type Output = Option<T>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
            let Ok(mut state) = self.0.lock() else {
                return Poll::Ready(None);
            };
            if let Some(value) = state.value.take() {
                return Poll::Ready(Some(value));
            }
            if state.closed {
                return Poll::Ready(None);
            }
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
