use crate::BusinessError;
use std::{error::Error, io::ErrorKind, time::SystemTime};

pub(crate) fn retryable_status(status: u16) -> bool {
    matches!(status, 408 | 429 | 500 | 502 | 503 | 504)
}

pub(crate) fn retry_after_ms(value: Option<&str>, now: SystemTime) -> Option<u64> {
    let value = value?.trim();
    if !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Some(
            value
                .parse::<u64>()
                .unwrap_or(u64::MAX)
                .saturating_mul(1000),
        );
    }
    let deadline = httpdate::parse_http_date(value).ok()?;
    Some(
        deadline
            .duration_since(now)
            .unwrap_or_default()
            .as_millis()
            .min(u64::MAX as u128) as u64,
    )
}

pub(crate) fn is_transient_transport(error: &(dyn Error + 'static)) -> bool {
    if let Some(error) = error.downcast_ref::<reqwest::Error>() {
        if error.is_timeout() {
            return true;
        }
        if error.is_builder() || error.is_redirect() {
            return false;
        }
    }
    let mut source = Some(error);
    while let Some(error) = source {
        if let Some(error) = error.downcast_ref::<std::io::Error>() {
            if matches!(
                error.kind(),
                ErrorKind::ConnectionRefused
                    | ErrorKind::ConnectionReset
                    | ErrorKind::ConnectionAborted
                    | ErrorKind::BrokenPipe
                    | ErrorKind::TimedOut
                    | ErrorKind::UnexpectedEof
                    | ErrorKind::NotConnected
            ) {
                return true;
            }
        }
        if let Some(error) = error.downcast_ref::<hyper::Error>() {
            if error.is_closed() || error.is_incomplete_message() {
                return true;
            }
        }
        source = error.source();
    }
    false
}

pub(crate) fn transport_error(
    error: reqwest::Error,
    label: &str,
    request_id: Option<String>,
    response_started: bool,
) -> BusinessError {
    let retryable = !response_started && is_transient_transport(&error);
    let mut message = format!("{label} request transport failed: {error}");
    let mut source = error.source();
    while let Some(cause) = source {
        message.push_str(&format!(": {cause}"));
        source = cause.source();
    }
    let mut result = BusinessError::provider(
        "transient",
        message,
        retryable,
        request_id,
    );
    result.details["response_started"] = response_started.into();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn retry_after_accepts_seconds_and_dates_without_overflow() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
        assert_eq!(retry_after_ms(Some(" 3 "), now), Some(3000));
        let date = httpdate::fmt_http_date(now + Duration::from_secs(10));
        assert_eq!(retry_after_ms(Some(&date), now), Some(10000));
        assert_eq!(
            retry_after_ms(
                Some(&httpdate::fmt_http_date(now - Duration::from_secs(1))),
                now
            ),
            Some(0)
        );
        assert_eq!(
            retry_after_ms(Some("999999999999999999999999999"), now),
            Some(u64::MAX)
        );
        for value in ["-1", "1.5", "invalid", ""] {
            assert_eq!(retry_after_ms(Some(value), now), None);
        }
    }

    #[test]
    fn only_temporary_statuses_and_transport_errors_are_retryable() {
        for status in [408, 429, 500, 502, 503, 504] {
            assert!(retryable_status(status));
        }
        for status in [400, 401, 403, 404, 422, 501, 505] {
            assert!(!retryable_status(status));
        }
        assert!(is_transient_transport(&std::io::Error::from(
            ErrorKind::ConnectionReset
        )));
        assert!(!is_transient_transport(&std::io::Error::new(
            ErrorKind::InvalidData,
            "invalid certificate"
        )));
    }
}
