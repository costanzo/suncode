//! HTTP client construction, the redirect origin policy, and bounded reads.

use super::MAX_RESPONSE_BYTES;
use super::{check_cancelled, fetch_failure, map_request_error, OutputFormat};
use crate::context::NetworkSettings;
use reqwest::{
    blocking::{Client, Response},
    header::{ACCEPT, ACCEPT_LANGUAGE, USER_AGENT},
    redirect::Policy,
    Url,
};
use std::io::Read;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use suncode_common::BusinessError;
use suncode_http::ApplyHttpSettings;

const MAX_REDIRECTS: usize = 10;
const READ_CHUNK_BYTES: usize = 16 * 1024;
const ACCEPT_LANGUAGE_VALUE: &str = "en-US,en;q=0.9";

/// Builds a client whose redirects stay on the approved origin of `url`.
pub(super) fn build(url: &Url, network: &NetworkSettings) -> Result<Client, BusinessError> {
    let origin = url.clone();
    let trust = suncode_http::trust_roots(
        network.verify_https_certificates,
        network.use_system_certificates,
        network.certificate_path.as_deref(),
    )?;
    let proxy = suncode_http::proxy_route(&network.proxy)?;
    Client::builder()
        .redirect(Policy::custom(move |attempt| {
            if attempt.previous().len() >= MAX_REDIRECTS {
                return attempt.error("too many redirects");
            }
            if redirect_allowed(&origin, attempt.url()) {
                attempt.follow()
            } else {
                attempt.error("redirect target is outside the approved web origin")
            }
        }))
        .apply_trust(trust)
        .apply_proxy(proxy)
        .build()
        .map_err(|_| fetch_failure("web client could not be created", false))
}

/// Same host and endpoint, or the standard `http:80` to `https:443` upgrade,
/// without embedded credentials.
fn redirect_allowed(origin: &Url, target: &Url) -> bool {
    let origin_port = origin.port_or_known_default();
    let target_port = target.port_or_known_default();
    let same_host = target.host_str().is_some() && target.host_str() == origin.host_str();
    let same_endpoint = target.scheme() == origin.scheme() && target_port == origin_port;
    let standard_https_upgrade = origin.scheme() == "http"
        && origin_port == Some(80)
        && target.scheme() == "https"
        && target_port == Some(443);
    let has_credentials = !target.username().is_empty() || target.password().is_some();
    same_host
        && (same_endpoint || standard_https_upgrade)
        && matches!(target.scheme(), "http" | "https")
        && !has_credentials
}

pub(super) fn send(
    client: &Client,
    url: Url,
    format: OutputFormat,
    user_agent: &str,
    timeout: Duration,
) -> Result<Response, BusinessError> {
    client
        .get(url)
        .header(USER_AGENT, user_agent)
        .header(ACCEPT, format.accept())
        .header(ACCEPT_LANGUAGE, ACCEPT_LANGUAGE_VALUE)
        .timeout(timeout)
        .send()
        .map_err(map_request_error)
}

pub(super) fn read_bounded(
    response: &mut Response,
    cancellation: Option<&AtomicBool>,
) -> Result<Vec<u8>, BusinessError> {
    let mut output = Vec::new();
    let mut chunk = [0_u8; READ_CHUNK_BYTES];
    loop {
        check_cancelled(cancellation)?;
        let count = response
            .read(&mut chunk)
            .map_err(|_| fetch_failure("response body could not be read", true))?;
        if count == 0 {
            break;
        }
        if output.len() + count > MAX_RESPONSE_BYTES {
            return Err(fetch_failure("response exceeds the 5 MiB limit", false));
        }
        output.extend_from_slice(&chunk[..count]);
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::redirect_allowed;
    use reqwest::Url;

    fn allowed(origin: &str, target: &str) -> bool {
        redirect_allowed(&Url::parse(origin).unwrap(), &Url::parse(target).unwrap())
    }

    #[test]
    fn redirects_stay_on_the_approved_origin() {
        assert!(allowed("https://example.test/a", "https://example.test/b"));
        assert!(allowed("http://example.test/a", "https://example.test/a"));
        assert!(!allowed("https://example.test/a", "http://example.test/a"));
        assert!(!allowed(
            "http://example.test:8080/",
            "https://example.test/"
        ));
        assert!(!allowed("https://example.test/", "https://other.test/"));
        assert!(!allowed(
            "https://example.test/",
            "https://example.test:8443/"
        ));
        assert!(!allowed(
            "https://example.test/",
            "https://user:pw@example.test/"
        ));
    }
}
