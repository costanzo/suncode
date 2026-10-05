//! Rust-owned Chrome DevTools Protocol client for the CEF browser instance.
//!
//! The desktop process owns CEF and exposes a localhost remote debugging port. This
//! crate discovers a page target and sends bounded CDP commands over WebSocket. It
//! does not start a browser process, load JavaScript packages, or own UI state.

use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use suncode_common::BusinessError;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

const MAX_MESSAGE_BYTES: usize = 8 * 1024 * 1024;
pub const PROTOCOL_VERSION: u32 = 1;
pub const DEFAULT_CDP_PORT: u16 = 9222;

type CdpSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug, Clone)]
pub struct CdpLaunch {
    pub port: u16,
    pub startup_timeout: Duration,
    pub request_timeout: Duration,
}

/// Kept as a compatibility name for the core lifecycle while the transport is CDP.
pub type WorkerLaunch = CdpLaunch;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
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

#[derive(Debug, Deserialize)]
struct CdpResponse {
    id: u64,
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<CdpError>,
}

#[derive(Debug, Deserialize)]
struct CdpError {
    code: i64,
    message: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RemoteObject {
    #[serde(default)]
    value: Option<Value>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EvaluateResult {
    #[serde(default)]
    result: RemoteObject,
    #[serde(default)]
    exception_details: Option<Value>,
}

pub struct WorkerProcess {
    socket: Mutex<CdpSocket>,
    next_id: AtomicU64,
    request_timeout: Duration,
    pub target: CdpTarget,
}

impl WorkerProcess {
    pub async fn spawn(launch: WorkerLaunch) -> Result<Self, BusinessError> {
        let client = Client::builder()
            .timeout(launch.startup_timeout)
            .build()
            .map_err(|e| {
                browser_error(
                    "browser_cdp_unavailable",
                    format!("CDP client could not start: {e}"),
                )
            })?;
        let endpoint = format!("http://127.0.0.1:{}/json/list", launch.port);
        let targets = client
            .get(endpoint)
            .send()
            .await
            .map_err(|e| {
                browser_error(
                    "browser_cdp_unavailable",
                    format!("CEF remote debugging is unavailable: {e}"),
                )
            })?
            .error_for_status()
            .map_err(|e| {
                browser_error(
                    "browser_cdp_unavailable",
                    format!("CEF remote debugging returned an error: {e}"),
                )
            })?
            .json::<Vec<CdpTarget>>()
            .await
            .map_err(|e| {
                browser_error(
                    "browser_cdp_protocol_error",
                    format!("CEF target list is invalid: {e}"),
                )
            })?;
        let target = targets
            .into_iter()
            .find(|target| target.target_type == "page" && target.web_socket_debugger_url.is_some())
            .ok_or_else(|| {
                browser_error("browser_cdp_no_page", "CEF did not expose a page target")
            })?;
        let ws_url = target.web_socket_debugger_url.clone().unwrap();
        let (socket, _) = tokio::time::timeout(launch.startup_timeout, connect_async(ws_url))
            .await
            .map_err(|_| {
                browser_error(
                    "browser_cdp_startup_timeout",
                    "CEF CDP connection timed out",
                )
            })?
            .map_err(|e| {
                browser_error(
                    "browser_cdp_unavailable",
                    format!("CEF CDP WebSocket connection failed: {e}"),
                )
            })?;
        let process = Self {
            socket: Mutex::new(socket),
            next_id: AtomicU64::new(1),
            request_timeout: launch.request_timeout,
            target,
        };
        process.command("Page.enable", json!({})).await?;
        process.command("Runtime.enable", json!({})).await?;
        Ok(process)
    }

    pub async fn request<T: DeserializeOwned>(
        &self,
        method: &str,
        params: Value,
    ) -> Result<T, BusinessError> {
        let value = self.browser_request(method, params).await?;
        serde_json::from_value(value).map_err(|e| {
            browser_error(
                "browser_cdp_protocol_error",
                format!("CDP result for {method} is invalid: {e}"),
            )
        })
    }

    async fn command(&self, method: &str, params: Value) -> Result<Value, BusinessError> {
        self.dispatch(method, params).await
    }

    async fn dispatch(&self, method: &str, params: Value) -> Result<Value, BusinessError> {
        if method.trim().is_empty() {
            return Err(BusinessError::invalid("CDP method is required"));
        }
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let payload = serde_json::to_string(&json!({"id": id, "method": method, "params": params}))
            .map_err(|_| {
                browser_error(
                    "browser_cdp_protocol_error",
                    "CDP request could not be encoded",
                )
            })?;
        if payload.len() > MAX_MESSAGE_BYTES {
            return Err(browser_error(
                "browser_cdp_protocol_error",
                "CDP request exceeds the maximum size",
            ));
        }
        let mut socket = self.socket.lock().await;
        socket
            .send(Message::Text(payload.into()))
            .await
            .map_err(ws_error)?;
        loop {
            let message = tokio::time::timeout(self.request_timeout, socket.next())
                .await
                .map_err(|_| browser_error("browser_cdp_request_timeout", "CDP request timed out"))?
                .ok_or_else(|| {
                    browser_error("browser_cdp_protocol_error", "CEF CDP connection closed")
                })?
                .map_err(ws_error)?;
            let text = match message {
                Message::Text(text) => text.to_string(),
                Message::Binary(bytes) => String::from_utf8(bytes.to_vec()).map_err(|_| {
                    browser_error("browser_cdp_protocol_error", "CDP response is not UTF-8")
                })?,
                Message::Ping(_) | Message::Pong(_) => continue,
                Message::Close(_) => {
                    return Err(browser_error(
                        "browser_cdp_protocol_error",
                        "CEF CDP connection closed",
                    ))
                }
                _ => continue,
            };
            if text.len() > MAX_MESSAGE_BYTES {
                return Err(browser_error(
                    "browser_cdp_protocol_error",
                    "CDP response exceeds the maximum size",
                ));
            }
            let response: CdpResponse = serde_json::from_str(&text).map_err(|_| {
                browser_error("browser_cdp_protocol_error", "CDP response is invalid JSON")
            })?;
            if response.id != id {
                continue;
            }
            if let Some(error) = response.error {
                return Err(browser_error(
                    "browser_cdp_command_failed",
                    format!("CDP command failed ({}): {}", error.code, error.message),
                ));
            }
            return Ok(response.result.unwrap_or(Value::Null));
        }
    }

    pub async fn close(&self) {
        let _ = self.command("Page.stopLoading", json!({})).await;
    }
    pub async fn force_close(&self) {}

    async fn evaluate(&self, expression: String) -> Result<Value, BusinessError> {
        let value = self
            .dispatch(
                "Runtime.evaluate",
                json!({"expression": expression, "returnByValue": true, "awaitPromise": true}),
            )
            .await?;
        let result: EvaluateResult = serde_json::from_value(value).map_err(|e| {
            browser_error(
                "browser_cdp_protocol_error",
                format!("Runtime.evaluate result is invalid: {e}"),
            )
        })?;
        if result.exception_details.is_some() {
            return Err(browser_error(
                "browser_cdp_evaluation_failed",
                "Browser DOM operation failed",
            ));
        }
        Ok(result
            .result
            .value
            .or_else(|| result.result.description.map(Value::String))
            .unwrap_or(Value::Null))
    }

    pub async fn browser_request(
        &self,
        method: &str,
        params: Value,
    ) -> Result<Value, BusinessError> {
        match method {
            "start" | "state" => Ok(
                json!({"started": true, "controlOwner": "agent", "pages": [{"id": self.target.id, "url": self.target.url}]}),
            ),
            "probe" => Ok(
                json!({"protocolVersion": PROTOCOL_VERSION, "transport": "cdp", "targetId": self.target.id}),
            ),
            "open" => {
                let url = params
                    .get("url")
                    .and_then(Value::as_str)
                    .ok_or_else(|| BusinessError::invalid("url is required"))?;
                self.command("Page.navigate", json!({"url": url})).await?;
                Ok(json!({"pageId": self.target.id, "url": url}))
            }
            "navigate" => {
                let url = params
                    .get("url")
                    .and_then(Value::as_str)
                    .ok_or_else(|| BusinessError::invalid("url is required"))?;
                self.command("Page.navigate", json!({"url": url})).await?;
                Ok(json!({"pageId": self.target.id, "url": url}))
            }
            "snapshot" => {
                let value = self
                    .evaluate("document.body ? document.body.innerText : ''".into())
                    .await?;
                Ok(json!({"pageId": self.target.id, "text": value, "url": self.target.url}))
            }
            "screenshot" => {
                let result: Value = self
                    .dispatch("Page.captureScreenshot", json!({"format":"png"}))
                    .await?;
                Ok(
                    json!({"pageId": self.target.id, "base64": result.get("data").cloned().unwrap_or(Value::Null)}),
                )
            }
            "click" => {
                let target = target_expression(params.get("target"))?;
                self.evaluate(format!("(()=>{{const e={target}; if(!e) throw Error('target'); e.click(); return true;}})()")) .await?;
                Ok(json!({"pageId": self.target.id}))
            }
            "fill" => {
                let target = target_expression(params.get("target"))?;
                let value = params
                    .get("value")
                    .and_then(Value::as_str)
                    .ok_or_else(|| BusinessError::invalid("value is required"))?;
                let encoded = serde_json::to_string(value).unwrap();
                self.evaluate(format!("(()=>{{const e={target}; if(!e) throw Error('target'); e.focus(); e.value={encoded}; e.dispatchEvent(new Event('input',{{bubbles:true}})); e.dispatchEvent(new Event('change',{{bubbles:true}})); return true;}})()")) .await?;
                Ok(json!({"pageId": self.target.id}))
            }
            "press" => {
                let key = params
                    .get("key")
                    .and_then(Value::as_str)
                    .ok_or_else(|| BusinessError::invalid("key is required"))?;
                self.command(
                    "Input.dispatchKeyEvent",
                    json!({"type":"keyDown","key":key}),
                )
                .await?;
                self.command("Input.dispatchKeyEvent", json!({"type":"keyUp","key":key}))
                    .await?;
                Ok(json!({"pageId": self.target.id}))
            }
            "close_page" => {
                self.command("Page.close", json!({})).await?;
                Ok(json!({"pageId": self.target.id}))
            }
            "take_control" | "return_control" => Ok(
                json!({"started": true, "controlOwner": "agent", "pages": [{"id": self.target.id, "url": self.target.url}]}),
            ),
            _ => Err(BusinessError::new(
                "browser_tool_unknown",
                "Browser CDP command is not supported",
            )),
        }
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
        "role" => Ok(format!("[...document.querySelectorAll({})].find(e=>e.getAttribute('role')==={} && e.innerText.trim()==={})", serde_json::to_string("[role]").unwrap(), serde_json::to_string(target.get("role").and_then(Value::as_str).unwrap_or("button")).unwrap(), encoded)),
        _ => Err(BusinessError::invalid("unsupported target kind")),
    }
}

fn ws_error(error: tokio_tungstenite::tungstenite::Error) -> BusinessError {
    browser_error(
        "browser_cdp_protocol_error",
        format!("CDP WebSocket failed: {error}"),
    )
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
