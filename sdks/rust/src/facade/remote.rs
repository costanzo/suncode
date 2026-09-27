use super::*;
use futures_util::StreamExt;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, thread::JoinHandle, time::Duration};
use tokio_util::sync::CancellationToken;
use url::Url;

const REMOTE_SERVER_URL: &str = "remote_server_url";
const REMOTE_PAIRING_CODE: &str = "remote_pairing_code";
const REMOTE_HOST_ID: &str = "remote_host_id";
const REMOTE_PAIRING_PAYLOAD: &str = "remote_mobile_pairing_payload";
const REMOTE_EVENTS_URL: &str = "remote_events_url";
const REMOTE_REQUESTS_URL: &str = "remote_requests_url";
const REMOTE_RESULTS_URL: &str = "remote_results_url";

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
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PairRequest {
    pairing_code: String,
    display_name: String,
    desktop_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PairResponse {
    host_id: String,
    mobile_pairing_payload: String,
    events_url: String,
    requests_url: String,
    results_url: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RemoteRequest {
    #[serde(alias = "requestId")]
    request_id: String,
    operation: String,
    #[serde(default)]
    arguments: Value,
}

#[derive(Debug, Serialize)]
struct RemoteResult<'a> {
    request_id: &'a str,
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RemoteError>,
}

#[derive(Debug, Serialize)]
struct RemoteError {
    code: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct RemoteEventEnvelope {
    session_id: String,
    occurred_at: String,
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
                .map(|value| value.pairing.mobile_pairing_payload.clone()),
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
        self.store
            .set_setting("global", "global", REMOTE_PAIRING_PAYLOAD, &Value::Null)?;
        self.store
            .set_setting("global", "global", REMOTE_EVENTS_URL, &Value::Null)?;
        self.store
            .set_setting("global", "global", REMOTE_REQUESTS_URL, &Value::Null)?;
        self.store
            .set_setting("global", "global", REMOTE_RESULTS_URL, &Value::Null)?;
        if let Ok(mut persisted) = self.persisted.lock() {
            *persisted = None;
        }
        self.set_status(RemoteServerStatus {
            configured: true,
            connected: false,
            connecting: false,
            host_id: None,
            mobile_pairing_payload: None,
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
            error: None,
        });
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|error| {
                BusinessError::unavailable(format!("Remote HTTP client could not start: {error}"))
            })?;
        let endpoint = url
            .join("/v1/desktop/pairings")
            .map_err(|_| BusinessError::invalid("Remote Server URL is invalid"))?;
        let response = client
            .post(endpoint)
            .json(&PairRequest {
                pairing_code: config.pairing_code.clone(),
                display_name: hostname(),
                desktop_version: suncode_agent::version().to_string(),
            })
            .send()
            .await
            .map_err(remote_http_error)?;
        let pairing = decode_pair_response(response).await?;
        if pairing.host_id.trim().is_empty() || pairing.mobile_pairing_payload.trim().is_empty() {
            return Err(BusinessError::new(
                "remote_pairing_invalid",
                "Remote Server returned incomplete pairing data",
            ));
        }
        for (key, value) in [
            (REMOTE_HOST_ID, json!(pairing.host_id)),
            (
                REMOTE_PAIRING_PAYLOAD,
                json!(pairing.mobile_pairing_payload),
            ),
            (REMOTE_EVENTS_URL, json!(pairing.events_url)),
            (REMOTE_REQUESTS_URL, json!(pairing.requests_url)),
            (REMOTE_RESULTS_URL, json!(pairing.results_url)),
        ] {
            self.store.set_setting("global", "global", key, &value)?;
        }
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
            REMOTE_PAIRING_PAYLOAD,
            REMOTE_EVENTS_URL,
            REMOTE_REQUESTS_URL,
            REMOTE_RESULTS_URL,
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
            mobile_pairing_payload: Some(persisted.pairing.mobile_pairing_payload.clone()),
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
    let Some(mobile_pairing_payload) = get(REMOTE_PAIRING_PAYLOAD) else {
        return Ok(None);
    };
    let Some(events_url) = get(REMOTE_EVENTS_URL) else {
        return Ok(None);
    };
    let Some(requests_url) = get(REMOTE_REQUESTS_URL) else {
        return Ok(None);
    };
    let Some(results_url) = get(REMOTE_RESULTS_URL) else {
        return Ok(None);
    };
    Ok(Some(PersistedRemote {
        config: RemoteServerConfiguration {
            server_url,
            pairing_code,
        },
        pairing: PairResponse {
            host_id,
            mobile_pairing_payload,
            events_url,
            requests_url,
            results_url,
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
        let request_url = resolve_endpoint(
            &persisted.config.server_url,
            &persisted.pairing.requests_url,
        );
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
                                    if let Some(request) = parse_sse_request(&frame) {
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
    let url = format!(
        "{}/v1/desktop/snapshot",
        persisted.config.server_url.trim_end_matches('/')
    );
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
                                let url = resolve_endpoint(&persisted.config.server_url, &persisted.pairing.events_url);
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
    Url::parse(base)
        .and_then(|url| url.join(path))
        .map(|url| url.to_string())
        .unwrap_or_else(|_| path.to_string())
}

fn request_headers(
    request: reqwest::RequestBuilder,
    persisted: &PersistedRemote,
    host_id: &str,
) -> reqwest::RequestBuilder {
    request
        .header("X-Host-Id", host_id)
        .header("X-Pairing-Code", &persisted.config.pairing_code)
}

fn parse_sse_request(frame: &[u8]) -> Option<RemoteRequest> {
    let text = std::str::from_utf8(frame).ok()?;
    let mut event = String::new();
    let mut data = String::new();
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("event:") {
            event = value.trim().into();
        }
        if let Some(value) = line.strip_prefix("data:") {
            if !data.is_empty() {
                data.push('\n');
            }
            data.push_str(value.trim_start());
        }
    }
    if event != "desktop.request" {
        return None;
    }
    serde_json::from_str(&data)
        .ok()
        .filter(|request: &RemoteRequest| !request.request_id.trim().is_empty())
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
            request_id: &request_id,
            success: true,
            data: Some(data),
            error: None,
        },
        Err(error) => RemoteResult {
            request_id: &request_id,
            success: false,
            data: None,
            error: Some(RemoteError {
                code: error.code.into(),
                message: error.message.into(),
            }),
        },
    };
    let url = resolve_endpoint(&persisted.config.server_url, &persisted.pairing.results_url);
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
    let string = |key: &str| {
        arguments
            .get(key)
            .and_then(Value::as_str)
            .ok_or_else(|| BusinessError::invalid(format!("{key} is required")))
    };
    match operation {
        "projects.list" => serde_json::to_value(sdk.list_projects()?)
            .map_err(|e| BusinessError::unavailable(e.to_string())),
        "sessions.list" => {
            let project_id = string("project_id")?;
            serde_json::to_value(sdk.list_sessions(project_id)?)
                .map_err(|e| BusinessError::unavailable(e.to_string()))
        }
        "session.create" => {
            let project_id = string("project_id")?;
            let title = arguments.get("title").and_then(Value::as_str);
            serde_json::to_value(sdk.create_session(project_id, title, None)?)
                .map_err(|e| BusinessError::unavailable(e.to_string()))
        }
        "session.send_message" => {
            let session_id = string("session_id")?;
            let text = string("text")?;
            let model = arguments.get("model").and_then(Value::as_str);
            let effort = arguments.get("reasoning_effort").and_then(Value::as_str);
            serde_json::to_value(
                sdk.submit_turn(session_id, text, request_id, model, effort)
                    .await?,
            )
            .map_err(|e| BusinessError::unavailable(e.to_string()))
        }
        "session.cancel" => {
            let session_id = string("session_id")?;
            let turn_id = string("turn_id")?;
            serde_json::to_value(sdk.cancel_turn(session_id, turn_id)?)
                .map_err(|e| BusinessError::unavailable(e.to_string()))
        }
        "session.retry" => {
            let session_id = string("session_id")?;
            serde_json::to_value(sdk.retry_last_turn(session_id).await?)
                .map_err(|e| BusinessError::unavailable(e.to_string()))
        }
        "approval.resolve" => {
            let approval_id = string("approval_id")?;
            let decision = string("decision")?;
            serde_json::to_value(sdk.resolve_approval(approval_id, decision).await?)
                .map_err(|e| BusinessError::unavailable(e.to_string()))
        }
        "question.reply" => {
            let question_id = string("question_id")?;
            let answers = arguments
                .get("answers")
                .cloned()
                .ok_or_else(|| BusinessError::invalid("answers are required"))?;
            serde_json::to_value(sdk.reply_question(question_id, &answers).await?)
                .map_err(|e| BusinessError::unavailable(e.to_string()))
        }
        _ => Err(BusinessError::invalid("Remote operation is not supported")),
    }
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
        let request = parse_sse_request(b"event: desktop.request\ndata: {\"request_id\":\"req-1\",\"operation\":\"projects.list\"}\n\n").unwrap();
        assert_eq!(request.request_id, "req-1");
        assert!(parse_sse_request(b"event: desktop.request\r\ndata: {\"request_id\":\"req-2\",\"operation\":\"projects.list\"}\r\n\r\n").is_some());
        assert!(parse_sse_request(b"event: heartbeat\ndata: {}\n\n").is_none());
        assert!(parse_sse_request(
            b"event: desktop.request\ndata: {\"operation\":\"projects.list\"}\n\n"
        )
        .is_none());
    }

    #[test]
    fn remote_url_rejects_embedded_credentials_and_non_http_schemes() {
        assert!(normalize_server_url("https://example.test").is_ok());
        assert!(normalize_server_url("https://user:password@example.test").is_err());
        assert!(normalize_server_url("file:///tmp").is_err());
    }
}
