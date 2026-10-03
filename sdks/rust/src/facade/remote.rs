use super::*;
use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm, Nonce};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use futures_util::StreamExt;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, thread::JoinHandle, time::Duration};
use tokio_util::sync::CancellationToken;
use url::Url;

const REMOTE_SERVER_URL: &str = "remote_server_url";
const REMOTE_PAIRING_CODE: &str = "remote_pairing_code";
const REMOTE_HOST_ID: &str = "remote_host_id";
const REMOTE_DESKTOP_TOKEN: &str = "remote_desktop_token";
const REMOTE_REFRESH_TOKEN: &str = "remote_refresh_token";
const REMOTE_ACCESS_TOKEN_EXPIRES_AT: &str = "remote_access_token_expires_at";
const REMOTE_MOBILE_PAIRING_CODE: &str = "remote_mobile_pairing_code";
const REMOTE_AES_KEY: &str = "remote_aes_key";

const UPLOAD_EVENT_TYPES: &[&str] = &[
    "turn.state",
    "tool.state",
    "tool.requested",
    "tool.result",
    "message.user",
    "message.assistant",
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
pub struct RemoteServerConfiguration {
    pub server_url: String,
    pub pairing_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    fn mobile_pairing_url(&self, endpoint: &str) -> String {
        let separator = if endpoint.contains('?') { '&' } else { '?' };
        format!(
            "{}{}code={}&hostId={}&k={}",
            endpoint.trim_end_matches('/'),
            separator,
            urlencoding(&self.mobile_pairing_code),
            urlencoding(&self.host_id),
            urlencoding(&self.aes_key)
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

#[derive(Debug, Serialize)]
struct RemoteResult {
    code: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<Value>,
}

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
    persisted: Mutex<Option<PersistedRemote>>,
    status: Arc<Mutex<RemoteServerStatus>>,
    worker: Mutex<Option<Worker>>,
}

impl RemoteController {
    pub fn new(store: Store) -> Self {
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
            mobile_pairing_payload: persisted
                .as_ref()
                .map(|value| value.pairing.mobile_pairing_url(&value.config.server_url)),
            access_token_expires_at: persisted
                .as_ref()
                .map(|value| value.pairing.access_token_expires_at.clone()),
            mobile_pairing_code: persisted
                .as_ref()
                .map(|value| value.pairing.mobile_pairing_code.clone()),
            mobile_pairing_url: persisted
                .as_ref()
                .map(|value| value.pairing.mobile_pairing_url(&value.config.server_url)),
            error: None,
        };
        Self {
            store,
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

    pub fn save_configuration(&self, config: RemoteServerConfiguration) -> SdkResult<()> {
        let config = validate_remote_configuration(config)?;
        self.stop_worker();
        for (key, value) in [
            (REMOTE_SERVER_URL, json!(config.server_url)),
            (REMOTE_PAIRING_CODE, json!(config.pairing_code)),
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

    pub async fn connect(&self, sdk: &AsyncAgentSdk) -> SdkResult<RemoteServerStatus> {
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
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|error| {
                BusinessError::unavailable(format!("Remote HTTP client could not start: {error}"))
            })?;
        let endpoint = resolve_endpoint(url.as_str(), "/v1/desktop/pairings");
        let response = client
            .post(endpoint)
            .json(&PairRequest {
                pairing_code: config.pairing_code.clone(),
                display_name: hostname(),
                agent_version: suncode_agent::version().to_string(),
            })
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
        self.start_worker(sdk.remote_view(), persisted)?;
        Ok(self.status())
    }

    pub fn disconnect(&self) -> SdkResult<RemoteServerStatus> {
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

    pub fn clear(&self) -> SdkResult<RemoteServerStatus> {
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

    pub fn start_saved(&self, sdk: &AsyncAgentSdk) -> SdkResult<()> {
        let Some(persisted) = self.persisted.lock().ok().and_then(|value| value.clone()) else {
            return Ok(());
        };
        self.start_worker(sdk.remote_view(), persisted)
    }

    fn start_worker(&self, sdk: AsyncAgentSdk, persisted: PersistedRemote) -> SdkResult<()> {
        let stop = CancellationToken::new();
        let thread_stop = stop.clone();
        let status = self.status.clone();
        let config = persisted.clone();
        self.set_status(RemoteServerStatus {
            configured: true,
            connected: false,
            connecting: true,
            host_id: Some(persisted.pairing.host_id.clone()),
            mobile_pairing_payload: Some(
                persisted
                    .pairing
                    .mobile_pairing_url(&persisted.config.server_url),
            ),
            access_token_expires_at: Some(persisted.pairing.access_token_expires_at.clone()),
            mobile_pairing_code: Some(persisted.pairing.mobile_pairing_code.clone()),
            mobile_pairing_url: Some(
                persisted
                    .pairing
                    .mobile_pairing_url(&persisted.config.server_url),
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
                        runtime.block_on(remote_worker(sdk, config, thread_stop, status))
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
}

impl AsyncAgentSdk {
    pub fn remote_server_configuration(&self) -> RemoteServerConfiguration {
        self.remote
            .as_ref()
            .map(|remote| remote.configuration())
            .unwrap_or(RemoteServerConfiguration {
                server_url: String::new(),
                pairing_code: String::new(),
            })
    }

    pub fn remote_server_status(&self) -> RemoteServerStatus {
        self.remote
            .as_ref()
            .map(|remote| remote.status())
            .unwrap_or(RemoteServerStatus {
                configured: false,
                connected: false,
                connecting: false,
                host_id: None,
                mobile_pairing_payload: None,
                access_token_expires_at: None,
                mobile_pairing_code: None,
                mobile_pairing_url: None,
                error: None,
            })
    }

    pub fn save_remote_server_configuration(
        &self,
        configuration: RemoteServerConfiguration,
    ) -> SdkResult<RemoteServerStatus> {
        let remote = self.remote.as_ref().ok_or_else(|| {
            BusinessError::unavailable("Remote Server is unavailable in this SDK view")
        })?;
        remote.save_configuration(configuration)?;
        Ok(remote.status())
    }

    pub async fn connect_remote_server(&self) -> SdkResult<RemoteServerStatus> {
        let remote = self.remote.as_ref().ok_or_else(|| {
            BusinessError::unavailable("Remote Server is unavailable in this SDK view")
        })?;
        match remote.connect(self).await {
            Ok(status) => Ok(status),
            Err(error) => {
                remote.set_status(RemoteServerStatus {
                    configured: true,
                    connected: false,
                    connecting: false,
                    host_id: None,
                    mobile_pairing_payload: None,
                    access_token_expires_at: None,
                    mobile_pairing_code: None,
                    mobile_pairing_url: None,
                    error: Some(error.message.clone()),
                });
                Err(error)
            }
        }
    }

    pub fn disconnect_remote_server(&self) -> SdkResult<RemoteServerStatus> {
        self.remote
            .as_ref()
            .ok_or_else(|| {
                BusinessError::unavailable("Remote Server is unavailable in this SDK view")
            })?
            .disconnect()
    }

    pub fn clear_remote_server_configuration(&self) -> SdkResult<RemoteServerStatus> {
        self.remote
            .as_ref()
            .ok_or_else(|| {
                BusinessError::unavailable("Remote Server is unavailable in this SDK view")
            })?
            .clear()
    }
}

fn validate_remote_configuration(
    mut config: RemoteServerConfiguration,
) -> SdkResult<RemoteServerConfiguration> {
    config.server_url = normalize_server_url(&config.server_url)?.to_string();
    config.pairing_code = config.pairing_code.trim().to_string();
    if config.pairing_code.is_empty() || config.pairing_code.len() > 256 {
        return Err(BusinessError::invalid("pairing code is required"));
    }
    Ok(config)
}

fn normalize_server_url(value: &str) -> SdkResult<Url> {
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
    UPLOAD_EVENT_TYPES.contains(&event_type) && event_type != "assistant.delta"
}

fn hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "SunCode Desktop".into())
}

fn load_persisted(store: &Store) -> SdkResult<Option<PersistedRemote>> {
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

async fn decode_pair_response(response: Response) -> SdkResult<PairResponse> {
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
    BusinessError::unavailable(format!("Remote Server request failed: {error}"))
}

async fn remote_worker(
    sdk: AsyncAgentSdk,
    persisted: PersistedRemote,
    stop: CancellationToken,
    status: Arc<Mutex<RemoteServerStatus>>,
) {
    let client = match Client::builder().timeout(Duration::from_secs(45)).build() {
        Ok(client) => client,
        Err(error) => {
            set_worker_status(&status, false, Some(error.to_string()));
            return;
        }
    };
    let host_id = persisted.pairing.host_id.clone();
    let mut subscribed = HashSet::new();
    if let Err(error) = upload_desktop_snapshot(&sdk, &client, &persisted, &host_id).await {
        logging::write_business_error("remote", "snapshot", &error, "phase=initial-sync");
    }
    subscribe_existing_sessions(&sdk, &client, &persisted, &host_id, &stop, &mut subscribed);
    let mut delay = Duration::from_secs(1);
    let mut snapshot_tick = tokio::time::interval(Duration::from_secs(30));
    snapshot_tick.tick().await;
    loop {
        if stop.is_cancelled() {
            break;
        }
        set_worker_status(&status, false, None);
        let request_url = resolve_endpoint(&persisted.config.server_url, "/v1/desktop/events");
        let response = request_headers(client.get(request_url), &persisted, &host_id)
            .header("Accept", "text/event-stream")
            .send()
            .await;
        match response {
            Ok(response) if response.status().is_success() => {
                set_worker_status(&status, true, None);
                delay = Duration::from_secs(1);
                let mut stream = response.bytes_stream();
                let mut buffer = Vec::new();
                loop {
                    tokio::select! {
                        _ = stop.cancelled() => return,
                        _ = snapshot_tick.tick() => {
                            if let Err(error) = upload_desktop_snapshot(&sdk, &client, &persisted, &host_id).await {
                                logging::write_business_error("remote", "snapshot", &error, "phase=periodic-sync");
                            }
                            subscribe_existing_sessions(&sdk, &client, &persisted, &host_id, &stop, &mut subscribed);
                        }
                        item = stream.next() => match item {
                            Some(Ok(bytes)) => {
                                buffer.extend_from_slice(&bytes);
                                while let Some((boundary, delimiter_len)) = sse_frame_boundary(&buffer) {
                                    let frame = buffer.drain(..boundary + delimiter_len).collect::<Vec<_>>();
                                    if let Some(request) = parse_sse_request(&frame, &persisted.pairing.aes_key) {
                                        handle_remote_request(&sdk, &client, &persisted, &host_id, request).await;
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
    sdk: &AsyncAgentSdk,
    client: &Client,
    persisted: &PersistedRemote,
    host_id: &str,
) -> SdkResult<()> {
    let projects = sdk.list_projects()?.projects;
    let mut sessions = Vec::new();
    for project in &projects {
        let result = sdk.list_sessions(&project.project_id)?;
        sessions.extend(result.sessions);
    }
    let payload = json!({ "projects": projects, "sessions": sessions });
    let request_id = uuid::Uuid::new_v4().to_string();
    let url = resolve_endpoint(&persisted.config.server_url, "/v1/desktop/snapshot");
    let response = request_headers(client.post(url), persisted, host_id)
        .header("X-Request-Id", request_id)
        .json(&payload)
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
    sdk: &AsyncAgentSdk,
    client: &Client,
    persisted: &PersistedRemote,
    host_id: &str,
    stop: &CancellationToken,
    subscribed: &mut HashSet<String>,
) {
    let Ok(projects) = sdk.list_projects() else {
        return;
    };
    for project in projects.projects {
        let Ok(sessions) = sdk.list_sessions(&project.project_id) else {
            continue;
        };
        for session in sessions.sessions {
            if !subscribed.insert(session.session_id.clone()) {
                continue;
            }
            let Ok(mut stream) = sdk.subscribe_session_events(&session.session_id) else {
                continue;
            };
            let client = client.clone();
            let persisted = persisted.clone();
            let host_id = host_id.to_string();
            let stop = stop.clone();
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = stop.cancelled() => break,
                        event = stream.next() => match event {
                            Some(Ok(event)) => {
                                let event_type = event.event_type().as_str();
                                if !should_upload_event(event_type) { continue; }
                                let event_id = uuid::Uuid::new_v4().to_string();
                                let body = RemoteEventEnvelope { session_id: event.session_id.clone(), occurred_at: event.occurred_at.clone(), event_type: event_type.to_string(), payload: event.payload.clone().into_value() };
                                let url = resolve_endpoint(&persisted.config.server_url, "/v1/desktop/events");
                                let response = request_headers(client.post(url), &persisted, &host_id).header("X-Request-Id", &event_id).json(&body).send().await;
                                if let Err(error) = response { logging::warn("remote", &format!("event upload failed: {error}")); }
                            }
                            Some(Err(error)) => { logging::warn("remote", &format!("session event stream ended: {error}")); break; }
                            None => break,
                        }
                    }
                }
            });
        }
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

fn parse_sse_request(frame: &[u8], aes_key: &str) -> Option<RemoteRequest> {
    let text = std::str::from_utf8(frame).ok()?;
    let mut event = String::new();
    let mut id = String::new();
    let mut data = String::new();
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("event:") {
            event = value.trim().into();
        }
        if let Some(value) = line.strip_prefix("id:") {
            id = value.trim().into();
        }
        if let Some(value) = line.strip_prefix("data:") {
            if !data.is_empty() {
                data.push('\n');
            }
            data.push_str(value.trim_start());
        }
    }
    let mut value: Value = serde_json::from_str(&data).ok()?;
    if event.starts_with("mobile.") {
        let request_id = id;
        let operation = mobile_event_operation(&event)?;
        let mut arguments = if let Some(enc_payload) = value.get("encPayload").and_then(Value::as_str) {
            decrypt_remote_payload(enc_payload, aes_key).ok()?
        } else {
            value.get("requestBody").cloned().unwrap_or_else(|| json!({}))
        };
        if !arguments.is_object() { arguments = json!({}); }
        if let Some(path) = value.get("pathParam").and_then(Value::as_object) {
            if let Some(object) = arguments.as_object_mut() {
                for (key, value) in path { object.entry(key.clone()).or_insert_with(|| value.clone()); }
            }
        }
        if let Some(query) = value.get("queryParam").and_then(Value::as_object) {
            if let Some(object) = arguments.as_object_mut() {
                for (key, value) in query { object.entry(key.clone()).or_insert_with(|| value.clone()); }
            }
        }
        return Some(RemoteRequest { request_id, operation: operation.into(), arguments });
    }
    if event != "desktop.request" && event != "desktop.command" { return None; }
    if value.get("request_id").is_none() && value.get("requestId").is_none() {
        value["request_id"] = Value::String(id);
    }
    serde_json::from_value(value).ok().filter(|request: &RemoteRequest| !request.request_id.trim().is_empty())
}

fn decrypt_remote_payload(payload: &str, key_text: &str) -> SdkResult<Value> {
    let encoded = payload.strip_prefix("e2e-v1:")
        .ok_or_else(|| BusinessError::invalid("unsupported encrypted payload version"))?;
    let bytes = URL_SAFE_NO_PAD.decode(encoded)
        .map_err(|_| BusinessError::invalid("encrypted payload is not valid base64url"))?;
    if bytes.len() < 12 + 16 {
        return Err(BusinessError::invalid("encrypted payload is truncated"));
    }
    let key = URL_SAFE_NO_PAD.decode(key_text)
        .map_err(|_| BusinessError::invalid("remote encryption key is not valid base64url"))?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| BusinessError::invalid("remote encryption key is invalid"))?;
    let nonce = Nonce::from_slice(&bytes[..12]);
    let plaintext = cipher.decrypt(nonce, &bytes[12..])
        .map_err(|_| BusinessError::invalid("encrypted payload authentication failed"))?;
    serde_json::from_slice(&plaintext)
        .map_err(|_| BusinessError::invalid("encrypted payload JSON is invalid"))
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
    sdk: &AsyncAgentSdk,
    client: &Client,
    persisted: &PersistedRemote,
    host_id: &str,
    request: RemoteRequest,
) {
    let request_id = request.request_id;
    let outcome = dispatch_request(sdk, &request_id, &request.operation, request.arguments).await;
    let result = match outcome {
        Ok(data) => RemoteResult {
            code: 0,
            message: None,
            payload: Some(data),
        },
        Err(error) => RemoteResult {
            code: 1,
            message: Some(error.message.into()),
            payload: None,
        },
    };
    let url = resolve_endpoint(&persisted.config.server_url, "/v1/desktop/responses");
    let _ = request_headers(client.post(url), persisted, host_id)
        .header("X-Request-Id", &request_id)
        .json(&result)
        .send()
        .await;
}

async fn dispatch_request(
    sdk: &AsyncAgentSdk,
    request_id: &str,
    operation: &str,
    arguments: Value,
) -> SdkResult<Value> {
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
            serde_json::to_value(sdk.list_sessions(project_id)?)
                .map_err(|e| BusinessError::unavailable(e.to_string()))
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
            let turn_id = arguments.get("turn_id").and_then(Value::as_str)
                .or_else(|| arguments.get("turnId").and_then(Value::as_str));
            let result = if let Some(turn_id) = turn_id {
                sdk.cancel_turn(session_id, turn_id)?
            } else {
                let snapshot = sdk.session_snapshot(session_id, 0)?;
                let turn_id = snapshot.conversation_turns.last().map(|turn| turn.turn_id.as_str())
                    .ok_or_else(|| BusinessError::invalid("turnId is required"))?;
                sdk.cancel_turn(session_id, turn_id)?
            };
            serde_json::to_value(result)
                .map_err(|e| BusinessError::unavailable(e.to_string()))
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
}

fn argument_value<'a>(arguments: &'a Value, snake: &str, camel: &str) -> Option<&'a Value> {
    arguments.get(snake).or_else(|| arguments.get(camel))
}

fn normalize_question_answers(answers: Value) -> SdkResult<Value> {
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

    #[test]
    fn allowlist_uploads_complete_assistant_and_never_delta() {
        assert!(should_upload_event("message.assistant"));
        assert!(should_upload_event("message.user"));
        assert!(!should_upload_event("assistant.delta"));
        assert!(!should_upload_event("provider.exchange.progress"));
        assert!(!should_upload_event("unknown.event"));
    }

    #[test]
    fn parses_only_sse_request_frames_with_request_ids() {
        let request = parse_sse_request(b"event: desktop.request\ndata: {\"request_id\":\"req-1\",\"operation\":\"projects.list\"}\n\n", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA").unwrap();
        assert_eq!(request.request_id, "req-1");
        assert!(parse_sse_request(b"event: desktop.request\r\ndata: {\"request_id\":\"req-2\",\"operation\":\"projects.list\"}\r\n\r\n", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA").is_some());
        assert!(parse_sse_request(b"event: heartbeat\ndata: {}\n\n", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA").is_none());
        assert!(parse_sse_request(
            b"event: desktop.request\ndata: {\"operation\":\"projects.list\"}\n\n", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
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
            b"event: mobile.sessions.approvals.resolve\nid: req-1\ndata: {\"pathParam\":{\"sessionId\":\"session-1\",\"approvalId\":\"approval-1\"},\"queryParam\":{\"revision\":7},\"requestBody\":{\"action\":\"allow_once\"}}\n\n",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        ).unwrap();
        assert_eq!(request.operation, "approval.resolve");
        assert_eq!(request.arguments["sessionId"], "session-1");
        assert_eq!(request.arguments["approvalId"], "approval-1");
        assert_eq!(request.arguments["revision"], 7);
        assert_eq!(request.arguments["action"], "allow_once");
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
}
