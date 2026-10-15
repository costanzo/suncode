use crate::cdp::error::{with_timeout, CDP_COMMAND_FAILED, CDP_PROTOCOL_ERROR};
use crate::cdp::session::CdpSession;
use serde_json::{json, Value};
use suncode_common::BusinessError;

impl CdpSession {
    pub(super) async fn navigate(&self, url: &str) -> Result<Value, BusinessError> {
        with_timeout(
            self.request_timeout,
            self.page.goto(url),
            CDP_COMMAND_FAILED,
            "Browser navigation failed",
        )
        .await?;
        Ok(json!({"pageId": self.target_id, "url": self.current_url().await?}))
    }

    pub(super) async fn close_page(&self) -> Result<Value, BusinessError> {
        with_timeout(
            self.request_timeout,
            self.page.clone().close(),
            CDP_COMMAND_FAILED,
            "Browser page close failed",
        )
        .await?;
        Ok(self.page_ack())
    }

    pub(super) async fn current_url(&self) -> Result<String, BusinessError> {
        let url = with_timeout(
            self.request_timeout,
            self.page.url(),
            CDP_PROTOCOL_ERROR,
            "Browser URL could not be read",
        )
        .await?;
        Ok(url.unwrap_or_default())
    }
}
