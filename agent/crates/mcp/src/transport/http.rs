use crate::{config::TlsConfig, error::mcp_error};
use http::{HeaderName, HeaderValue};
use rmcp::transport::{
    streamable_http_client::StreamableHttpClientTransportConfig, StreamableHttpClientTransport,
};
use std::{
    collections::{BTreeMap, HashMap},
    time::Duration,
};
use suncode_common::BusinessError;
use suncode_http::ApplyHttpSettings;

const MAX_SSE_EVENT_BYTES: usize = 2 * 1024 * 1024;
const CONTROL_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const SESSION_RECOVERY_TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) fn transport(
    url: String,
    headers: BTreeMap<String, String>,
    tls: &TlsConfig,
) -> Result<StreamableHttpClientTransport<reqwest::Client>, BusinessError> {
    let headers = http_headers(headers)?;
    let client = http_client(tls)?;
    let config = StreamableHttpClientTransportConfig::with_uri(url)
        .max_concurrent_requests(1)
        .control_request_timeout(CONTROL_REQUEST_TIMEOUT)
        .session_recovery_timeout(SESSION_RECOVERY_TIMEOUT)
        .max_sse_event_size(MAX_SSE_EVENT_BYTES)
        .custom_headers(headers);
    Ok(StreamableHttpClientTransport::with_client(client, config))
}

fn http_headers(
    values: BTreeMap<String, String>,
) -> Result<HashMap<HeaderName, HeaderValue>, BusinessError> {
    values
        .into_iter()
        .map(|(name, value)| {
            let name = HeaderName::try_from(name)
                .map_err(|_| BusinessError::invalid("MCP HTTP header name is invalid"))?;
            let value = HeaderValue::try_from(value)
                .map_err(|_| BusinessError::invalid("MCP HTTP header value is invalid"))?;
            Ok((name, value))
        })
        .collect()
}

fn http_client(config: &TlsConfig) -> Result<reqwest::Client, BusinessError> {
    let trust = suncode_http::trust_roots(
        config.verify_certificates,
        config.use_system_certificates,
        config.certificate_path.as_deref(),
    )?;
    let proxy = suncode_http::proxy_route(&config.proxy)?;
    reqwest::Client::builder()
        .pool_max_idle_per_host(0)
        .redirect(reqwest::redirect::Policy::none())
        .apply_trust(trust)
        .apply_proxy(proxy)
        .build()
        .map_err(|error| mcp_error("mcp_http_client_failed", error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use suncode_common::{HttpProxyConfiguration, HttpProxyMode};

    #[test]
    fn builds_clients_for_each_proxy_mode() {
        for proxy in [
            HttpProxyConfiguration {
                mode: HttpProxyMode::NoProxy,
                ..HttpProxyConfiguration::default()
            },
            HttpProxyConfiguration::default(),
            HttpProxyConfiguration {
                mode: HttpProxyMode::Custom,
                url: "http://127.0.0.1:8080".into(),
                username: "developer".into(),
                password: "secret".into(),
                bypass: vec![".internal.example.test".into()],
            },
        ] {
            assert!(http_client(&TlsConfig {
                verify_certificates: true,
                use_system_certificates: true,
                certificate_path: None,
                proxy,
            })
            .is_ok());
        }
    }
}
