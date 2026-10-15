//! HTTP client construction, the redirect origin policy, and bounded reads.

use super::MAX_RESPONSE_BYTES;
use super::{check_cancelled, fetch_failure, invalid, map_request_error, OutputFormat};
use crate::context::NetworkSettings;
use crate::error::fatal;
use reqwest::{
    blocking::{Client, ClientBuilder, Response},
    header::{ACCEPT, ACCEPT_LANGUAGE, USER_AGENT},
    redirect::Policy,
    Url,
};
use std::io::Read;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use suncode_common::{BusinessError, HttpProxyConfiguration, HttpProxyMode};

const MAX_REDIRECTS: usize = 10;
const READ_CHUNK_BYTES: usize = 16 * 1024;
const ACCEPT_LANGUAGE_VALUE: &str = "en-US,en;q=0.9";
const PEM_MARKER: &[u8] = b"-----BEGIN CERTIFICATE-----";
const DER_SEQUENCE_TAG: u8 = 0x30;

/// Builds a client whose redirects stay on the approved origin of `url`.
pub(super) fn build(url: &Url, network: &NetworkSettings) -> Result<Client, BusinessError> {
    let origin = url.clone();
    let verify = network.verify_https_certificates;
    let mut builder = Client::builder()
        .danger_accept_invalid_certs(!verify)
        .danger_accept_invalid_hostnames(!verify)
        .tls_built_in_root_certs(network.use_system_certificates)
        .redirect(Policy::custom(move |attempt| {
            if attempt.previous().len() >= MAX_REDIRECTS {
                return attempt.error("too many redirects");
            }
            if redirect_allowed(&origin, attempt.url()) {
                attempt.follow()
            } else {
                attempt.error("redirect target is outside the approved web origin")
            }
        }));
    // A custom certificate adds to the trusted roots whether or not the
    // built-in roots are enabled, matching the MCP HTTP client.
    if verify {
        if let Some(path) = network.certificate_path.as_deref() {
            let bytes = std::fs::read(path)
                .map_err(|_| fatal("certificate_unavailable", "certificate file is unavailable"))?;
            for certificate in load_certificates(&bytes)? {
                builder = builder.add_root_certificate(certificate);
            }
        }
    }
    apply_proxy_configuration(builder, &network.proxy)?
        .build()
        .map_err(|_| fetch_failure("web client could not be created", false))
}

/// Parses a PEM bundle or a single DER certificate. With rustls, reqwest
/// defers parsing to the handshake, so validate here to fail early.
fn load_certificates(bytes: &[u8]) -> Result<Vec<reqwest::Certificate>, BusinessError> {
    let invalid = || {
        fatal(
            "certificate_invalid",
            "certificate file is not a valid PEM or DER certificate",
        )
    };
    if bytes
        .windows(PEM_MARKER.len())
        .any(|window| window == PEM_MARKER)
    {
        let certificates = reqwest::Certificate::from_pem_bundle(bytes).map_err(|_| invalid())?;
        if certificates.is_empty() {
            return Err(invalid());
        }
        return Ok(certificates);
    }
    // A DER certificate is an ASN.1 SEQUENCE.
    if bytes.first() != Some(&DER_SEQUENCE_TAG) {
        return Err(invalid());
    }
    reqwest::Certificate::from_der(bytes)
        .map(|certificate| vec![certificate])
        .map_err(|_| invalid())
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

fn apply_proxy_configuration(
    builder: ClientBuilder,
    configuration: &HttpProxyConfiguration,
) -> Result<ClientBuilder, BusinessError> {
    match configuration.mode {
        HttpProxyMode::NoProxy => Ok(builder.no_proxy()),
        HttpProxyMode::System => Ok(builder),
        HttpProxyMode::Custom => {
            let mut proxy = reqwest::Proxy::all(&configuration.url)
                .map_err(|_| invalid("custom proxy URL is invalid"))?;
            if !configuration.username.is_empty() {
                proxy = proxy.basic_auth(&configuration.username, &configuration.password);
            }
            proxy = proxy.no_proxy(reqwest::NoProxy::from_string(
                &configuration.no_proxy_value(),
            ));
            Ok(builder.no_proxy().proxy(proxy))
        }
    }
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
    use super::{load_certificates, redirect_allowed};
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

    #[test]
    fn loads_a_system_pem_bundle() {
        // Present on macOS and most Linux distributions; skip where absent.
        let Some(bundle) = ["/etc/ssl/cert.pem", "/etc/ssl/certs/ca-certificates.crt"]
            .iter()
            .find_map(|path| std::fs::read(path).ok())
        else {
            return;
        };
        assert!(!load_certificates(&bundle).unwrap().is_empty());
    }
}
