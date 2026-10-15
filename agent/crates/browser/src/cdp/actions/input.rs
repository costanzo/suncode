use crate::cdp::error::{with_timeout, CDP_COMMAND_FAILED};
use crate::cdp::locator::{js_string, Locator};
use crate::cdp::session::CdpSession;
use chromiumoxide::cdp::browser_protocol::input::{DispatchKeyEventParams, DispatchKeyEventType};
use serde_json::Value;
use suncode_common::BusinessError;

impl CdpSession {
    pub(super) async fn click(&self, target: &Locator) -> Result<Value, BusinessError> {
        self.on_element(target, "e.click();").await
    }

    /// Sets the value and fires `input`/`change` so framework bindings update.
    pub(super) async fn fill(&self, target: &Locator, value: &str) -> Result<Value, BusinessError> {
        let value = js_string(value);
        self.on_element(
            target,
            &format!(
                "e.focus(); e.value={value}; e.dispatchEvent(new Event('input',{{bubbles:true}})); e.dispatchEvent(new Event('change',{{bubbles:true}}));"
            ),
        )
        .await
    }

    pub(super) async fn press(&self, key: &str) -> Result<Value, BusinessError> {
        let key_down = key_event(DispatchKeyEventType::KeyDown, key)?;
        let key_up = key_event(DispatchKeyEventType::KeyUp, key)?;
        with_timeout(
            self.request_timeout,
            self.page.execute(key_down),
            CDP_COMMAND_FAILED,
            "Browser key press failed",
        )
        .await?;
        with_timeout(
            self.request_timeout,
            self.page.execute(key_up),
            CDP_COMMAND_FAILED,
            "Browser key release failed",
        )
        .await?;
        Ok(self.page_ack())
    }

    /// Runs `body` with the located element bound to `e`; fails if not found.
    async fn on_element(&self, target: &Locator, body: &str) -> Result<Value, BusinessError> {
        let target = target.expression();
        let _: bool = self
            .evaluate(format!(
                "(()=>{{const e={target}; if(!e) throw Error('target'); {body} return true;}})()"
            ))
            .await?;
        Ok(self.page_ack())
    }
}

fn key_event(
    event_type: DispatchKeyEventType,
    key: &str,
) -> Result<DispatchKeyEventParams, BusinessError> {
    DispatchKeyEventParams::builder()
        .r#type(event_type)
        .key(key)
        .build()
        .map_err(BusinessError::invalid)
}
