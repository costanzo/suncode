//! Error construction shared by the operations.

use suncode_common::BusinessError;

/// A failure the caller should not retry unchanged.
pub(crate) fn fatal(code: &str, message: impl Into<String>) -> BusinessError {
    BusinessError::new(code, message).with_retryable(false)
}

/// A transient failure the caller may retry.
pub(crate) fn retryable(code: &str, message: impl Into<String>) -> BusinessError {
    BusinessError::new(code, message).with_retryable(true)
}
