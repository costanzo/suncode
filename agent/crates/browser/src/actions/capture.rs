use crate::error::{with_timeout, CDP_COMMAND_FAILED};
use crate::session::CdpSession;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chromiumoxide::page::ScreenshotParams;
use serde_json::{json, Value};
use suncode_common::BusinessError;

impl CdpSession {
    pub(super) async fn snapshot(&self) -> Result<Value, BusinessError> {
        let text: String = self
            .evaluate("document.body ? document.body.innerText : ''".into())
            .await?;
        Ok(json!({"pageId": self.target_id, "text": text, "url": self.current_url().await?}))
    }

    pub(super) async fn screenshot(&self, full_page: bool) -> Result<Value, BusinessError> {
        let params = ScreenshotParams::builder().full_page(full_page).build();
        let bytes = with_timeout(
            self.request_timeout,
            self.page.screenshot(params),
            CDP_COMMAND_FAILED,
            "Browser screenshot failed",
        )
        .await?;
        Ok(json!({"pageId": self.target_id, "base64": BASE64.encode(bytes)}))
    }
}
