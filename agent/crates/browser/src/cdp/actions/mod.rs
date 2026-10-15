//! Page actions, one module per capability, dispatched from `BrowserCommand`.

mod capture;
mod input;
mod navigation;

use crate::cdp::command::BrowserCommand;
use crate::cdp::error::{browser_error, with_timeout, CDP_EVALUATION_FAILED, CDP_PROTOCOL_ERROR};
use crate::cdp::launch::PROTOCOL_VERSION;
use crate::cdp::session::CdpSession;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use suncode_common::BusinessError;

impl CdpSession {
    pub(crate) async fn execute(&self, command: BrowserCommand) -> Result<Value, BusinessError> {
        match command {
            BrowserCommand::State => self.state().await,
            BrowserCommand::Probe => Ok(json!({
                "protocolVersion": PROTOCOL_VERSION,
                "transport": "cdp",
                "targetId": self.target_id
            })),
            BrowserCommand::Navigate { url } => self.navigate(&url).await,
            BrowserCommand::Snapshot => self.snapshot().await,
            BrowserCommand::Screenshot { full_page } => self.screenshot(full_page).await,
            BrowserCommand::Click { target } => self.click(&target).await,
            BrowserCommand::Fill { target, value } => self.fill(&target, &value).await,
            BrowserCommand::Press { key } => self.press(&key).await,
            BrowserCommand::ClosePage => self.close_page().await,
        }
    }

    /// The agent always holds control of its attached page.
    async fn state(&self) -> Result<Value, BusinessError> {
        Ok(json!({
            "started": true,
            "controlOwner": "agent",
            "pages": [{"id": self.target_id, "url": self.current_url().await?}]
        }))
    }

    /// Acknowledgement returned by actions that produce no data.
    fn page_ack(&self) -> Value {
        json!({"pageId": self.target_id})
    }

    async fn evaluate<T: DeserializeOwned>(&self, expression: String) -> Result<T, BusinessError> {
        let result = with_timeout(
            self.request_timeout,
            self.page.evaluate(expression),
            CDP_EVALUATION_FAILED,
            "Browser DOM operation failed",
        )
        .await?;
        result.into_value().map_err(|error| {
            browser_error(
                CDP_PROTOCOL_ERROR,
                format!("Browser evaluation result is invalid: {error}"),
            )
        })
    }
}
