//! Shared contracts and errors used by the Rust SunCode crates.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fmt;

/// The single business-level error crossing SunCode crate boundaries.
///
/// Lower-level adapters may use their native error types internally, but they
/// must convert them to this type before returning from a public crate API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BusinessError {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub details: Value,
    #[serde(skip)]
    pub retryable: bool,
    #[serde(skip)]
    pub provider_request_id: Option<String>,
}

impl BusinessError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: json!({}),
            retryable: false,
            provider_request_id: None,
        }
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = details;
        self
    }

    pub fn details(self, details: Value) -> Self {
        self.with_details(details)
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new("invalid_arguments", message)
    }

    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::new("agent_unavailable", message)
    }

    pub fn missing(kind: &str) -> Self {
        Self::new(format!("{}_not_found", kind), format!("{} not found", kind))
    }

    pub fn database(message: impl Into<String>) -> Self {
        Self::new("database_error", message)
    }

    pub fn provider(
        code: impl Into<String>,
        message: impl Into<String>,
        retryable: bool,
        provider_request_id: Option<String>,
    ) -> Self {
        let mut error = Self::new(code, message);
        error.retryable = retryable;
        error.provider_request_id = provider_request_id;
        error.details = json!({
            "retryable": error.retryable,
            "provider_request_id": error.provider_request_id,
        });
        error
    }

    pub fn with_retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self.details["retryable"] = json!(retryable);
        self
    }
}

impl fmt::Display for BusinessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for BusinessError {}

impl From<serde_json::Error> for BusinessError {
    fn from(error: serde_json::Error) -> Self {
        Self::new("serialization_error", error.to_string())
    }
}

impl From<std::io::Error> for BusinessError {
    fn from(error: std::io::Error) -> Self {
        Self::new("io_error", error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::BusinessError;
    use serde_json::json;

    #[test]
    fn preserves_the_shared_business_error_shape() {
        let error = BusinessError::provider(
            "transient",
            "provider request failed",
            true,
            Some("request-123".into()),
        );
        assert_eq!(error.code, "transient");
        assert_eq!(error.details["retryable"], json!(true));
        assert_eq!(error.details["provider_request_id"], json!("request-123"));
        assert_eq!(
            serde_json::to_value(&error).unwrap()["message"],
            json!("provider request failed")
        );
    }
}
