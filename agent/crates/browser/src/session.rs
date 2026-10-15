//! CDP session lifecycle: connect to CEF, attach to a page, dispatch requests.

use crate::command::BrowserCommand;
use crate::discovery::discover_page;
use crate::error::{browser_error, CDP_PROTOCOL_ERROR, CDP_STARTUP_TIMEOUT, CDP_UNAVAILABLE};
use crate::launch::CdpLaunch;
use chromiumoxide::handler::{Handler, HandlerConfig};
use chromiumoxide::{Browser, Page};
use futures_util::StreamExt;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::time::Duration;
use suncode_common::BusinessError;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/// One agent connection to a single CEF page target.
pub struct CdpSession {
    /// Retaining the Browser keeps chromiumoxide's command channel alive. CEF is
    /// external, so `close` only disconnects this client handler.
    _browser: Browser,
    pub(crate) page: Page,
    pub(crate) target_id: String,
    pub(crate) request_timeout: Duration,
    handler: Mutex<Option<JoinHandle<()>>>,
}

impl CdpSession {
    pub async fn connect(launch: CdpLaunch) -> Result<Self, BusinessError> {
        let handler_config = HandlerConfig {
            request_timeout: launch.request_timeout,
            ..HandlerConfig::default()
        };
        let (browser, handler) = tokio::time::timeout(
            launch.startup_timeout,
            Browser::connect_with_config(launch.endpoint(), handler_config),
        )
        .await
        .map_err(|_| browser_error(CDP_STARTUP_TIMEOUT, "CEF CDP connection timed out"))?
        .map_err(|error| {
            browser_error(
                CDP_UNAVAILABLE,
                format!("CEF CDP connection failed: {error}"),
            )
        })?;
        let handler_task = tokio::spawn(drive_handler(handler));
        let page = match discover_page(&browser, &launch).await {
            Ok(page) => page,
            Err(error) => {
                handler_task.abort();
                return Err(error);
            }
        };
        Ok(Self {
            _browser: browser,
            target_id: page.target_id().as_ref().to_owned(),
            page,
            request_timeout: launch.request_timeout,
            handler: Mutex::new(Some(handler_task)),
        })
    }

    /// Executes a wire request and deserializes its JSON result.
    pub async fn request<T: DeserializeOwned>(
        &self,
        method: &str,
        params: Value,
    ) -> Result<T, BusinessError> {
        let command = BrowserCommand::parse(method, &params)?;
        let value = self.execute(command).await?;
        serde_json::from_value(value).map_err(|error| {
            browser_error(
                CDP_PROTOCOL_ERROR,
                format!("CDP result for {method} is invalid: {error}"),
            )
        })
    }

    /// Disconnects the CDP handler. Idempotent; never closes CEF itself.
    pub async fn close(&self) {
        if let Some(task) = self.handler.lock().await.take() {
            task.abort();
        }
    }
}

/// Pumps chromiumoxide's handler until the WebSocket connection fails.
/// Invalid messages are already ignored by the default `HandlerConfig`.
async fn drive_handler(mut handler: Handler) {
    while let Some(result) = handler.next().await {
        if let Err(error) = result {
            suncode_common::logging::warn("browser.cdp", format!("handler_stopped error={error}"));
            break;
        }
    }
}
