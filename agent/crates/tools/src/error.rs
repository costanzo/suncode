//! Error construction shared by the operations.

use serde_json::{json, Value};
use suncode_common::BusinessError;

/// A failure the caller should not retry unchanged.
pub(crate) fn fatal(code: &str, message: impl Into<String>) -> BusinessError {
    BusinessError::new(code, message).with_retryable(false)
}

/// A transient failure the caller may retry.
pub(crate) fn retryable(code: &str, message: impl Into<String>) -> BusinessError {
    BusinessError::new(code, message).with_retryable(true)
}

/// The JSON failure shape returned by the public `Operations` API.
pub(crate) fn failure_value(failure: BusinessError) -> Value {
    json!({"code":failure.code,"message":failure.message,"retryable":failure.retryable})
}
