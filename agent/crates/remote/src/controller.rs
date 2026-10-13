use serde_json::{json, Value};
use std::{
    sync::{Arc, Mutex},
    thread::JoinHandle,
    time::Duration,
};
use suncode_common::BusinessError;
use suncode_data::Store;
use tokio_util::sync::CancellationToken;

use crate::{
    client::{
        decode_pair_response, log_http_request, remote_http_client, remote_http_error,
        resolve_endpoint, RemoteTimeout,
    },
    config::*,
    crypto::generate_aes_key,
    logging,
    worker::remote_worker,
    RemoteHost, RemoteResult,
};

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
        let status = RemoteServerStatus::from_persisted(persisted.as_ref(), configured, false);
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
        let client = remote_http_client(
            &self.host.network_configuration(),
            RemoteTimeout::Total(Duration::from_secs(15)),
        )?;
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
        self.set_status(RemoteServerStatus::from_persisted(
            Some(&persisted),
            true,
            true,
        ));
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
                        logging::error("remote", format!("worker runtime failed: {error}"))
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
