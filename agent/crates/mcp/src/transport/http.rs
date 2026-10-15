use crate::{config::TlsConfig, error::mcp_error};
use http::{HeaderName, HeaderValue};
use rmcp::transport::{
    streamable_http_client::StreamableHttpClientTransportConfig, StreamableHttpClientTransport,
};
use std::{
    collections::{BTreeMap, HashMap},
    time::Duration,
};
use suncode_common::{BusinessError, HttpProxyMode};

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
    let mut builder = reqwest::Client::builder()
        .pool_max_idle_per_host(0)
        .redirect(reqwest::redirect::Policy::none())
        .danger_accept_invalid_certs(!config.verify_certificates);
    let custom_certificate = if let Some(path) = &config.certificate_path {
        let pem = std::fs::read(path)
            .map_err(|_| BusinessError::invalid("custom certificate file is unavailable"))?;
        Some(
            reqwest::Certificate::from_pem(&pem)
                .map_err(|_| BusinessError::invalid("custom certificate file is invalid"))?,
        )
    } else {
        None
    };
    if config.use_system_certificates {
        if let Some(certificate) = custom_certificate {
            builder = builder.tls_certs_merge([certificate]);
        }
    } else {
        builder = builder.tls_certs_only(custom_certificate);
    }
    builder = match config.proxy.mode {
        HttpProxyMode::NoProxy => builder.no_proxy(),
        HttpProxyMode::System => builder,
        HttpProxyMode::Custom => {
            let mut proxy = reqwest::Proxy::all(&config.proxy.url)
                .map_err(|_| BusinessError::invalid("custom proxy URL is invalid"))?;
            if !config.proxy.username.is_empty() {
                proxy = proxy.basic_auth(&config.proxy.username, &config.proxy.password);
            }
            proxy = proxy.no_proxy(reqwest::NoProxy::from_string(
                &config.proxy.no_proxy_value(),
            ));
            builder.no_proxy().proxy(proxy)
        }
    };
    builder
        .build()
        .map_err(|error| mcp_error("mcp_http_client_failed", error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use suncode_common::HttpProxyConfiguration;

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
