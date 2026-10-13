use reqwest::{Client, Response};
use serde::Serialize;
use serde_json::{json, Value};
use std::{error::Error, time::Duration};
use suncode_common::BusinessError;
use url::Url;

use crate::{
    config::{PairResponse, PersistedRemote},
    events::redact_remote_value,
    logging, RemoteNetworkConfiguration, RemoteResult,
};

pub(crate) async fn decode_pair_response(response: Response) -> RemoteResult<PairResponse> {
    if !response.status().is_success() {
        return Err(BusinessError::unavailable(format!(
            "Remote pairing failed with HTTP {}",
            response.status()
        )));
    }
    let body: Value = response
        .json()
        .await
        .map_err(|_| BusinessError::unavailable("Remote pairing response was invalid"))?;
    let data = body.get("data").unwrap_or(&body);
    serde_json::from_value(data.clone()).map_err(|_| {
        BusinessError::unavailable("Remote pairing response did not match the Desktop contract")
    })
}

pub(crate) fn remote_http_error(error: reqwest::Error) -> BusinessError {
    let mut causes = Vec::new();
    let mut source = error.source();
    while let Some(current) = source {
        causes.push(current.to_string());
        source = current.source();
    }
    let detail = causes.last().map(|cause| format!("; cause: {cause}"));
    BusinessError::unavailable(format!(
        "Remote Server request failed: {error}{}",
        detail.unwrap_or_default()
    ))
}

/// How long a request may take.
#[derive(Clone, Copy)]
pub(crate) enum RemoteTimeout {
    /// Bounds the whole request, including the response body.
    Total(Duration),
    /// For long-lived streams: bounds only connecting and the gap between bytes.
    Idle(Duration),
}

/// The relay sends an SSE heartbeat every 15 s; three missed beats mean the stream is dead.
pub(crate) const EVENT_STREAM_IDLE_TIMEOUT: Duration = Duration::from_secs(45);

pub(crate) fn remote_http_client(
    configuration: &RemoteNetworkConfiguration,
    timeout: RemoteTimeout,
) -> RemoteResult<Client> {
    let verify_certificates = configuration.verify_https_certificates;
    let use_system_certificates = configuration.use_system_certificates;
    let builder = match timeout {
        RemoteTimeout::Total(timeout) => Client::builder().timeout(timeout),
        RemoteTimeout::Idle(timeout) => Client::builder()
            .connect_timeout(timeout)
            .read_timeout(timeout),
    };
    let mut builder = builder
        .danger_accept_invalid_certs(!verify_certificates)
        .danger_accept_invalid_hostnames(!verify_certificates);
    if verify_certificates {
        if !use_system_certificates {
            builder = builder.tls_built_in_root_certs(false);
        }
        if let Some(path) = configuration.certificate_path.clone() {
            let bytes = std::fs::read(&path).map_err(|error| {
                BusinessError::unavailable(format!(
                    "Remote Server certificate file could not be read: {error}"
                ))
            })?;
            let certificate = reqwest::Certificate::from_pem(&bytes)
                .or_else(|_| reqwest::Certificate::from_der(&bytes))
                .map_err(|error| {
                    BusinessError::unavailable(format!(
                        "Remote Server certificate file is invalid: {error}"
                    ))
                })?;
            builder = builder.add_root_certificate(certificate);
        }
    }
    let proxy = configuration.proxy.clone();
    builder = match proxy.mode {
        suncode_common::HttpProxyMode::NoProxy => builder.no_proxy(),
        suncode_common::HttpProxyMode::System => builder,
        suncode_common::HttpProxyMode::Custom => {
            let mut configured = reqwest::Proxy::all(&proxy.url)
                .map_err(|_| BusinessError::unavailable("Remote Server proxy URL is invalid"))?;
            if !proxy.username.is_empty() {
                configured = configured.basic_auth(&proxy.username, &proxy.password);
            }
            configured =
                configured.no_proxy(reqwest::NoProxy::from_string(&proxy.no_proxy_value()));
            builder.no_proxy().proxy(configured)
        }
    };
    builder.build().map_err(|error| {
        BusinessError::unavailable(format!("Remote HTTP client could not start: {error}"))
    })
}

pub(crate) fn resolve_endpoint(base: &str, path: &str) -> String {
    let Ok(mut url) = Url::parse(base) else {
        return path.to_string();
    };
    if let Ok(absolute) = Url::parse(path) {
        return absolute.to_string();
    }
    let base_path = url.path().trim_end_matches('/');
    if path.starts_with('/') && !base_path.is_empty() {
        url.set_path(&format!("{base_path}/{}", path.trim_start_matches('/')));
        return url.to_string();
    }
    url.join(path)
        .map(|joined| joined.to_string())
        .unwrap_or_else(|_| path.to_string())
}

pub(crate) fn request_headers(
    request: reqwest::RequestBuilder,
    persisted: &PersistedRemote,
    host_id: &str,
) -> reqwest::RequestBuilder {
    request
        .header("X-Host-Id", host_id)
        .bearer_auth(&persisted.pairing.access_token)
}

pub(crate) fn log_http_request<T: Serialize>(method: &str, url: &str, body: &T) {
    let body = serde_json::to_value(body).unwrap_or_else(|_| json!({}));
    logging::debug(
        "remote",
        format!(
            "HTTP request method={method} url={url} query={} body={}",
            Url::parse(url)
                .ok()
                .and_then(|url| url.query().map(str::to_owned))
                .unwrap_or_default(),
            redact_remote_value(&body)
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_resolution_keeps_a_server_context_path() {
        assert_eq!(
            resolve_endpoint("https://example.test/remote-server", "/v1/desktop/events"),
            "https://example.test/remote-server/v1/desktop/events"
        );
        assert_eq!(
            resolve_endpoint("https://example.test", "/v1/desktop/events"),
            "https://example.test/v1/desktop/events"
        );
    }
}
