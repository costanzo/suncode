//! Execution of `browser_*` tool calls against a project session.

use super::BrowserRuntime;
use crate::tools::{session_method, session_params};
use serde_json::Value;
use suncode_common::{logging, BusinessError};
use tokio_util::sync::CancellationToken;

impl BrowserRuntime {
    /// Runs one `browser_*` tool call, starting the project browser on demand.
    /// Cancellation or a failed request drops the session so the next call reconnects.
    pub async fn call(
        &self,
        project_id: &str,
        name: &str,
        arguments: Value,
        cancellation: CancellationToken,
    ) -> Result<Value, BusinessError> {
        logging::debug(
            "browser.call",
            format!("begin project={} tool={}", project_id, name),
        );
        self.require_enabled()?;
        if cancellation.is_cancelled() {
            return Err(BusinessError::new(
                "cancelled",
                "Browser operation was cancelled",
            ));
        }
        self.start_project(project_id).await?;
        let session = self.session(project_id).await?;
        let method = session_method(name)?;
        let params = session_params(name, &arguments)?;
        let request = session.request::<Value>(method, params);
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
        if let Some(pages) = result.get("pages").and_then(Value::as_array) {
            let mut projects = self.inner.projects.lock().await;
            if let Some(slot) = projects.get_mut(project_id) {
                slot.active_page_count = pages.len();
            }
        }
        logging::debug(
            "browser.call",
            format!("complete project={} tool={}", project_id, name),
        );
        if name == "browser_screenshot" {
            result = self.retain_screenshot(result)?;
        }
        Ok(result)
    }
}
