//! Stable LSP error codes and message redaction.

use suncode_common::BusinessError;

pub(crate) fn lsp_error(code: &str, message: impl Into<String>) -> BusinessError {
    BusinessError::new(code, message).with_retryable(matches!(
        code,
        "lsp_process_start_failed"
            | "lsp_startup_timeout"
            | "lsp_request_timeout"
            | "lsp_connection_closed"
            | "lsp_write_timeout"
            | "lsp_io_failed"
    ))
}

pub(crate) fn redact<'a>(message: &str, values: impl IntoIterator<Item = &'a String>) -> String {
    let mut redacted = message.replace(['\r', '\n'], " ");
    for value in values {
        if value.len() >= 4 {
            redacted = redacted.replace(value, "[redacted]");
        }
    }
    redacted.chars().take(1_024).collect()
}
