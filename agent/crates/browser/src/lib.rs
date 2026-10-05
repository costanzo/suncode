//! Rust-owned Chromium DevTools Protocol client for the CEF browser instance.
//!
//! Avalonia owns CEF and exposes a localhost remote debugging endpoint. This crate
//! connects to that already-running endpoint through `chromiumoxide`; it never
//! launches Chromium or owns browser UI state.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chromiumoxide::cdp::browser_protocol::input::{DispatchKeyEventParams, DispatchKeyEventType};
use chromiumoxide::handler::HandlerConfig;
use chromiumoxide::page::ScreenshotParams;
use chromiumoxide::{Browser, Page};
use futures_util::StreamExt;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use std::time::Duration;
use suncode_common::BusinessError;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

pub const PROTOCOL_VERSION: u32 = 1;
pub const DEFAULT_CDP_PORT: u16 = 9222;

#[derive(Debug, Clone)]
pub struct CdpLaunch {
    pub port: u16,
    pub startup_timeout: Duration,
    pub request_timeout: Duration,
}

/// Kept as a compatibility name for the core lifecycle while the transport is CDP.
pub type WorkerLaunch = CdpLaunch;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CdpTarget {
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub web_socket_debugger_url: Option<String>,
    #[serde(rename = "type", default)]
    pub target_type: String,
}

pub struct WorkerProcess {
    /// Retaining the Browser keeps chromiumoxide's command channel alive. CEF is
    /// external, so close/force_close only disconnect this client handler.
    _browser: Browser,
    page: Page,
    handler: Mutex<Option<JoinHandle<()>>>,
    request_timeout: Duration,
    pub target: CdpTarget,
}

impl WorkerProcess {
    pub async fn spawn(launch: WorkerLaunch) -> Result<Self, BusinessError> {
        let endpoint = format!("http://127.0.0.1:{}/", launch.port);
        let handler_config = HandlerConfig {
            request_timeout: launch.request_timeout,
            ..HandlerConfig::default()
        };
        let (browser, mut handler) = tokio::time::timeout(
            launch.startup_timeout,
            Browser::connect_with_config(endpoint, handler_config),
        )
        .await
        .map_err(|_| {
            browser_error(
                "browser_cdp_startup_timeout",
                "CEF CDP connection timed out",
            )
        })?
        .map_err(|error| {
            browser_error(
                "browser_cdp_unavailable",
                format!("CEF CDP connection failed: {error}"),
            )
        })?;
        let handler_task = tokio::spawn(async move {
            while let Some(result) = handler.next().await {
                if result.is_err() {
                    break;
                }
            }
        });
        let pages = tokio::time::timeout(launch.startup_timeout, browser.pages())
            .await
            .map_err(|_| {
                browser_error(
                    "browser_cdp_startup_timeout",
                    "CEF page discovery timed out",
                )
            })?
            .map_err(|error| {
                browser_error(
                    "browser_cdp_protocol_error",
                    format!("CEF pages could not be discovered: {error}"),
                )
            })?;
        let page = pages.into_iter().next().ok_or_else(|| {
            browser_error("browser_cdp_no_page", "CEF did not expose a page target")
        })?;
        let target = CdpTarget {
            id: page.target_id().as_ref().to_owned(),
            title: String::new(),
            url: page.url().await.ok().flatten().unwrap_or_default(),
            web_socket_debugger_url: None,
            target_type: "page".into(),
        };
        Ok(Self {
            _browser: browser,
            page,
            handler: Mutex::new(Some(handler_task)),
            request_timeout: launch.request_timeout,
            target,
        })
    }

    pub async fn request<T: DeserializeOwned>(
        &self,
        method: &str,
        params: Value,
    ) -> Result<T, BusinessError> {
        let value = self.browser_request(method, params).await?;
        serde_json::from_value(value).map_err(|error| {
            browser_error(
                "browser_cdp_protocol_error",
                format!("CDP result for {method} is invalid: {error}"),
            )
        })
    }

    pub async fn close(&self) {
        if let Some(task) = self.handler.lock().await.take() {
            task.abort();
        }
    }

    pub async fn force_close(&self) {
        self.close().await;
    }

    async fn evaluate<T: DeserializeOwned>(&self, expression: String) -> Result<T, BusinessError> {
        let result = tokio::time::timeout(self.request_timeout, self.page.evaluate(expression))
            .await
            .map_err(|_| browser_error("browser_cdp_request_timeout", "CDP request timed out"))?
            .map_err(|error| {
                browser_error(
                    "browser_cdp_evaluation_failed",
                    format!("Browser DOM operation failed: {error}"),
                )
            })?;
        result.into_value().map_err(|error| {
            browser_error(
                "browser_cdp_protocol_error",
                format!("Browser evaluation result is invalid: {error}"),
            )
        })
    }

    pub async fn browser_request(
        &self,
        method: &str,
        params: Value,
    ) -> Result<Value, BusinessError> {
        match method {
            "start" | "state" => Ok(json!({
                "started": true,
                "controlOwner": "agent",
                "pages": [{"id": self.target.id, "url": self.current_url().await?}]
            })),
            "probe" => Ok(json!({
                "protocolVersion": PROTOCOL_VERSION,
                "transport": "cdp",
                "targetId": self.target.id
            })),
            "open" | "navigate" => {
                let url = params
                    .get("url")
                    .and_then(Value::as_str)
                    .ok_or_else(|| BusinessError::invalid("url is required"))?;
                tokio::time::timeout(self.request_timeout, self.page.goto(url))
                    .await
                    .map_err(|_| {
                        browser_error("browser_cdp_request_timeout", "CDP request timed out")
                    })?
                    .map_err(|error| {
                        browser_error(
                            "browser_cdp_command_failed",
                            format!("Browser navigation failed: {error}"),
                        )
                    })?;
                Ok(json!({"pageId": self.target.id, "url": self.current_url().await?}))
            }
            "snapshot" => {
                let text: String = self
                    .evaluate("document.body ? document.body.innerText : ''".into())
                    .await?;
                Ok(
                    json!({"pageId": self.target.id, "text": text, "url": self.current_url().await?}),
                )
            }
            "screenshot" => {
                let full_page = params
                    .get("fullPage")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let bytes = tokio::time::timeout(
                    self.request_timeout,
                    self.page
                        .screenshot(ScreenshotParams::builder().full_page(full_page).build()),
                )
                .await
                .map_err(|_| browser_error("browser_cdp_request_timeout", "CDP request timed out"))?
                .map_err(|error| {
                    browser_error(
                        "browser_cdp_command_failed",
                        format!("Browser screenshot failed: {error}"),
                    )
                })?;
                Ok(json!({"pageId": self.target.id, "base64": BASE64.encode(bytes)}))
            }
            "click" => {
                let target = target_expression(params.get("target"))?;
                let _: bool = self
                    .evaluate(format!("(()=>{{const e={target}; if(!e) throw Error('target'); e.click(); return true;}})()"))
                    .await?;
                Ok(json!({"pageId": self.target.id}))
            }
            "fill" => {
                let target = target_expression(params.get("target"))?;
                let value = params
                    .get("value")
                    .and_then(Value::as_str)
                    .ok_or_else(|| BusinessError::invalid("value is required"))?;
                let encoded = serde_json::to_string(value).unwrap();
                let _: bool = self
                    .evaluate(format!("(()=>{{const e={target}; if(!e) throw Error('target'); e.focus(); e.value={encoded}; e.dispatchEvent(new Event('input',{{bubbles:true}})); e.dispatchEvent(new Event('change',{{bubbles:true}})); return true;}})()"))
                    .await?;
                Ok(json!({"pageId": self.target.id}))
            }
            "press" => {
                let key = params
                    .get("key")
                    .and_then(Value::as_str)
                    .ok_or_else(|| BusinessError::invalid("key is required"))?;
                let key_down = DispatchKeyEventParams::builder()
                    .r#type(DispatchKeyEventType::KeyDown)
                    .key(key)
                    .build()
                    .map_err(BusinessError::invalid)?;
                let key_up = DispatchKeyEventParams::builder()
                    .r#type(DispatchKeyEventType::KeyUp)
                    .key(key)
                    .build()
                    .map_err(BusinessError::invalid)?;
                tokio::time::timeout(self.request_timeout, self.page.execute(key_down))
                    .await
                    .map_err(|_| {
                        browser_error("browser_cdp_request_timeout", "CDP request timed out")
                    })?
                    .map_err(|error| {
                        browser_error(
                            "browser_cdp_command_failed",
                            format!("Browser key press failed: {error}"),
                        )
                    })?;
                tokio::time::timeout(self.request_timeout, self.page.execute(key_up))
                    .await
                    .map_err(|_| {
                        browser_error("browser_cdp_request_timeout", "CDP request timed out")
                    })?
                    .map_err(|error| {
                        browser_error(
                            "browser_cdp_command_failed",
                            format!("Browser key release failed: {error}"),
                        )
                    })?;
                Ok(json!({"pageId": self.target.id}))
            }
            "close_page" => {
                tokio::time::timeout(self.request_timeout, self.page.clone().close())
                    .await
                    .map_err(|_| {
                        browser_error("browser_cdp_request_timeout", "CDP request timed out")
                    })?
                    .map_err(|error| {
                        browser_error(
                            "browser_cdp_command_failed",
                            format!("Browser page close failed: {error}"),
                        )
                    })?;
                Ok(json!({"pageId": self.target.id}))
            }
            "take_control" | "return_control" => Ok(json!({
                "started": true,
                "controlOwner": "agent",
                "pages": [{"id": self.target.id, "url": self.current_url().await?}]
            })),
            _ => Err(BusinessError::new(
                "browser_tool_unknown",
                "Browser CDP command is not supported",
            )),
        }
    }

    async fn current_url(&self) -> Result<String, BusinessError> {
        tokio::time::timeout(self.request_timeout, self.page.url())
            .await
            .map_err(|_| browser_error("browser_cdp_request_timeout", "CDP request timed out"))?
            .map(|url| url.unwrap_or_default())
            .map_err(|error| {
                browser_error(
                    "browser_cdp_protocol_error",
                    format!("Browser URL could not be read: {error}"),
                )
            })
    }
}

fn target_expression(target: Option<&Value>) -> Result<String, BusinessError> {
    let target = target
        .and_then(Value::as_object)
        .ok_or_else(|| BusinessError::invalid("target is required"))?;
    let kind = target
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| BusinessError::invalid("target kind is required"))?;
    if kind == "ref" {
        return Err(BusinessError::new(
            "browser_reference_stale",
            "CDP element references must be refreshed from the current snapshot",
        ));
    }
    let value = target
        .get("value")
        .and_then(Value::as_str)
        .or_else(|| target.get("name").and_then(Value::as_str))
        .ok_or_else(|| BusinessError::invalid("target value is required"))?;
    let encoded = serde_json::to_string(value).unwrap();
    match kind {
        "label" => Ok(format!("document.querySelector('label[for='+{encoded}+']')?.control || document.querySelector('[aria-label='+{encoded}+']')")),
        "placeholder" => Ok(format!("document.querySelector('[placeholder='+{encoded}+']')")),
        "test_id" => Ok(format!("document.querySelector('[data-testid='+{encoded}+']')")),
        "text" => Ok(format!("[...document.querySelectorAll('button,a,input,textarea,select')].find(e=>e.innerText.trim()=== {encoded})")),
        "role" => Ok(format!("[...document.querySelectorAll('[role]')].find(e=>e.getAttribute('role')==={} && e.innerText.trim()==={})", serde_json::to_string(target.get("role").and_then(Value::as_str).unwrap_or("button")).unwrap(), encoded)),
        _ => Err(BusinessError::invalid("unsupported target kind")),
    }
}

fn browser_error(code: &str, message: impl Into<String>) -> BusinessError {
    BusinessError::new(code, message.into()).with_retryable(true)
}

pub fn target_name() -> &'static str {
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "darwin-arm64"
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "win32-x64"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "linux-x64"
    } else {
        "unsupported"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_is_stable() {
        assert!(!target_name().is_empty());
    }

    #[test]
    fn target_expression_rejects_unbounded_selectors() {
        assert!(target_expression(Some(&json!({"css":"body"}))).is_err());
    }
}
