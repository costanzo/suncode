use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use futures_util::StreamExt;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashSet,
    error::Error,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    thread::JoinHandle,
    time::Duration,
};
use suncode_common::{BusinessError, HttpProxyConfiguration};
use suncode_data::Store;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use url::Url;

pub type RemoteResult<T> = Result<T, BusinessError>;

#[derive(Clone)]
pub struct RemoteNetworkConfiguration {
    pub verify_https_certificates: bool,
    pub use_system_certificates: bool,
    pub certificate_path: Option<std::path::PathBuf>,
    pub proxy: HttpProxyConfiguration,
}

#[derive(Debug, Clone)]
pub struct RemoteSessionEvent {
    pub session_id: String,
    pub occurred_at: String,
    pub event_type: String,
    pub payload: Value,
}

/// Agent-owned operations used by the transport worker. The transport crate
/// never reaches into SDK DTOs or persistence semantics.
pub trait RemoteHost: Send + Sync {
    fn network_configuration(&self) -> RemoteNetworkConfiguration;
    fn snapshot(&self) -> RemoteResult<Value>;
    fn subscribe_session_events(
        &self,
        session_id: &str,
    ) -> RemoteResult<mpsc::Receiver<RemoteResult<RemoteSessionEvent>>>;
    fn dispatch<'a>(
        &'a self,
        request_id: &'a str,
        operation: &'a str,
        arguments: Value,
    ) -> Pin<Box<dyn Future<Output = RemoteResult<Value>> + Send + 'a>>;
}

mod logging {
    pub fn debug(_scope: &str, _message: impl std::fmt::Display) {}
    pub fn warn(_scope: &str, _message: impl std::fmt::Display) {}
    pub fn error(_scope: &str, _message: impl std::fmt::Display) {}
    pub fn write_business_error(
        _scope: &str,
        _operation: &str,
        _error: &suncode_common::BusinessError,
        _context: impl std::fmt::Display,
    ) {
    }
}

const REMOTE_SERVER_URL: &str = "remote_server_url";
const REMOTE_PAIRING_CODE: &str = "remote_pairing_code";
const REMOTE_HOST_ID: &str = "remote_host_id";
const REMOTE_DESKTOP_TOKEN: &str = "remote_desktop_token";
const REMOTE_REFRESH_TOKEN: &str = "remote_refresh_token";
const REMOTE_ACCESS_TOKEN_EXPIRES_AT: &str = "remote_access_token_expires_at";
const REMOTE_MOBILE_PAIRING_CODE: &str = "remote_mobile_pairing_code";
const REMOTE_AES_KEY: &str = "remote_aes_key";
const REMOTE_E2E_ENABLED: &str = "remote_e2e_enabled";
const REMOTE_DELTA_MAX_CHARS: usize = 160;
const REMOTE_DELTA_FLUSH_INTERVAL: Duration = Duration::from_millis(120);

const UPLOAD_EVENT_TYPES: &[&str] = &[
    "turn.state",
    "tool.state",
    "tool.requested",
    "tool.result",
    "message.user",
    "message.assistant",
    "assistant.delta",
    "message.tool",
    "turn.queued",
    "turn.completed",
    "usage.updated",
    "context.compacted",
    "approval.requested",
    "approval.resolved",
    "question.asked",
    "question.replied",
    "question.rejected",
    "todo.updated",
    "checkpoint.captured",
    "checkpoint.item_restored",
    "checkpoint.restore_failed",
    "checkpoint.restored",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RemoteServerConfiguration {
    pub server_url: String,
    pub pairing_code: String,
    #[serde(default = "default_e2e_enabled")]
    pub e2e_enabled: bool,
}

fn default_e2e_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RemoteServerStatus {
    pub configured: bool,
    pub connected: bool,
    pub connecting: bool,
    pub host_id: Option<String>,
    pub mobile_pairing_payload: Option<String>,
    pub access_token_expires_at: Option<String>,
    pub mobile_pairing_code: Option<String>,
    pub mobile_pairing_url: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PairRequest {
    pairing_code: String,
    display_name: String,
    agent_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PairResponse {
    host_id: String,
    access_token: String,
    refresh_token: String,
    access_token_expires_at: String,
    mobile_pairing_code: String,
    #[serde(skip)]
    aes_key: String,
}

impl PairResponse {
    fn mobile_pairing_url(&self, endpoint: &str, e2e_enabled: bool) -> String {
        let separator = if endpoint.contains('?') { '&' } else { '?' };
        format!(
            "{}{}code={}&hostId={}&k={}&e2e={}",
            endpoint.trim_end_matches('/'),
            separator,
            urlencoding(&self.mobile_pairing_code),
            urlencoding(&self.host_id),
            urlencoding(&self.aes_key),
            if e2e_enabled { "1" } else { "0" }
        )
    }
}

fn urlencoding(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn generate_aes_key() -> String {
    let mut key = [0u8; 32];
    key[..16].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
    key[16..].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
    URL_SAFE_NO_PAD.encode(key)
}

#[derive(Debug, Clone, Deserialize)]
struct RemoteRequest {
    #[serde(alias = "requestId")]
    request_id: String,
    #[serde(alias = "command", default)]
    operation: String,
    #[serde(default, alias = "payload")]
    arguments: Value,
}

/// Desktop failure body for `POST /v1/desktop/responses`. Successful results are
/// posted as the bare command payload (or `encPayload`), never wrapped.
#[derive(Debug, Serialize)]
struct RemoteErrorBody {
    code: i32,
    message: String,
}

const REMOTE_DESKTOP_ERROR_CODE: i32 = 50400;

#[derive(Debug, Serialize)]
struct RemoteEventEnvelope {
    #[serde(rename = "session_id")]
    session_id: String,
    #[serde(rename = "occurred_at")]
    occurred_at: String,
    #[serde(rename = "event_type")]
    event_type: String,
    payload: Value,
}

#[derive(Clone)]
struct PersistedRemote {
    config: RemoteServerConfiguration,
    pairing: PairResponse,
}

struct Worker {
    stop: CancellationToken,
    join: JoinHandle<()>,
}

pub struct RemoteController {
    store: Store,
    host: Arc<dyn RemoteHost>,
    persisted: Mutex<Option<PersistedRemote>>,
    status: Arc<Mutex<RemoteServerStatus>>,
    worker: Mutex<Option<Worker>>,
}

impl RemoteController {
    pub fn new(store: Store, host: Arc<dyn RemoteHost>) -> Self {
        let persisted = load_persisted(&store).ok().flatten();
        let configured = store.settings(None, None).ok().is_some_and(|values| {
            values.iter().any(|entry| {
                entry.key == REMOTE_SERVER_URL
                    && entry.value.as_str().is_some_and(|value| !value.is_empty())
            })
        });
        let status = RemoteServerStatus {
            configured,
            connected: false,
            connecting: false,
            host_id: persisted
                .as_ref()
                .map(|value| value.pairing.host_id.clone()),
            mobile_pairing_payload: persisted.as_ref().map(|value| {
                value
                    .pairing
                    .mobile_pairing_url(&value.config.server_url, value.config.e2e_enabled)
            }),
            access_token_expires_at: persisted
                .as_ref()
                .map(|value| value.pairing.access_token_expires_at.clone()),
            mobile_pairing_code: persisted
                .as_ref()
                .map(|value| value.pairing.mobile_pairing_code.clone()),
            mobile_pairing_url: persisted.as_ref().map(|value| {
                value
                    .pairing
                    .mobile_pairing_url(&value.config.server_url, value.config.e2e_enabled)
            }),
            error: None,
        };
        Self {
            store,
            host,
            persisted: Mutex::new(persisted),
            status: Arc::new(Mutex::new(status)),
            worker: Mutex::new(None),
        }
    }

    pub fn configuration(&self) -> RemoteServerConfiguration {
        let values = self.store.settings(None, None).unwrap_or_default();
        let get = |key: &str| {
            values
                .iter()
                .find(|entry| entry.key == key)
                .and_then(|entry| entry.value.as_str())
                .unwrap_or_default()
                .to_string()
        };
        RemoteServerConfiguration {
            server_url: get(REMOTE_SERVER_URL),
            pairing_code: get(REMOTE_PAIRING_CODE),
            e2e_enabled: values
                .iter()
                .find(|entry| entry.key == REMOTE_E2E_ENABLED)
                .and_then(|entry| entry.value.as_bool())
                .unwrap_or(true),
        }
    }

    pub fn status(&self) -> RemoteServerStatus {
        self.status
            .lock()
            .map(|value| value.clone())
            .unwrap_or(RemoteServerStatus {
                configured: false,
                connected: false,
                connecting: false,
                host_id: None,
                mobile_pairing_payload: None,
                access_token_expires_at: None,
                mobile_pairing_code: None,
                mobile_pairing_url: None,
                error: Some("Remote status unavailable".into()),
            })
    }

    pub fn save_configuration(&self, config: RemoteServerConfiguration) -> RemoteResult<()> {
        let config = validate_remote_configuration(config)?;
        self.stop_worker();
        for (key, value) in [
            (REMOTE_SERVER_URL, json!(config.server_url)),
            (REMOTE_PAIRING_CODE, json!(config.pairing_code)),
            (REMOTE_E2E_ENABLED, json!(config.e2e_enabled)),
        ] {
            self.store.set_setting("global", "global", key, &value)?;
        }
        self.store
            .set_setting("global", "global", REMOTE_HOST_ID, &Value::Null)?;
        for key in [
            REMOTE_HOST_ID,
            REMOTE_DESKTOP_TOKEN,
            REMOTE_REFRESH_TOKEN,
            REMOTE_ACCESS_TOKEN_EXPIRES_AT,
            REMOTE_MOBILE_PAIRING_CODE,
            REMOTE_AES_KEY,
        ] {
            self.store
                .set_setting("global", "global", key, &Value::Null)?;
        }
        if let Ok(mut persisted) = self.persisted.lock() {
            *persisted = None;
        }
        self.set_status(RemoteServerStatus {
            configured: true,
            connected: false,
            connecting: false,
            host_id: None,
            mobile_pairing_payload: None,
            access_token_expires_at: None,
            mobile_pairing_code: None,
            mobile_pairing_url: None,
            error: None,
        });
        Ok(())
    }

    pub async fn connect(&self) -> RemoteResult<RemoteServerStatus> {
        self.stop_worker();
        let config = self.configuration();
        let url = normalize_server_url(&config.server_url)?;
        if config.pairing_code.trim().is_empty() {
            return Err(BusinessError::invalid("pairing code is required"));
        }
        self.set_status(RemoteServerStatus {
            configured: true,
            connected: false,
            connecting: true,
            host_id: None,
            mobile_pairing_payload: None,
            access_token_expires_at: None,
            mobile_pairing_code: None,
            mobile_pairing_url: None,
            error: None,
        });
        let client =
            remote_http_client(&self.host.network_configuration(), Duration::from_secs(15))?;
        let endpoint = resolve_endpoint(url.as_str(), "/v1/desktop/pairings");
        let pair_body = PairRequest {
            pairing_code: config.pairing_code.clone(),
            display_name: hostname(),
            agent_version: env!("CARGO_PKG_VERSION").to_string(),
        };
        log_http_request("POST", &endpoint, &pair_body);
        let response = client
            .post(endpoint)
            .json(&pair_body)
            .send()
            .await
            .map_err(remote_http_error)?;
        let mut pairing = decode_pair_response(response).await?;
        if pairing.host_id.trim().is_empty()
            || pairing.access_token.trim().is_empty()
            || pairing.refresh_token.trim().is_empty()
            || pairing.access_token_expires_at.trim().is_empty()
            || pairing.mobile_pairing_code.trim().is_empty()
        {
            return Err(BusinessError::new(
                "remote_pairing_invalid",
                "Remote Server returned incomplete pairing data",
            ));
        }
        for (key, value) in [
            (REMOTE_HOST_ID, json!(pairing.host_id)),
            (REMOTE_DESKTOP_TOKEN, json!(pairing.access_token)),
            (REMOTE_REFRESH_TOKEN, json!(pairing.refresh_token)),
            (
                REMOTE_ACCESS_TOKEN_EXPIRES_AT,
                json!(pairing.access_token_expires_at),
            ),
            (
                REMOTE_MOBILE_PAIRING_CODE,
                json!(pairing.mobile_pairing_code),
            ),
        ] {
            self.store.set_setting("global", "global", key, &value)?;
        }
        pairing.aes_key = generate_aes_key();
        self.store
            .set_setting("global", "global", REMOTE_AES_KEY, &json!(pairing.aes_key))?;
        let persisted = PersistedRemote { config, pairing };
        if let Ok(mut current) = self.persisted.lock() {
            *current = Some(persisted.clone());
        }
        self.start_worker(persisted)?;
        Ok(self.status())
    }

    pub fn disconnect(&self) -> RemoteResult<RemoteServerStatus> {
        self.stop_worker();
        self.set_status(RemoteServerStatus {
            configured: self
                .persisted
                .lock()
                .ok()
                .is_some_and(|value| value.is_some()),
            connected: false,
            connecting: false,
            host_id: self.status().host_id,
            mobile_pairing_payload: self.status().mobile_pairing_payload,
            access_token_expires_at: self.status().access_token_expires_at,
            mobile_pairing_code: self.status().mobile_pairing_code,
            mobile_pairing_url: self.status().mobile_pairing_url,
            error: None,
        });
        Ok(self.status())
    }

    pub fn clear(&self) -> RemoteResult<RemoteServerStatus> {
        self.stop_worker();
        for key in [
            REMOTE_SERVER_URL,
            REMOTE_PAIRING_CODE,
            REMOTE_HOST_ID,
            REMOTE_DESKTOP_TOKEN,
            REMOTE_REFRESH_TOKEN,
            REMOTE_ACCESS_TOKEN_EXPIRES_AT,
            REMOTE_MOBILE_PAIRING_CODE,
            REMOTE_AES_KEY,
            REMOTE_E2E_ENABLED,
        ] {
            self.store
                .set_setting("global", "global", key, &Value::Null)?;
        }
        if let Ok(mut persisted) = self.persisted.lock() {
            *persisted = None;
        }
        let status = RemoteServerStatus {
            configured: false,
            connected: false,
            connecting: false,
            host_id: None,
            mobile_pairing_payload: None,
            access_token_expires_at: None,
            mobile_pairing_code: None,
            mobile_pairing_url: None,
            error: None,
        };
        self.set_status(status.clone());
        Ok(status)
    }

    pub fn start_saved(&self) -> RemoteResult<()> {
        let Some(persisted) = self.persisted.lock().ok().and_then(|value| value.clone()) else {
            return Ok(());
        };
        self.start_worker(persisted)
    }

    fn start_worker(&self, persisted: PersistedRemote) -> RemoteResult<()> {
        let stop = CancellationToken::new();
        let thread_stop = stop.clone();
        let status = self.status.clone();
        let config = persisted.clone();
        let host = self.host.clone();
        self.set_status(RemoteServerStatus {
            configured: true,
            connected: false,
            connecting: true,
            host_id: Some(persisted.pairing.host_id.clone()),
            mobile_pairing_payload: Some(
                persisted
                    .pairing
                    .mobile_pairing_url(&persisted.config.server_url, persisted.config.e2e_enabled),
            ),
            access_token_expires_at: Some(persisted.pairing.access_token_expires_at.clone()),
            mobile_pairing_code: Some(persisted.pairing.mobile_pairing_code.clone()),
            mobile_pairing_url: Some(
                persisted
                    .pairing
                    .mobile_pairing_url(&persisted.config.server_url, persisted.config.e2e_enabled),
            ),
            error: None,
        });
        let join = std::thread::Builder::new()
            .name("suncode-remote".into())
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build();
                match runtime {
                    Ok(runtime) => {
                        runtime.block_on(remote_worker(host, config, thread_stop, status))
                    }
                    Err(error) => {
                        logging::error("remote", &format!("worker runtime failed: {error}"))
                    }
                }
            })
            .map_err(|error| {
                BusinessError::unavailable(format!("Remote worker could not start: {error}"))
            })?;
        let mut slot = self
            .worker
            .lock()
            .map_err(|_| BusinessError::unavailable("Remote worker state unavailable"))?;
        *slot = Some(Worker { stop, join });
        Ok(())
    }

    fn stop_worker(&self) {
        if let Ok(mut worker) = self.worker.lock() {
            if let Some(worker) = worker.take() {
                worker.stop.cancel();
                if worker.join.thread().id() != std::thread::current().id() {
                    let _ = worker.join.join();
                }
            }
        }
    }

    fn set_status(&self, value: RemoteServerStatus) {
        if let Ok(mut status) = self.status.lock() {
            *status = value;
        }
    }

    pub fn set_status_for_error(&self, error: String) {
        let mut status = self.status();
        status.connected = false;
        status.connecting = false;
        status.error = Some(error);
        self.set_status(status);
    }
}

fn validate_remote_configuration(
    mut config: RemoteServerConfiguration,
) -> RemoteResult<RemoteServerConfiguration> {
    config.server_url = normalize_server_url(&config.server_url)?.to_string();
    config.pairing_code = config.pairing_code.trim().to_string();
    if config.pairing_code.is_empty() || config.pairing_code.len() > 256 {
        return Err(BusinessError::invalid("pairing code is required"));
    }
    Ok(config)
}

fn normalize_server_url(value: &str) -> RemoteResult<Url> {
    let url = Url::parse(value.trim())
        .map_err(|_| BusinessError::invalid("Remote Server URL is invalid"))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(BusinessError::invalid(
            "Remote Server URL must be an HTTP(S) origin without embedded credentials",
        ));
    }
    Ok(url)
}

pub fn should_upload_event(event_type: &str) -> bool {
    UPLOAD_EVENT_TYPES.contains(&event_type)
}

fn hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "SunCode Desktop".into())
}

fn load_persisted(store: &Store) -> RemoteResult<Option<PersistedRemote>> {
    let values = store.settings(None, None)?;
    let get = |key: &str| {
        values
            .iter()
            .find(|entry| entry.key == key)
            .and_then(|entry| entry.value.as_str())
            .map(ToOwned::to_owned)
    };
    let (Some(server_url), Some(pairing_code)) = (get(REMOTE_SERVER_URL), get(REMOTE_PAIRING_CODE))
    else {
        return Ok(None);
    };
    let Some(host_id) = get(REMOTE_HOST_ID) else {
        return Ok(None);
    };
    let Some(desktop_token) = get(REMOTE_DESKTOP_TOKEN) else {
        return Ok(None);
    };
    let Some(refresh_token) = get(REMOTE_REFRESH_TOKEN) else {
        return Ok(None);
    };
    let Some(access_token_expires_at) = get(REMOTE_ACCESS_TOKEN_EXPIRES_AT) else {
        return Ok(None);
    };
    let Some(mobile_pairing_code) = get(REMOTE_MOBILE_PAIRING_CODE) else {
        return Ok(None);
    };
    let Some(aes_key) = get(REMOTE_AES_KEY) else {
        return Ok(None);
    };
    Ok(Some(PersistedRemote {
        config: RemoteServerConfiguration {
            server_url,
            pairing_code,
            e2e_enabled: values
                .iter()
                .find(|entry| entry.key == REMOTE_E2E_ENABLED)
                .and_then(|entry| entry.value.as_bool())
                .unwrap_or(true),
        },
        pairing: PairResponse {
            host_id,
            access_token: desktop_token,
            refresh_token,
            access_token_expires_at,
            mobile_pairing_code,
            aes_key,
        },
    }))
}

async fn decode_pair_response(response: Response) -> RemoteResult<PairResponse> {
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

fn remote_http_error(error: reqwest::Error) -> BusinessError {
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

fn remote_http_client(
    configuration: &RemoteNetworkConfiguration,
    timeout: Duration,
) -> RemoteResult<Client> {
    let verify_certificates = configuration.verify_https_certificates;
    let use_system_certificates = configuration.use_system_certificates;
    let mut builder = Client::builder()
        .timeout(timeout)
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

async fn remote_worker(
    host: Arc<dyn RemoteHost>,
    persisted: PersistedRemote,
    stop: CancellationToken,
    status: Arc<Mutex<RemoteServerStatus>>,
) {
    let client = match remote_http_client(&host.network_configuration(), Duration::from_secs(45)) {
        Ok(client) => client,
        Err(error) => {
            set_worker_status(&status, false, Some(error.message));
            return;
        }
    };
    let host_id = persisted.pairing.host_id.clone();
    let mut subscribed = HashSet::new();
    let mut delay = Duration::from_secs(1);
    let mut snapshot_tick = tokio::time::interval(Duration::from_secs(30));
    snapshot_tick.tick().await;
    loop {
        if stop.is_cancelled() {
            break;
        }
        set_worker_status(&status, false, None);
        let request_url = resolve_endpoint(&persisted.config.server_url, "/v1/desktop/events");
        log_http_request("GET", &request_url, &json!({"accept": "text/event-stream"}));
        let response = request_headers(client.get(request_url), &persisted, &host_id)
            .header("Accept", "text/event-stream")
            .send()
            .await;
        match response {
            Ok(response) if response.status().is_success() => {
                set_worker_status(&status, true, None);
                delay = Duration::from_secs(1);
                if let Err(error) =
                    upload_desktop_snapshot(&host, &client, &persisted, &host_id).await
                {
                    logging::write_business_error(
                        "remote",
                        "snapshot",
                        &error,
                        "phase=initial-sync",
                    );
                }
                subscribe_existing_sessions(
                    &host,
                    &client,
                    &persisted,
                    &host_id,
                    &stop,
                    &mut subscribed,
                );
                let mut stream = response.bytes_stream();
                let mut buffer = Vec::new();
                loop {
                    tokio::select! {
                        _ = stop.cancelled() => return,
                        _ = snapshot_tick.tick() => {
                            if let Err(error) = upload_desktop_snapshot(&host, &client, &persisted, &host_id).await {
                                logging::write_business_error("remote", "snapshot", &error, "phase=periodic-sync");
                            }
                            subscribe_existing_sessions(&host, &client, &persisted, &host_id, &stop, &mut subscribed);
                        }
                        item = stream.next() => match item {
                            Some(Ok(bytes)) => {
                                buffer.extend_from_slice(&bytes);
                                while let Some((boundary, delimiter_len)) = sse_frame_boundary(&buffer) {
                                    let frame = buffer.drain(..boundary + delimiter_len).collect::<Vec<_>>();
                                    log_sse_frame(&frame);
                                    if is_sse_comment(&frame) {
                                        logging::debug("remote", "SSE comment received (heartbeat)");
                                        continue;
                                    }
                                    if let Some(request) = parse_sse_request(&frame, &persisted.pairing.aes_key) {
                                        logging::debug(
                                            "remote",
                                            format!(
                                                "SSE event accepted request_id={} operation={} arguments={}",
                                                request.request_id,
                                                request.operation,
                                                redact_remote_value(&request.arguments)
                                            ),
                                        );
                                        handle_remote_request(&host, &client, &persisted, &host_id, request).await;
                                    } else {
                                        logging::debug("remote", "SSE event ignored or could not be parsed");
                                    }
                                }
                            }
                            Some(Err(error)) => { set_worker_status(&status, false, Some(error.to_string())); break; }
                            None => { set_worker_status(&status, false, Some("Remote event stream closed".into())); break; }
                        }
                    }
                }
            }
            Ok(response) => set_worker_status(
                &status,
                false,
                Some(format!(
                    "Remote event stream returned HTTP {}",
                    response.status()
                )),
            ),
            Err(error) => set_worker_status(&status, false, Some(error.to_string())),
        }
        tokio::select! { _ = stop.cancelled() => break, _ = tokio::time::sleep(delay) => {} }
        delay = (delay * 2).min(Duration::from_secs(30));
    }
    set_worker_status(&status, false, None);
}

async fn upload_desktop_snapshot(
    host: &Arc<dyn RemoteHost>,
    client: &Client,
    persisted: &PersistedRemote,
    host_id: &str,
) -> RemoteResult<()> {
    let payload = host.snapshot()?;
    let request_id = uuid::Uuid::new_v4().to_string();
    let url = resolve_endpoint(&persisted.config.server_url, "/v1/desktop/snapshot");
    let body = remote_request_body(&payload, persisted)?;
    log_http_request("POST", &url, &body);
    let response = request_headers(client.post(url), persisted, host_id)
        .header("X-Request-Id", request_id)
        .json(&body)
        .send()
        .await
        .map_err(remote_http_error)?;
    if !response.status().is_success() {
        return Err(BusinessError::unavailable(format!(
            "Remote snapshot upload returned HTTP {}",
            response.status()
        )));
    }
    Ok(())
}

fn subscribe_existing_sessions(
    host: &Arc<dyn RemoteHost>,
    client: &Client,
    persisted: &PersistedRemote,
    host_id: &str,
    stop: &CancellationToken,
    subscribed: &mut HashSet<String>,
) {
    let Ok(snapshot) = host.snapshot() else {
        return;
    };
    let sessions = snapshot
        .get("sessions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for session in sessions {
        let Some(session_id) = session.get("sessionId").and_then(Value::as_str) else {
            continue;
        };
        let session_id = session_id.to_owned();
        if !subscribed.insert(session_id.clone()) {
            continue;
        }
        let Ok(mut stream) = host.subscribe_session_events(&session_id) else {
            continue;
        };
        let client = client.clone();
        let persisted = persisted.clone();
        let host_id = host_id.to_string();
        let stop = stop.clone();
        tokio::spawn(async move {
            let mut pending_delta = RemoteAssistantDeltaBuffer::default();
            let mut flush_tick = tokio::time::interval(REMOTE_DELTA_FLUSH_INTERVAL);
            flush_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            flush_tick.tick().await;
            loop {
                tokio::select! {
                    _ = stop.cancelled() => {
                        flush_remote_assistant_delta(
                            &mut pending_delta,
                            &client,
                            &persisted,
                            &host_id,
                            &session_id,
                        ).await;
                        break;
                    },
                    _ = flush_tick.tick() => {
                        flush_remote_assistant_delta(
                            &mut pending_delta,
                            &client,
                            &persisted,
                            &host_id,
                            &session_id,
                        ).await;
                    }
                    event = stream.recv() => match event {
                        Some(Ok(event)) => {
                            let event_type = event.event_type.as_str();
                            if event_type == "assistant.delta" {
                                let payload = event.payload.clone();
                                let occurred_at = event.occurred_at.clone();
                                for chunk in pending_delta.push(payload, &occurred_at) {
                                    upload_remote_session_event(
                                        &client,
                                        &persisted,
                                        &host_id,
                                        &session_id,
                                        "assistant.delta",
                                        pending_delta.occurred_at(),
                                        chunk,
                                    ).await;
                                }
                                continue;
                            }
                            flush_remote_assistant_delta(
                                &mut pending_delta,
                                &client,
                                &persisted,
                                &host_id,
                                &session_id,
                            ).await;
                            if !should_upload_event(event_type) { continue; }
                            upload_remote_session_event(
                                &client,
                                &persisted,
                                &host_id,
                                &event.session_id,
                                event_type,
                                &event.occurred_at,
                                event.payload.clone(),
                            ).await;
                        }
                        Some(Err(error)) => {
                            flush_remote_assistant_delta(
                                &mut pending_delta,
                                &client,
                                &persisted,
                                &host_id,
                                &session_id,
                            ).await;
                            logging::warn("remote", &format!("session event stream ended: {error}"));
                            break;
                        }
                        None => {
                            flush_remote_assistant_delta(
                                &mut pending_delta,
                                &client,
                                &persisted,
                                &host_id,
                                &session_id,
                            ).await;
                            break;
                        },
                    }
                }
            }
        });
    }
}

#[derive(Default)]
struct RemoteAssistantDeltaBuffer {
    turn_id: Option<String>,
    text: String,
    occurred_at: String,
}

impl RemoteAssistantDeltaBuffer {
    fn push(&mut self, payload: Value, occurred_at: &str) -> Vec<Value> {
        let text = payload
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if text.is_empty() {
            return Vec::new();
        }
        let turn_id = payload
            .get("turn_id")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let mut ready = Vec::new();
        if self.turn_id.is_some() && self.turn_id != turn_id {
            if let Some(payload) = self.flush() {
                ready.push(payload);
            }
        }
        self.turn_id = turn_id;
        self.text.push_str(text);
        self.occurred_at = occurred_at.to_owned();

        while let Some(end) = self.next_chunk_end() {
            ready.push(self.take_chunk(end));
        }
        ready
    }

    fn flush(&mut self) -> Option<Value> {
        if self.text.is_empty() {
            return None;
        }
        let text = std::mem::take(&mut self.text);
        Some(json!({"turn_id": self.turn_id, "text": text}))
    }

    fn occurred_at(&self) -> &str {
        &self.occurred_at
    }

    fn next_chunk_end(&self) -> Option<usize> {
        let max_end = self
            .text
            .char_indices()
            .nth(REMOTE_DELTA_MAX_CHARS)
            .map(|(index, _)| index)
            .or_else(|| {
                (self.text.chars().count() >= REMOTE_DELTA_MAX_CHARS).then_some(self.text.len())
            });
        if let Some(end) = paragraph_end(&self.text) {
            if max_end.is_none_or(|limit| end <= limit) {
                return Some(end);
            }
        }
        max_end
    }

    fn take_chunk(&mut self, end: usize) -> Value {
        let remaining = self.text.split_off(end);
        let text = std::mem::replace(&mut self.text, remaining);
        json!({"turn_id": self.turn_id, "text": text})
    }
}

fn paragraph_end(text: &str) -> Option<usize> {
    let lf = text.find("\n\n").map(|index| index + 2);
    let crlf = text.find("\r\n\r\n").map(|index| index + 4);
    match (lf, crlf) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(end), None) | (None, Some(end)) => Some(end),
        (None, None) => None,
    }
}

async fn flush_remote_assistant_delta(
    buffer: &mut RemoteAssistantDeltaBuffer,
    client: &Client,
    persisted: &PersistedRemote,
    host_id: &str,
    session_id: &str,
) {
    if let Some(payload) = buffer.flush() {
        let occurred_at = buffer.occurred_at().to_owned();
        upload_remote_session_event(
            client,
            persisted,
            host_id,
            session_id,
            "assistant.delta",
            &occurred_at,
            payload,
        )
        .await;
    }
}

async fn upload_remote_session_event(
    client: &Client,
    persisted: &PersistedRemote,
    host_id: &str,
    session_id: &str,
    event_type: &str,
    occurred_at: &str,
    payload: Value,
) {
    let event_id = uuid::Uuid::new_v4().to_string();
    let event = RemoteEventEnvelope {
        session_id: session_id.to_owned(),
        occurred_at: occurred_at.to_owned(),
        event_type: event_type.to_owned(),
        payload,
    };
    let url = resolve_endpoint(&persisted.config.server_url, "/v1/desktop/events");
    let body = match remote_request_body(
        &serde_json::to_value(&event).unwrap_or_else(|_| json!({})),
        persisted,
    ) {
        Ok(body) => body,
        Err(error) => {
            logging::warn(
                "remote",
                &format!("event encryption failed: {}", error.message),
            );
            return;
        }
    };
    log_http_request("POST", &url, &body);
    match request_headers(client.post(url), persisted, host_id)
        .header("X-Request-Id", &event_id)
        .json(&body)
        .send()
        .await
    {
        Ok(response) if !response.status().is_success() => logging::warn(
            "remote",
            format!("event upload returned HTTP {}", response.status()),
        ),
        Err(error) => logging::warn("remote", format!("event upload failed: {error}")),
        _ => {}
    }
}

fn set_worker_status(status: &Mutex<RemoteServerStatus>, connected: bool, error: Option<String>) {
    if let Ok(mut value) = status.lock() {
        value.connected = connected;
        value.connecting = !connected && error.is_none();
        value.error = error;
    }
}

fn resolve_endpoint(base: &str, path: &str) -> String {
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

fn request_headers(
    request: reqwest::RequestBuilder,
    persisted: &PersistedRemote,
    host_id: &str,
) -> reqwest::RequestBuilder {
    request
        .header("X-Host-Id", host_id)
        .bearer_auth(&persisted.pairing.access_token)
}

fn redact_remote_value(value: &Value) -> String {
    let mut value = value.clone();
    redact_remote_object(&mut value);
    serde_json::to_string(&value).unwrap_or_else(|_| "<invalid-json>".into())
}

fn redact_remote_object(value: &mut Value) {
    match value {
        Value::Object(object) => {
            for (key, value) in object.iter_mut() {
                if matches!(
                    key.as_str(),
                    "encPayload"
                        | "accessToken"
                        | "refreshToken"
                        | "apiKey"
                        | "password"
                        | "pairingCode"
                        | "pairing_code"
                        | "code"
                        | "k"
                ) {
                    *value = Value::String("<redacted>".into());
                } else {
                    redact_remote_object(value);
                }
            }
        }
        Value::Array(values) => values.iter_mut().for_each(redact_remote_object),
        _ => {}
    }
}

fn log_http_request<T: Serialize>(method: &str, url: &str, body: &T) {
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

fn log_sse_frame(frame: &[u8]) {
    let text = String::from_utf8_lossy(frame);
    let rendered = text
        .lines()
        .map(|line| {
            line.strip_prefix("data:")
                .and_then(|data| serde_json::from_str::<Value>(data.trim()).ok())
                .map(|value| format!("data: {}", redact_remote_value(&value)))
                .unwrap_or_else(|| line.to_owned())
        })
        .collect::<Vec<_>>()
        .join("\\n");
    logging::debug(
        "remote",
        format!("SSE event received frame={}", rendered.trim()),
    );
}

fn is_sse_comment(frame: &[u8]) -> bool {
    frame
        .split(|byte| *byte == b'\n' || *byte == b'\r')
        .map(|line| line.strip_prefix(b" ").unwrap_or(line))
        .filter(|line| !line.is_empty())
        .all(|line| line.starts_with(b":"))
}

fn parse_sse_request(frame: &[u8], aes_key: &str) -> Option<RemoteRequest> {
    let text = std::str::from_utf8(frame).ok()?;
    let mut event = String::new();
    let mut id = String::new();
    let mut data_lines = Vec::new();
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("event:") {
            event = value.trim().into();
        }
        if let Some(value) = line.strip_prefix("id:") {
            id = value.trim().into();
        }
        if let Some(value) = line.strip_prefix("data:") {
            data_lines.push(value.strip_prefix(' ').unwrap_or(value).to_owned());
        }
    }
    if event.starts_with("mobile.") {
        if data_lines.len() != 2 || id.trim().is_empty() {
            return None;
        }
        let routing: Value = serde_json::from_str(&data_lines[0]).ok()?;
        let path = routing.get("pathParam")?.as_object()?;
        let query = routing.get("queryParam")?.as_object()?;
        let body: Value = serde_json::from_str(&data_lines[1]).ok()?;
        let operation = mobile_event_operation(&event)?;
        let encrypted_payload = body
            .get("encPayload")
            .and_then(Value::as_str)
            .filter(|payload| !payload.trim().is_empty());
        let mut arguments = if let Some(enc_payload) = encrypted_payload {
            decrypt_remote_payload(enc_payload, aes_key).ok()?
        } else {
            body
        };
        if !arguments.is_object() {
            arguments = json!({});
        }
        if let Some(object) = arguments.as_object_mut() {
            for (key, value) in path {
                object.entry(key.clone()).or_insert_with(|| value.clone());
            }
        }
        if let Some(object) = arguments.as_object_mut() {
            for (key, value) in query {
                object.entry(key.clone()).or_insert_with(|| value.clone());
            }
        }
        return Some(RemoteRequest {
            request_id: id,
            operation: operation.into(),
            arguments,
        });
    }
    if event != "desktop.request" && event != "desktop.command" {
        return None;
    }
    let mut value: Value = serde_json::from_str(&data_lines.join("\n")).ok()?;
    if value.get("request_id").is_none() && value.get("requestId").is_none() {
        value["request_id"] = Value::String(id);
    }
    serde_json::from_value(value)
        .ok()
        .filter(|request: &RemoteRequest| !request.request_id.trim().is_empty())
}

fn decrypt_remote_payload(payload: &str, key_text: &str) -> RemoteResult<Value> {
    let encoded = payload
        .strip_prefix("e2e-v1:")
        .ok_or_else(|| BusinessError::invalid("unsupported encrypted payload version"))?;
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| BusinessError::invalid("encrypted payload is not valid base64url"))?;
    if bytes.len() < 12 + 16 {
        return Err(BusinessError::invalid("encrypted payload is truncated"));
    }
    let key = URL_SAFE_NO_PAD
        .decode(key_text)
        .map_err(|_| BusinessError::invalid("remote encryption key is not valid base64url"))?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| BusinessError::invalid("remote encryption key is invalid"))?;
    let nonce = Nonce::from_slice(&bytes[..12]);
    let plaintext = cipher
        .decrypt(nonce, &bytes[12..])
        .map_err(|_| BusinessError::invalid("encrypted payload authentication failed"))?;
    serde_json::from_slice(&plaintext)
        .map_err(|_| BusinessError::invalid("encrypted payload JSON is invalid"))
}

fn encrypt_remote_payload(value: &Value, key_text: &str) -> RemoteResult<String> {
    let key = URL_SAFE_NO_PAD
        .decode(key_text)
        .map_err(|_| BusinessError::invalid("remote encryption key is not valid base64url"))?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| BusinessError::invalid("remote encryption key is invalid"))?;
    let nonce_source = uuid::Uuid::new_v4();
    let nonce_bytes = &nonce_source.as_bytes()[..12];
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = serde_json::to_vec(value)
        .map_err(|_| BusinessError::invalid("remote payload JSON is invalid"))?;
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|_| BusinessError::unavailable("remote payload encryption failed"))?;
    let mut bytes = nonce_bytes.to_vec();
    bytes.extend_from_slice(&ciphertext);
    Ok(format!("e2e-v1:{}", URL_SAFE_NO_PAD.encode(bytes)))
}

fn remote_request_body(value: &Value, persisted: &PersistedRemote) -> RemoteResult<Value> {
    if !persisted.config.e2e_enabled {
        return Ok(value.clone());
    }
    Ok(json!({ "encPayload": encrypt_remote_payload(value, &persisted.pairing.aes_key)? }))
}

/// Builds the `/v1/desktop/responses` body: the plain payload (or its
/// `encPayload` wrapper) on success, and an unencrypted `{code, message}` on
/// failure so the Server can map it to an HTTP error for Mobile.
fn remote_response_body(
    outcome: RemoteResult<Value>,
    persisted: &PersistedRemote,
) -> RemoteResult<Value> {
    match outcome {
        Ok(data) => remote_request_body(&data, persisted),
        Err(error) => serde_json::to_value(RemoteErrorBody {
            code: REMOTE_DESKTOP_ERROR_CODE,
            message: error.message,
        })
        .map_err(|e| BusinessError::unavailable(e.to_string())),
    }
}

fn mobile_event_operation(event: &str) -> Option<&'static str> {
    Some(match event {
        "mobile.projects.list" => "projects.list",
        "mobile.sessions.list" => "sessions.list",
        "mobile.sessions.create" => "session.create",
        "mobile.sessions.get" => "session.get",
        "mobile.sessions.messages.send" => "session.message",
        "mobile.sessions.approvals.resolve" => "approval.resolve",
        "mobile.sessions.questions.reply" => "question.reply",
        "mobile.sessions.cancel" => "turn.cancel",
        "mobile.sessions.retry" => "turn.retry",
        _ => return None,
    })
}

fn sse_frame_boundary(buffer: &[u8]) -> Option<(usize, usize)> {
    if let Some(position) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
        return Some((position, 4));
    }
    buffer
        .windows(2)
        .position(|window| window == b"\n\n")
        .map(|position| (position, 2))
}

async fn handle_remote_request(
    host: &Arc<dyn RemoteHost>,
    client: &Client,
    persisted: &PersistedRemote,
    host_id: &str,
    request: RemoteRequest,
) {
    let request_id = request.request_id;
    logging::debug(
        "remote",
        format!(
            "dispatching remote request request_id={request_id} operation={}",
            request.operation
        ),
    );
    let outcome = host
        .dispatch(&request_id, &request.operation, request.arguments)
        .await;
    let url = resolve_endpoint(&persisted.config.server_url, "/v1/desktop/responses");
    let body = match remote_response_body(outcome, persisted) {
        Ok(body) => body,
        Err(error) => {
            logging::warn(
                "remote",
                &format!("response encryption failed: {}", error.message),
            );
            return;
        }
    };
    log_http_request("POST", &url, &body);
    match request_headers(client.post(url), persisted, host_id)
        .header("X-Request-Id", &request_id)
        .json(&body)
        .send()
        .await
    {
        Ok(response) => logging::debug(
            "remote",
            format!(
                "remote response uploaded request_id={request_id} status={}",
                response.status()
            ),
        ),
        Err(error) => logging::warn(
            "remote",
            format!("remote response upload failed request_id={request_id} error={error}"),
        ),
    }
}

/*
    let string = |snake: &str, camel: &str| {
        argument_value(&arguments, snake, camel)
            .and_then(Value::as_str)
            .ok_or_else(|| BusinessError::invalid(format!("{camel} is required")))
    };
    match operation {
        "projects.list" => serde_json::to_value(sdk.list_projects()?)
            .map_err(|e| BusinessError::unavailable(e.to_string())),
        "sessions.list" => {
            let project_id = string("project_id", "projectId")?;
            let result = if project_id.trim().is_empty() {
                let mut sessions = Vec::new();
                let mut session_states = std::collections::HashMap::new();
                for project in sdk.list_projects()?.projects {
                    let result = sdk.list_sessions(&project.project_id)?;
                    sessions.extend(result.sessions);
                    session_states.extend(result.session_states);
                }
                SessionsResult {
                    project_id: String::new(),
                    sessions,
                    session_states,
                }
            } else {
                sdk.list_sessions(project_id)?
            };
            serde_json::to_value(result).map_err(|e| BusinessError::unavailable(e.to_string()))
        }
        "session.create" => {
            let project_id = string("project_id", "projectId")?;
            let title = argument_value(&arguments, "title", "title").and_then(Value::as_str);
            serde_json::to_value(sdk.create_session(project_id, title, None)?)
                .map_err(|e| BusinessError::unavailable(e.to_string()))
        }
        "session.get" => {
            let session_id = string("session_id", "sessionId")?;
            serde_json::to_value(sdk.session_snapshot(session_id, 0)?)
                .map_err(|e| BusinessError::unavailable(e.to_string()))
        }
        "session.send_message" | "session.message" => {
            let session_id = string("session_id", "sessionId")?;
            let text = string("text", "text")?;
            let model = arguments.get("model").and_then(Value::as_str);
            let effort = arguments.get("reasoning_effort").and_then(Value::as_str);
            serde_json::to_value(
                sdk.submit_turn(session_id, text, request_id, model, effort)
                    .await?,
            )
            .map_err(|e| BusinessError::unavailable(e.to_string()))
        }
        "session.cancel" | "turn.cancel" => {
            let session_id = string("session_id", "sessionId")?;
            let turn_id = arguments
                .get("turn_id")
                .and_then(Value::as_str)
                .or_else(|| arguments.get("turnId").and_then(Value::as_str));
            let result = if let Some(turn_id) = turn_id {
                sdk.cancel_turn(session_id, turn_id)?
            } else {
                let snapshot = sdk.session_snapshot(session_id, 0)?;
                let turn_id = snapshot
                    .conversation_turns
                    .last()
                    .map(|turn| turn.turn_id.as_str())
                    .ok_or_else(|| BusinessError::invalid("turnId is required"))?;
                sdk.cancel_turn(session_id, turn_id)?
            };
            serde_json::to_value(result).map_err(|e| BusinessError::unavailable(e.to_string()))
        }
        "session.retry" | "turn.retry" => {
            let session_id = string("session_id", "sessionId")?;
            serde_json::to_value(sdk.retry_last_turn(session_id).await?)
                .map_err(|e| BusinessError::unavailable(e.to_string()))
        }
        "approval.resolve" => {
            let approval_id = string("approval_id", "approvalId")?;
            let decision = string("decision", "action")?;
            serde_json::to_value(sdk.resolve_approval(approval_id, decision).await?)
                .map_err(|e| BusinessError::unavailable(e.to_string()))
        }
        "question.reply" => {
            let question_id = string("question_id", "questionId")?;
            let answers = arguments
                .get("answers")
                .cloned()
                .ok_or_else(|| BusinessError::invalid("answers are required"))?;
            let answers = normalize_question_answers(answers)?;
            serde_json::to_value(sdk.reply_question(question_id, &answers).await?)
                .map_err(|e| BusinessError::unavailable(e.to_string()))
        }
        _ => Err(BusinessError::invalid("Remote operation is not supported")),
    }
}*/

#[cfg(test)]
fn normalize_question_answers(answers: Value) -> RemoteResult<Value> {
    let Some(values) = answers.as_array() else {
        return Err(BusinessError::invalid("answers must be an array"));
    };
    if values.iter().all(Value::is_string) {
        return Ok(Value::Array(
            values
                .iter()
                .map(|value| Value::Array(vec![value.clone()]))
                .collect(),
        ));
    }
    Ok(answers)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn persisted_fixture(e2e_enabled: bool) -> PersistedRemote {
        PersistedRemote {
            config: RemoteServerConfiguration {
                server_url: "https://example.test".into(),
                pairing_code: "pair".into(),
                e2e_enabled,
            },
            pairing: PairResponse {
                host_id: "host-1".into(),
                access_token: "access".into(),
                refresh_token: "refresh".into(),
                access_token_expires_at: "2026-10-04T00:00:00Z".into(),
                mobile_pairing_code: "once".into(),
                aes_key: generate_aes_key(),
            },
        }
    }

    #[test]
    fn response_body_posts_plain_success_payload_without_code_envelope() {
        let body = remote_response_body(
            Ok(json!({"projects": [{"projectId": "p-1"}]})),
            &persisted_fixture(false),
        )
        .unwrap();
        assert_eq!(body, json!({"projects": [{"projectId": "p-1"}]}));
        assert!(body.get("code").is_none());
        assert!(body.get("payload").is_none());
    }

    #[test]
    fn response_body_encrypts_only_success_payloads() {
        let persisted = persisted_fixture(true);
        let body = remote_response_body(Ok(json!({"ok": true})), &persisted).unwrap();
        assert_eq!(body.as_object().unwrap().len(), 1);
        assert!(body["encPayload"].is_string());

        let error = remote_response_body(
            Err(BusinessError::invalid("projectId is required")),
            &persisted,
        )
        .unwrap();
        assert_eq!(
            error,
            json!({"code": REMOTE_DESKTOP_ERROR_CODE, "message": "projectId is required"})
        );
    }

    #[test]
    fn allowlist_uploads_complete_assistant_and_coalesced_delta_events() {
        assert!(should_upload_event("message.assistant"));
        assert!(should_upload_event("message.user"));
        assert!(should_upload_event("assistant.delta"));
        assert!(!should_upload_event("provider.exchange.progress"));
        assert!(!should_upload_event("unknown.event"));
    }

    #[test]
    fn remote_delta_buffer_combines_fragments_until_flush() {
        let mut buffer = RemoteAssistantDeltaBuffer::default();
        assert!(buffer
            .push(json!({"turn_id": "turn-1", "text": "Hello"}), "t1")
            .is_empty());
        assert!(buffer
            .push(json!({"turn_id": "turn-1", "text": " world"}), "t2")
            .is_empty());
        assert_eq!(
            buffer.flush(),
            Some(json!({"turn_id": "turn-1", "text": "Hello world"}))
        );
        assert_eq!(buffer.occurred_at(), "t2");
    }

    #[test]
    fn remote_delta_buffer_splits_at_unicode_character_limit() {
        let mut buffer = RemoteAssistantDeltaBuffer::default();
        let text = "你".repeat(REMOTE_DELTA_MAX_CHARS + 2);
        let chunks = buffer.push(json!({"turn_id": "turn-1", "text": text}), "t1");
        assert_eq!(chunks.len(), 1);
        assert_eq!(
            chunks[0]["text"].as_str().unwrap().chars().count(),
            REMOTE_DELTA_MAX_CHARS
        );
        assert_eq!(
            buffer.flush().unwrap()["text"]
                .as_str()
                .unwrap()
                .chars()
                .count(),
            2
        );
    }

    #[test]
    fn remote_delta_buffer_caps_long_paragraphs_at_character_limit() {
        let mut buffer = RemoteAssistantDeltaBuffer::default();
        let text = format!(
            "{}\n\nLater paragraph",
            "x".repeat(REMOTE_DELTA_MAX_CHARS + 20)
        );
        let chunks = buffer.push(json!({"turn_id": "turn-1", "text": text}), "t1");
        assert!(chunks.len() >= 2);
        assert!(
            chunks
                .iter()
                .all(|chunk| chunk["text"].as_str().unwrap().chars().count()
                    <= REMOTE_DELTA_MAX_CHARS)
        );
    }

    #[test]
    fn remote_delta_buffer_prefers_paragraph_boundary_and_flushes_on_turn_change() {
        let mut buffer = RemoteAssistantDeltaBuffer::default();
        let chunks = buffer.push(
            json!({"turn_id": "turn-1", "text": "First paragraph.\n\nSecond"}),
            "t1",
        );
        assert_eq!(
            chunks,
            vec![json!({"turn_id": "turn-1", "text": "First paragraph.\n\n"})]
        );
        let chunks = buffer.push(json!({"turn_id": "turn-2", "text": "Next turn"}), "t2");
        assert_eq!(chunks, vec![json!({"turn_id": "turn-1", "text": "Second"})]);
        assert_eq!(
            buffer.flush(),
            Some(json!({"turn_id": "turn-2", "text": "Next turn"}))
        );
    }

    #[test]
    fn parses_only_sse_request_frames_with_request_ids() {
        let request = parse_sse_request(b"event: desktop.request\ndata: {\"request_id\":\"req-1\",\"operation\":\"projects.list\"}\n\n", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA").unwrap();
        assert_eq!(request.request_id, "req-1");
        assert!(parse_sse_request(b"event: desktop.request\r\ndata: {\"request_id\":\"req-2\",\"operation\":\"projects.list\"}\r\n\r\n", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA").is_some());
        assert!(parse_sse_request(
            b"event: heartbeat\ndata: {}\n\n",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        )
        .is_none());
        assert!(parse_sse_request(
            b"event: desktop.request\ndata: {\"operation\":\"projects.list\"}\n\n",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        )
        .is_none());
    }

    #[test]
    fn accepts_java_desktop_command_envelope_and_camel_case_payload() {
        let request = parse_sse_request(
            b"event: desktop.command\ndata: {\"requestId\":\"req-1\",\"hostId\":\"host-1\",\"sessionId\":\"session-1\",\"command\":\"session.message\",\"payload\":{\"text\":\"hello\"}}\n\n", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        )
        .unwrap();
        assert_eq!(request.request_id, "req-1");
        assert_eq!(request.operation, "session.message");
        assert_eq!(request.arguments["text"], "hello");
    }

    #[test]
    fn keeps_desktop_sse_routing_values_with_plaintext_body() {
        let request = parse_sse_request(
            b"event: mobile.sessions.approvals.resolve\nid: req-1\ndata: {\"pathParam\":{\"sessionId\":\"session-1\",\"approvalId\":\"approval-1\"},\"queryParam\":{\"revision\":7}}\ndata: {\"action\":\"allow_once\"}\n\n",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        ).unwrap();
        assert_eq!(request.operation, "approval.resolve");
        assert_eq!(request.arguments["sessionId"], "session-1");
        assert_eq!(request.arguments["approvalId"], "approval-1");
        assert_eq!(request.arguments["revision"], 7);
        assert_eq!(request.arguments["action"], "allow_once");
    }

    #[test]
    fn rejects_mobile_sse_frames_without_both_data_lines() {
        assert!(parse_sse_request(
            b"event: mobile.sessions.messages.send\nid: req-1\ndata: {\"pathParam\":{},\"queryParam\":{}}\n\n",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        )
        .is_none());
    }

    #[test]
    fn decrypts_top_level_encrypted_mobile_payload_on_desktop() {
        let key = generate_aes_key();
        let encrypted = encrypt_remote_payload(&json!({"text": "hello"}), &key).unwrap();
        let routing = json!({
            "pathParam": {"sessionId": "session-1"},
            "queryParam": {"expectedRevision": 7},
        });
        let body = json!({"encPayload": encrypted});
        let frame = format!(
            "event: mobile.sessions.messages.send\nid: req-1\ndata: {}\ndata: {}\n\n",
            routing, body
        );
        let request = parse_sse_request(frame.as_bytes(), &key).unwrap();
        assert_eq!(request.operation, "session.message");
        assert_eq!(request.arguments["text"], "hello");
        assert_eq!(request.arguments["sessionId"], "session-1");
        assert_eq!(request.arguments["expectedRevision"], 7);
    }

    #[test]
    fn parses_mobile_projects_list_event() {
        let request = parse_sse_request(
            b"event: mobile.projects.list\nid: 1791094808946\ndata: {\"pathParam\":{\"hostId\":\"BwyviZPiGD\"},\"queryParam\":{}}\ndata: {}\n\n",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        )
        .expect("mobile projects list event should be accepted");
        assert_eq!(request.request_id, "1791094808946");
        assert_eq!(request.operation, "projects.list");
        assert_eq!(request.arguments["hostId"], "BwyviZPiGD");
    }

    #[test]
    fn normalizes_java_question_answers_to_sdk_shape() {
        assert_eq!(
            normalize_question_answers(serde_json::json!(["yes", "no"])).unwrap(),
            serde_json::json!([["yes"], ["no"]])
        );
        assert_eq!(
            normalize_question_answers(serde_json::json!([["yes"], ["no"]])).unwrap(),
            serde_json::json!([["yes"], ["no"]])
        );
    }

    #[test]
    fn remote_url_rejects_embedded_credentials_and_non_http_schemes() {
        assert!(normalize_server_url("https://example.test").is_ok());
        assert!(normalize_server_url("https://user:password@example.test").is_err());
        assert!(normalize_server_url("file:///tmp").is_err());
    }

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

    #[test]
    fn remote_status_uses_camel_case_for_managed_clients() {
        let status = RemoteServerStatus {
            configured: true,
            connected: true,
            connecting: false,
            host_id: Some("host-1".into()),
            mobile_pairing_payload: Some("https://relay.example/pair".into()),
            access_token_expires_at: Some("2026-10-04T00:00:00Z".into()),
            mobile_pairing_code: Some("once".into()),
            mobile_pairing_url: Some("https://relay.example/pair".into()),
            error: None,
        };
        let value = serde_json::to_value(status).unwrap();
        assert_eq!(value["hostId"], "host-1");
        assert_eq!(value["mobilePairingUrl"], "https://relay.example/pair");
        assert!(value.get("host_id").is_none());
        assert!(value.get("mobile_pairing_url").is_none());
    }
}
