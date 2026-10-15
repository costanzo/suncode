//! Browser error codes and the shared CDP timeout/error mapping.

use std::fmt::Display;
use std::future::Future;
use std::time::Duration;
use suncode_common::BusinessError;

pub const CDP_STARTUP_TIMEOUT: &str = "browser_cdp_startup_timeout";
pub const CDP_UNAVAILABLE: &str = "browser_cdp_unavailable";
pub const CDP_NO_PAGE: &str = "browser_cdp_no_page";
pub const CDP_REQUEST_TIMEOUT: &str = "browser_cdp_request_timeout";
pub const CDP_PROTOCOL_ERROR: &str = "browser_cdp_protocol_error";
pub const CDP_COMMAND_FAILED: &str = "browser_cdp_command_failed";
pub const CDP_EVALUATION_FAILED: &str = "browser_cdp_evaluation_failed";
pub const TOOL_UNKNOWN: &str = "browser_tool_unknown";

/// CDP transport failures are retryable: the manager reconnects to CEF.
pub(crate) fn browser_error(code: &str, message: impl Into<String>) -> BusinessError {
    BusinessError::new(code, message.into()).with_retryable(true)
}

/// Bounds a CDP future by `timeout` and maps its error to `code` with `context`.
pub(crate) async fn with_timeout<T, E: Display>(
    timeout: Duration,
    future: impl Future<Output = Result<T, E>>,
    code: &str,
    context: &str,
) -> Result<T, BusinessError> {
    tokio::time::timeout(timeout, future)
        .await
        .map_err(|_| browser_error(CDP_REQUEST_TIMEOUT, "CDP request timed out"))?
        .map_err(|error| browser_error(code, format!("{context}: {error}")))
}
