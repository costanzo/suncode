use super::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, net::{SocketAddr, TcpStream, ToSocketAddrs}, path::{Path, PathBuf}, sync::{Arc, Mutex}, time::{Duration, Instant, UNIX_EPOCH}};
use suncode_tool::BackgroundProcess;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PreviewStatus { Stopped, Starting, Ready, Failed }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewState {
    pub project_id: String,
    pub status: PreviewStatus,
    pub url: Option<String>,
    pub program: Option<String>,
    pub error: Option<String>,
    pub reload_generation: u64,
}

struct PreviewSlot { state: PreviewState, process: Option<BackgroundProcess>, root: PathBuf, fingerprint: u64 }

#[derive(Clone)]
pub(super) struct PreviewManager {
    operations: Arc<suncode_tool::Operations>,
    slots: Arc<Mutex<HashMap<String, PreviewSlot>>>,
}

impl PreviewManager {
    pub(super) fn new(operations: Arc<suncode_tool::Operations>) -> Self {
        Self { operations, slots: Arc::new(Mutex::new(HashMap::new())) }
    }

    pub(super) fn state(&self, project_id: &str) -> PreviewState {
        let mut slots = match self.slots.lock() { Ok(value) => value, Err(_) => return stopped(project_id) };
        if let Some(slot) = slots.get_mut(project_id) {
            if slot.process.as_ref().is_some_and(|process| !process.is_running()) {
                slot.state.status = PreviewStatus::Stopped;
                slot.process = None;
            }
            let current = project_fingerprint(&slot.root);
            if current != slot.fingerprint {
                slot.fingerprint = current;
                slot.state.reload_generation = slot.state.reload_generation.wrapping_add(1);
            }
            return slot.state.clone();
        }
        stopped(project_id)
    }

    pub(super) fn start(&self, project_id: &str, project_root: &Path, program: &str, args: Vec<String>, cwd: Option<String>, url: &str) -> Result<PreviewState, BusinessError> {
        validate_url(url)?;
        if program.trim().is_empty() || program.len() > 256 { return Err(BusinessError::invalid("preview program is invalid")); }
        self.stop(project_id);
        let process = self.operations.start_background_process(project_root, json!({"program": program, "args": args, "cwd": cwd, "timeout_ms": 86_400_000u64, "env": { "HOST": "127.0.0.1" }}))?;
        if let Err(error) = wait_until_ready(url, &process) {
            process.stop();
            return Err(error);
        }
        let state = PreviewState { project_id: project_id.into(), status: PreviewStatus::Ready, url: Some(url.into()), program: Some(program.into()), error: None, reload_generation: 0 };
        self.slots.lock().map_err(|_| BusinessError::unavailable("preview state is unavailable"))?.insert(project_id.into(), PreviewSlot { state: state.clone(), process: Some(process), root: project_root.to_path_buf(), fingerprint: project_fingerprint(project_root) });
        Ok(state)
    }

    pub(super) fn stop(&self, project_id: &str) -> PreviewState {
        if let Ok(mut slots) = self.slots.lock() {
            if let Some(mut slot) = slots.remove(project_id) { if let Some(process) = slot.process.take() { process.stop(); } return slot.state; }
        }
        stopped(project_id)
    }

    pub(super) fn shutdown(&self) {
        if let Ok(mut slots) = self.slots.lock() { for slot in slots.values_mut() { if let Some(process) = slot.process.take() { process.stop(); } } slots.clear(); }
    }
}

fn stopped(project_id: &str) -> PreviewState { PreviewState { project_id: project_id.into(), status: PreviewStatus::Stopped, url: None, program: None, error: None, reload_generation: 0 } }

fn project_fingerprint(root: &Path) -> u64 {
    fn visit(path: &Path, value: &mut u64, budget: &mut usize) {
        if *budget == 0 { return; }
        let Ok(entries) = std::fs::read_dir(path) else { return };
        for entry in entries.flatten() {
            if *budget == 0 { return; }
            let name = entry.file_name().to_string_lossy().to_string();
            if matches!(name.as_str(), ".git" | "node_modules" | "bin" | "obj" | "target") { continue; }
            *budget -= 1;
            let Ok(metadata) = entry.metadata() else { continue };
            let modified = metadata.modified().ok().and_then(|time| time.duration_since(UNIX_EPOCH).ok()).map(|duration| duration.as_secs()).unwrap_or_default();
            *value = value.wrapping_add(metadata.len()).wrapping_add(modified);
            if metadata.is_dir() { visit(&entry.path(), value, budget); }
        }
    }
    let mut value = 0u64;
    visit(root, &mut value, &mut 20_000);
    value
}

fn validate_url(value: &str) -> Result<(), BusinessError> {
    let url = url::Url::parse(value).map_err(|_| BusinessError::invalid("preview URL is invalid"))?;
    if url.scheme() != "http" || url.host_str().is_none_or(|host| !matches!(host, "localhost" | "127.0.0.1")) || !url.username().is_empty() { return Err(BusinessError::new("preview_url_denied", "preview URL must be a loopback HTTP URL")); }
    Ok(())
}

fn wait_until_ready(value: &str, process: &BackgroundProcess) -> Result<(), BusinessError> {
    let parsed = url::Url::parse(value).map_err(|_| BusinessError::invalid("preview URL is invalid"))?;
    let port = parsed.port_or_known_default().ok_or_else(|| BusinessError::invalid("preview URL must include a known HTTP port"))?;
    // validate_url limits the host to localhost/127.0.0.1. Resolve it rather than
    // assuming IPv4: dev servers such as Vite bind `localhost`, which is ::1 on macOS.
    let host = parsed.host_str().unwrap_or("127.0.0.1");
    let addresses: Vec<SocketAddr> = (host, port)
        .to_socket_addrs()
        .map_err(|_| BusinessError::new("preview_server_unavailable", "preview server address could not be resolved"))?
        .collect();
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if !process.is_running() {
            return Err(BusinessError::new("preview_server_exited", "preview server exited before it became ready"));
        }
        if addresses.iter().any(|address| TcpStream::connect_timeout(address, Duration::from_millis(150)).is_ok()) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(BusinessError::new("preview_server_not_ready", "preview server did not listen within 30 seconds"))
}

#[cfg(test)]
mod tests {
    use super::validate_url;

    #[test]
    fn preview_urls_are_loopback_only() {
        assert!(validate_url("http://127.0.0.1:5173/").is_ok());
        assert!(validate_url("http://localhost:5173/").is_ok());
        assert!(validate_url("https://localhost:5173/").is_err());
        assert!(validate_url("http://192.168.1.1:5173/").is_err());
    }
}
