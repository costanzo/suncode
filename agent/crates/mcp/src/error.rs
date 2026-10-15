use suncode_common::BusinessError;

pub(crate) fn mcp_error(code: &str, message: impl Into<String>) -> BusinessError {
    BusinessError::new(code, redact_error(&message.into())).with_retryable(true)
}

pub(crate) fn mcp_service_error(
    message: &str,
    error: &impl std::fmt::Display,
    secret_values: &[String],
) -> BusinessError {
    redact_business_error(
        mcp_error("mcp_protocol_error", format!("{message}: {error}")),
        secret_values,
    )
}

pub(crate) fn redact_business_error(
    mut error: BusinessError,
    secret_values: &[String],
) -> BusinessError {
    for secret in secret_values {
        error.message = error.message.replace(secret, "[redacted]");
    }
    error
}

fn redact_error(message: &str) -> String {
    let mut value = message.chars().take(500).collect::<String>();
    for marker in ["Authorization", "Bearer", "token=", "key="] {
        if let Some(index) = value
            .to_ascii_lowercase()
            .find(&marker.to_ascii_lowercase())
        {
            value.truncate(index);
            value.push_str("[redacted]");
            break;
        }
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_likely_credentials_from_errors() {
        assert_eq!(
            redact_error("request failed Authorization: secret"),
            "request failed [redacted]"
        );
    }

    #[test]
    fn redacts_configured_values_from_protocol_errors() {
        let error = mcp_service_error(
            "request failed",
            &"remote echoed opaque-secret-value",
            &["opaque-secret-value".into()],
        );
        assert_eq!(error.message, "request failed: remote echoed [redacted]");
    }
}
