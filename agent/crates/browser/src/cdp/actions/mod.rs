//! Page actions, one module per capability, dispatched from [`BrowserAction`].

mod capture;
mod input;
mod navigation;

use crate::action::BrowserAction;
use crate::cdp::error::{browser_error, with_timeout, CDP_EVALUATION_FAILED, CDP_PROTOCOL_ERROR};
use crate::cdp::session::CdpSession;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use suncode_common::BusinessError;

/// The attached page, as reported to the runtime and the model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PageState {
    pub(crate) page_id: String,
    pub(crate) url: String,
}

impl PageState {
    pub(crate) fn to_json(&self) -> Value {
        json!({"pages": [{"id": self.page_id, "url": self.url}]})
    }
}

impl CdpSession {
    pub(crate) async fn execute(&self, action: BrowserAction) -> Result<Value, BusinessError> {
        match action {
            BrowserAction::State => Ok(self.state().await?.to_json()),
            BrowserAction::Navigate { url } => self.navigate(&url).await,
            BrowserAction::Snapshot => self.snapshot().await,
            BrowserAction::Screenshot { full_page } => self.screenshot(full_page).await,
            BrowserAction::Click { target } => self.click(&target).await,
            BrowserAction::Fill { target, value } => self.fill(&target, &value).await,
            BrowserAction::Press { key } => self.press(&key).await,
            BrowserAction::ClosePage => self.close_page().await,
        }
    }

    pub(crate) async fn state(&self) -> Result<PageState, BusinessError> {
        Ok(PageState {
            page_id: self.target_id.clone(),
            url: self.current_url().await?,
        })
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
