//! Execution of `browser_*` tool calls against a project session.

use super::BrowserRuntime;
use crate::action::BrowserAction;
use crate::tools::parse_call;
use serde_json::Value;
use suncode_common::tool::{
    BoxFuture, Cancellation, Risk, ToolDefinition, ToolInvocation, ToolProvider,
};
use suncode_common::{logging, BusinessError};

impl BrowserRuntime {
    /// Runs one `browser_*` tool call, starting the project browser on demand.
    /// Cancellation or a failed request drops the session so the next call reconnects.
    pub async fn call(
        &self,
        project_id: &str,
        name: &str,
        arguments: Value,
        cancellation: &dyn Cancellation,
    ) -> Result<Value, BusinessError> {
        logging::debug(
            "browser.call",
            format!("begin project={} tool={}", project_id, name),
        );
        self.require_enabled()?;
        let action = parse_call(name, &arguments)?;
        if cancellation.is_cancelled() {
            return Err(BusinessError::new(
                "cancelled",
                "Browser operation was cancelled",
            ));
        }
        self.start_project(project_id).await?;
        let session = self.session(project_id).await?;
        let is_screenshot = matches!(action, BrowserAction::Screenshot { .. });
        let request = session.execute(action);
        tokio::pin!(request);
        let mut result = tokio::select! {
            _ = cancellation.cancelled() => {
                session.close().await;
                self.inner.projects.lock().await.remove(project_id);
                return Err(BusinessError::new("cancelled", "Browser operation was cancelled"));
            }
            result = &mut request => match result {
                Ok(result) => result,
                Err(error) => {
                    logging::write_business_error(
                        "browser.call",
                        "request",
                        &error,
                        format!("project={} tool={}", project_id, name),
                    );
                    session.close().await;
                    self.fail_slot(project_id, &error).await;
                    return Err(error);
                }
            }
        };
        logging::debug(
            "browser.call",
            format!("complete project={} tool={}", project_id, name),
        );
        if is_screenshot {
            result = self.retain_screenshot(result)?;
        }
        Ok(result)
    }
}

impl ToolProvider for BrowserRuntime {
    fn owns(&self, name: &str) -> bool {
        crate::tools::is_browser_tool(name)
    }

    fn catalog<'a>(&'a self, _project_id: &'a str) -> BoxFuture<'a, Vec<ToolDefinition>> {
        Box::pin(BrowserRuntime::catalog(self))
    }

    fn validate(&self, name: &str, arguments: &Value) -> Result<(), BusinessError> {
        crate::tools::validate_browser_arguments(name, arguments)
    }

    fn risk(&self, _name: &str) -> Option<Risk> {
        Some(Risk::BrowserAccess)
    }

    fn execute<'a>(
        &'a self,
        invocation: ToolInvocation<'a>,
        cancellation: &'a dyn Cancellation,
    ) -> BoxFuture<'a, Result<Value, BusinessError>> {
        Box::pin(self.call(
            invocation.project_id,
            invocation.name,
            invocation.arguments,
            cancellation,
        ))
    }
}
