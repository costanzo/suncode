use futures_util::StreamExt;
use reqwest::Client;
use serde_json::{json, Value};
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
    time::Duration,
};
use suncode_common::BusinessError;
use tokio_util::sync::CancellationToken;

use crate::{
    client::{
        log_http_request, remote_http_client, remote_http_error, request_headers, resolve_endpoint,
        RemoteTimeout, EVENT_STREAM_IDLE_TIMEOUT,
    },
    config::{PersistedRemote, RemoteServerStatus},
    crypto::{remote_request_body, remote_response_body},
    events::{
        flush_remote_assistant_delta, redact_remote_value, should_upload_event,
        upload_remote_session_event, RemoteAssistantDeltaBuffer, REMOTE_DELTA_FLUSH_INTERVAL,
    },
    logging,
    sse::{
        is_sse_comment, log_sse_frame, parse_sse_request, sse_frame_boundary, RemoteRequest,
        MAX_SSE_FRAME_BYTES,
    },
    RemoteHost, RemoteResult,
};

pub(crate) async fn remote_worker(
    host: Arc<dyn RemoteHost>,
    persisted: PersistedRemote,
    stop: CancellationToken,
    status: Arc<Mutex<RemoteServerStatus>>,
) {
    let network = host.network_configuration();
    let clients = remote_http_client(&network, RemoteTimeout::Total(Duration::from_secs(45)))
        .and_then(|client| {
            remote_http_client(&network, RemoteTimeout::Idle(EVENT_STREAM_IDLE_TIMEOUT))
                .map(|events| (client, events))
        });
    let (client, event_client) = match clients {
        Ok(clients) => clients,
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
        let response = request_headers(event_client.get(request_url), &persisted, &host_id)
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
                                if buffer.len() > MAX_SSE_FRAME_BYTES && sse_frame_boundary(&buffer).is_none() {
                                    set_worker_status(&status, false, Some("Remote event frame exceeded the size limit".into()));
                                    break;
                                }
                                while let Some((boundary, delimiter_len)) = sse_frame_boundary(&buffer) {
                                    let frame = buffer.drain(..boundary + delimiter_len).collect::<Vec<_>>();
                                    log_sse_frame(&frame);
                                    if is_sse_comment(&frame) {
                                        logging::debug("remote", "SSE comment received (heartbeat)");
                                        continue;
                                    }
                                    if let Some(request) = parse_sse_request(&frame, &persisted.pairing.aes_key, persisted.config.e2e_enabled) {
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
    let request_id = suncode_common::id::new(suncode_common::id::IdKind::RemoteRequest);
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
                            logging::warn("remote", format!("session event stream ended: {error}"));
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

fn set_worker_status(status: &Mutex<RemoteServerStatus>, connected: bool, error: Option<String>) {
    if let Ok(mut value) = status.lock() {
        value.connected = connected;
        value.connecting = !connected && error.is_none();
        value.error = error;
    }
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
                format!("response encryption failed: {}", error.message),
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
