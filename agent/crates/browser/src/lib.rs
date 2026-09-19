//! Bounded protocol and process lifecycle for the bundled Playwright Browser Use worker.
//!
//! This crate owns no model semantics, policy, approvals, persistence, project files, or UI DTOs.

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Stdio,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use suncode_common::BusinessError;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
    sync::Mutex,
};

const MAX_FRAME_BYTES: usize = 8 * 1024 * 1024;
pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeLock {
    pub schema_version: u32,
    pub protocol_version: u32,
    pub worker: WorkerLock,
    pub node: NodeLock,
    pub playwright: PlaywrightLock,
    pub chromium: ChromiumLock,
    pub ffmpeg: FfmpegLock,
    pub linux: LinuxLock,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkerLock {
    pub version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NodeLock {
    pub version: String,
    pub targets: BTreeMap<String, NodeTargetLock>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NodeTargetLock {
    pub archive: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlaywrightLock {
    pub version: String,
    pub package_integrity: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChromiumLock {
    pub version: String,
    pub revision: String,
    pub regular_build_only: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FfmpegLock {
    pub revision: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LinuxLock {
    pub supported_distributions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkerHello {
    pub protocol_version: u32,
    pub worker_version: String,
    pub node_version: String,
    pub playwright_version: String,
    pub chromium_version: String,
    pub chromium_revision: String,
    pub target: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeManifest {
    pub schema_version: u32,
    pub target: String,
    pub protocol_version: u32,
    pub worker_version: String,
    pub node_version: String,
    pub playwright_version: String,
    pub chromium_version: String,
    pub chromium_revision: String,
    pub ffmpeg_revision: String,
    pub node_archive: String,
    pub node_archive_sha256: String,
    pub node_tree_sha256: String,
    pub worker_tree_sha256: String,
    pub browser_tree_sha256: String,
}

#[derive(Debug, Clone)]
pub struct RuntimeLayout {
    pub node_path: PathBuf,
    pub worker_path: PathBuf,
    pub runtime_lock_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct WorkerLaunch {
    pub layout: RuntimeLayout,
    pub working_directory: PathBuf,
    pub startup_timeout: Duration,
    pub request_timeout: Duration,
    pub environment: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct Envelope<T> {
    id: Option<u64>,
    ok: bool,
    event: Option<String>,
    result: Option<T>,
    error: Option<WorkerError>,
}

#[derive(Debug, Deserialize)]
struct WorkerError {
    code: String,
    message: String,
}

pub struct WorkerProcess {
    stdin: Mutex<ChildStdin>,
    stdout: Mutex<BufReader<ChildStdout>>,
    child: Mutex<Child>,
    next_id: AtomicU64,
    request_timeout: Duration,
    pub hello: WorkerHello,
}

impl RuntimeLock {
    pub fn read(path: &Path) -> Result<Self, BusinessError> {
        let bytes = std::fs::read(path).map_err(|error| {
            browser_error(
                "browser_runtime_missing",
                format!("browser runtime lock is unavailable: {error}"),
            )
        })?;
        serde_json::from_slice(&bytes).map_err(|_| {
            browser_error("browser_runtime_invalid", "browser runtime lock is invalid")
        })
    }

    pub fn validate_hello(&self, hello: &WorkerHello) -> Result<(), BusinessError> {
        let expected_target = target_name();
        if self.schema_version != 1
            || self.protocol_version != PROTOCOL_VERSION
            || hello.protocol_version != self.protocol_version
            || hello.worker_version != self.worker.version
            || hello.node_version != self.node.version
            || hello.playwright_version != self.playwright.version
            || hello.chromium_version != self.chromium.version
            || hello.chromium_revision != self.chromium.revision
            || hello.target != expected_target
            || !self.node.targets.contains_key(expected_target)
        {
            return Err(browser_error(
                "browser_runtime_mismatch",
                "bundled browser runtime identity does not match the installation manifest",
            ));
        }
        Ok(())
    }
}

pub fn verify_runtime_tree(root: &Path) -> Result<RuntimeManifest, BusinessError> {
    let lock = RuntimeLock::read(&root.join("runtime-lock.json"))?;
    let manifest: RuntimeManifest = serde_json::from_slice(
        &std::fs::read(root.join("runtime-manifest.json")).map_err(|error| {
            browser_error(
                "browser_runtime_missing",
                format!("browser runtime manifest is unavailable: {error}"),
            )
        })?,
    )
    .map_err(|_| {
        browser_error(
            "browser_runtime_invalid",
            "browser runtime manifest is invalid",
        )
    })?;
    let expected_node = lock.node.targets.get(target_name()).ok_or_else(|| {
        browser_error(
            "browser_platform_unsupported",
            "browser runtime target is not present in the runtime lock",
        )
    })?;
    if manifest.schema_version != 1
        || manifest.target != target_name()
        || manifest.protocol_version != lock.protocol_version
        || manifest.worker_version != lock.worker.version
        || manifest.node_version != lock.node.version
        || manifest.playwright_version != lock.playwright.version
        || manifest.chromium_version != lock.chromium.version
        || manifest.chromium_revision != lock.chromium.revision
        || manifest.ffmpeg_revision != lock.ffmpeg.revision
        || manifest.node_archive != expected_node.archive
        || manifest.node_archive_sha256 != expected_node.sha256
    {
        return Err(browser_error(
            "browser_runtime_mismatch",
            "browser runtime manifest does not match the runtime lock",
        ));
    }
    for (directory, expected) in [
        (root.join("node"), manifest.node_tree_sha256.as_str()),
        (root.join("worker"), manifest.worker_tree_sha256.as_str()),
        (root.join("browsers"), manifest.browser_tree_sha256.as_str()),
    ] {
        let actual = tree_hash(&directory)?;
        if actual != expected {
            return Err(browser_error(
                "browser_runtime_integrity_failed",
                "browser runtime files do not match the packaged manifest",
            ));
        }
    }
    Ok(manifest)
}

impl WorkerProcess {
    pub async fn spawn(launch: WorkerLaunch) -> Result<Self, BusinessError> {
        validate_layout(&launch.layout)?;
        let node_path = std::fs::canonicalize(&launch.layout.node_path).map_err(|error| {
            browser_error(
                "browser_runtime_missing",
                format!("Node.js executable could not be resolved: {error}"),
            )
        })?;
        let worker_path = std::fs::canonicalize(&launch.layout.worker_path).map_err(|error| {
            browser_error(
                "browser_runtime_missing",
                format!("browser worker could not be resolved: {error}"),
            )
        })?;
        std::fs::create_dir_all(&launch.working_directory).map_err(|error| {
            browser_error(
                "browser_runtime_unavailable",
                format!("browser runtime directory could not be created: {error}"),
            )
        })?;
        let runtime_lock = RuntimeLock::read(&launch.layout.runtime_lock_path)?;
        let mut command = Command::new(node_path);
        command
            .arg(worker_path)
            .current_dir(&launch.working_directory)
            .env_clear()
            .envs(filtered_environment(&launch.environment))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        #[cfg(unix)]
        command.process_group(0);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let mut child = command.spawn().map_err(|error| {
            browser_error(
                "browser_process_start_failed",
                format!("browser worker could not start: {error}"),
            )
        })?;
        let stdin = child.stdin.take().ok_or_else(|| {
            browser_error(
                "browser_process_start_failed",
                "browser worker stdin is unavailable",
            )
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            browser_error(
                "browser_process_start_failed",
                "browser worker stdout is unavailable",
            )
        })?;
        let mut stdout = BufReader::new(stdout);
        let initial = tokio::time::timeout(
            launch.startup_timeout,
            read_frame::<_, Envelope<WorkerHello>>(&mut stdout),
        )
        .await
        .map_err(|_| {
            browser_error(
                "browser_startup_timeout",
                "browser worker startup timed out",
            )
        })??;
        if !initial.ok || initial.event.as_deref() != Some("hello") {
            let _ = child.kill().await;
            return Err(worker_envelope_error(initial));
        }
        let hello = initial.result.ok_or_else(|| {
            browser_error(
                "browser_runtime_invalid",
                "browser worker hello result is missing",
            )
        })?;
        runtime_lock.validate_hello(&hello)?;
        Ok(Self {
            stdin: Mutex::new(stdin),
            stdout: Mutex::new(stdout),
            child: Mutex::new(child),
            next_id: AtomicU64::new(1),
            request_timeout: launch.request_timeout,
            hello,
        })
    }

    pub async fn request<T: DeserializeOwned>(
        &self,
        method: &str,
        params: Value,
    ) -> Result<T, BusinessError> {
        if method.trim().is_empty() {
            return Err(BusinessError::invalid("browser worker method is required"));
        }
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let mut stdin = self.stdin.lock().await;
        let mut stdout = self.stdout.lock().await;
        write_frame(
            &mut *stdin,
            &json!({"id":id,"method":method,"params":params}),
        )
        .await?;
        let envelope = tokio::time::timeout(
            self.request_timeout,
            read_frame::<_, Envelope<T>>(&mut *stdout),
        )
        .await
        .map_err(|_| {
            browser_error(
                "browser_request_timeout",
                "browser worker request timed out",
            )
        })??;
        if envelope.id != Some(id) {
            return Err(browser_error(
                "browser_protocol_error",
                "browser worker response ID did not match the request",
            ));
        }
        if !envelope.ok {
            return Err(worker_envelope_error(envelope));
        }
        envelope.result.ok_or_else(|| {
            browser_error(
                "browser_protocol_error",
                "browser worker response result is missing",
            )
        })
    }

    pub async fn close(&self) {
        let _ = tokio::time::timeout(
            Duration::from_secs(3),
            self.request::<Value>("shutdown", json!({})),
        )
        .await;
        self.force_close().await;
    }

    pub async fn force_close(&self) {
        let mut child = self.child.lock().await;
        terminate_process_tree(&mut child).await;
        let _ = child.kill().await;
        let _ = child.wait().await;
    }
}

impl Drop for WorkerProcess {
    fn drop(&mut self) {
        if let Ok(mut child) = self.child.try_lock() {
            terminate_process_tree_on_drop(&mut child);
            let _ = child.start_kill();
        }
    }
}

async fn terminate_process_tree(child: &mut Child) {
    #[cfg(windows)]
    if let Some(pid) = child.id() {
        use std::os::windows::process::CommandExt;

        let mut command = Command::new("taskkill");
        command
            .creation_flags(0x08000000)
            .args(["/F", "/T", "/PID", &pid.to_string()]);
        let _ = command.status().await;
    }
    #[cfg(unix)]
    if let Some(pid) = child.id() {
        unsafe {
            let _ = libc::kill(-(pid as i32), libc::SIGKILL);
        }
    }
}

fn terminate_process_tree_on_drop(child: &mut Child) {
    #[cfg(windows)]
    if let Some(pid) = child.id() {
        use std::os::windows::process::CommandExt;

        let _ = std::process::Command::new("taskkill")
            .creation_flags(0x08000000)
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .status();
    }
    #[cfg(unix)]
    if let Some(pid) = child.id() {
        unsafe {
            let _ = libc::kill(-(pid as i32), libc::SIGKILL);
        }
    }
}

pub async fn write_frame<W: AsyncWrite + Unpin, T: Serialize>(
    writer: &mut W,
    value: &T,
) -> Result<(), BusinessError> {
    let payload = serde_json::to_vec(value).map_err(|_| {
        browser_error(
            "browser_protocol_error",
            "browser frame could not be encoded",
        )
    })?;
    if payload.len() > MAX_FRAME_BYTES {
        return Err(browser_error(
            "browser_protocol_error",
            "browser frame exceeds the maximum size",
        ));
    }
    let length = u32::try_from(payload.len())
        .map_err(|_| browser_error("browser_protocol_error", "browser frame length is invalid"))?;
    writer
        .write_all(&length.to_be_bytes())
        .await
        .map_err(protocol_io_error)?;
    writer
        .write_all(&payload)
        .await
        .map_err(protocol_io_error)?;
    writer.flush().await.map_err(protocol_io_error)
}

pub async fn read_frame<R: AsyncRead + Unpin, T: DeserializeOwned>(
    reader: &mut R,
) -> Result<T, BusinessError> {
    let length = reader.read_u32().await.map_err(protocol_io_error)? as usize;
    if length > MAX_FRAME_BYTES {
        return Err(browser_error(
            "browser_protocol_error",
            "browser frame exceeds the maximum size",
        ));
    }
    let mut payload = vec![0; length];
    reader
        .read_exact(&mut payload)
        .await
        .map_err(protocol_io_error)?;
    serde_json::from_slice(&payload)
        .map_err(|_| browser_error("browser_protocol_error", "browser frame is invalid JSON"))
}

pub fn target_name() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "darwin-arm64"
    }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "win32-x64"
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "linux-x64"
    }
    #[cfg(not(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "x86_64")
    )))]
    {
        "unsupported"
    }
}

fn validate_layout(layout: &RuntimeLayout) -> Result<(), BusinessError> {
    for (path, label) in [
        (&layout.node_path, "Node.js executable"),
        (&layout.worker_path, "browser worker"),
        (&layout.runtime_lock_path, "browser runtime lock"),
    ] {
        if !path.is_file() {
            return Err(browser_error(
                "browser_runtime_missing",
                format!("{label} is missing"),
            ));
        }
    }
    if target_name() == "unsupported" {
        return Err(browser_error(
            "browser_platform_unsupported",
            "Browser Use is unavailable on this operating-system architecture",
        ));
    }
    Ok(())
}

fn filtered_environment(overrides: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    let allowed = [
        "PATH",
        "HOME",
        "USER",
        "LOGNAME",
        "SHELL",
        "LANG",
        "LC_ALL",
        "TMPDIR",
        "TEMP",
        "TMP",
        "DISPLAY",
        "WAYLAND_DISPLAY",
        "XDG_RUNTIME_DIR",
        "DBUS_SESSION_BUS_ADDRESS",
        "SystemRoot",
        "WINDIR",
        "USERPROFILE",
        "LOCALAPPDATA",
        "APPDATA",
    ];
    let mut values = allowed
        .into_iter()
        .filter_map(|name| {
            std::env::var(name)
                .ok()
                .map(|value| (name.to_string(), value))
        })
        .collect::<BTreeMap<_, _>>();
    for (name, value) in overrides {
        if !name.eq_ignore_ascii_case("NODE_OPTIONS")
            && !name.starts_with("DYLD_")
            && name != "LD_PRELOAD"
        {
            values.insert(name.clone(), value.clone());
        }
    }
    values
}

fn worker_envelope_error<T>(envelope: Envelope<T>) -> BusinessError {
    envelope
        .error
        .map(|error| browser_error(&error.code, error.message))
        .unwrap_or_else(|| {
            browser_error(
                "browser_protocol_error",
                "browser worker returned an unsuccessful response",
            )
        })
}

fn protocol_io_error(error: std::io::Error) -> BusinessError {
    browser_error(
        "browser_protocol_error",
        format!("browser worker protocol failed: {error}"),
    )
}

fn browser_error(code: &str, message: impl Into<String>) -> BusinessError {
    BusinessError::new(code, message.into()).with_retryable(true)
}

fn tree_hash(root: &Path) -> Result<String, BusinessError> {
    let mut hasher = Sha256::new();
    hash_directory(root, root, &mut hasher)?;
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn hash_directory(root: &Path, current: &Path, hasher: &mut Sha256) -> Result<(), BusinessError> {
    let mut entries = std::fs::read_dir(current)
        .map_err(|error| {
            browser_error(
                "browser_runtime_missing",
                format!("browser runtime directory is unavailable: {error}"),
            )
        })?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| {
            browser_error(
                "browser_runtime_invalid",
                format!("browser runtime file type is unavailable: {error}"),
            )
        })?;
        if file_type.is_dir() {
            hash_directory(root, &path, hasher)?;
        } else if file_type.is_file() {
            let relative = path.strip_prefix(root).map_err(|_| {
                browser_error(
                    "browser_runtime_invalid",
                    "browser runtime path escaped its root",
                )
            })?;
            hasher.update(b"file\0");
            hasher.update(relative.to_string_lossy().replace('\\', "/").as_bytes());
            hasher.update(b"\0");
            hasher.update(std::fs::read(&path).map_err(|error| {
                browser_error(
                    "browser_runtime_invalid",
                    format!("browser runtime file could not be read: {error}"),
                )
            })?);
        } else if file_type.is_symlink() {
            let relative = path.strip_prefix(root).map_err(|_| {
                browser_error(
                    "browser_runtime_invalid",
                    "browser runtime symlink escaped its root",
                )
            })?;
            let target = std::fs::read_link(&path).map_err(|error| {
                browser_error(
                    "browser_runtime_invalid",
                    format!("browser runtime symlink could not be read: {error}"),
                )
            })?;
            if target.is_absolute() {
                return Err(browser_error(
                    "browser_runtime_invalid",
                    "browser runtime contains an absolute symlink",
                ));
            }
            let canonical_root = std::fs::canonicalize(root).map_err(|error| {
                browser_error(
                    "browser_runtime_invalid",
                    format!("browser runtime root could not be resolved: {error}"),
                )
            })?;
            let canonical_target = std::fs::canonicalize(&path).map_err(|error| {
                browser_error(
                    "browser_runtime_invalid",
                    format!("browser runtime symlink target could not be resolved: {error}"),
                )
            })?;
            if !canonical_target.starts_with(&canonical_root) {
                return Err(browser_error(
                    "browser_runtime_invalid",
                    "browser runtime symlink escaped its root",
                ));
            }
            hasher.update(b"link\0");
            hasher.update(relative.to_string_lossy().replace('\\', "/").as_bytes());
            hasher.update(b"\0");
            hasher.update(target.to_string_lossy().replace('\\', "/").as_bytes());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn framed_json_round_trips() {
        let (mut left, mut right) = tokio::io::duplex(4096);
        let writer = tokio::spawn(async move {
            write_frame(&mut left, &json!({"hello":"browser"}))
                .await
                .unwrap();
        });
        let value: Value = read_frame(&mut right).await.unwrap();
        writer.await.unwrap();
        assert_eq!(value, json!({"hello":"browser"}));
    }

    #[test]
    fn committed_runtime_lock_matches_protocol_and_current_target() {
        let lock: RuntimeLock = serde_json::from_str(include_str!(
            "../../../../browser-runtime/runtime-lock.json"
        ))
        .unwrap();
        assert_eq!(lock.protocol_version, PROTOCOL_VERSION);
        if target_name() != "unsupported" {
            assert!(lock.node.targets.contains_key(target_name()));
        }
        assert!(lock.chromium.regular_build_only);
    }

    #[test]
    fn runtime_identity_mismatch_fails_closed() {
        let lock: RuntimeLock = serde_json::from_str(include_str!(
            "../../../../browser-runtime/runtime-lock.json"
        ))
        .unwrap();
        let hello = WorkerHello {
            protocol_version: PROTOCOL_VERSION,
            worker_version: "0.1.0".into(),
            node_version: "wrong".into(),
            playwright_version: lock.playwright.version.clone(),
            chromium_version: lock.chromium.version.clone(),
            chromium_revision: lock.chromium.revision.clone(),
            target: target_name().into(),
        };
        assert_eq!(
            lock.validate_hello(&hello).unwrap_err().code,
            "browser_runtime_mismatch"
        );
    }

    #[test]
    fn filtered_environment_rejects_code_injection_variables() {
        let values = filtered_environment(&BTreeMap::from([
            ("NODE_OPTIONS".into(), "--require malware.js".into()),
            ("DYLD_INSERT_LIBRARIES".into(), "bad.dylib".into()),
            ("LD_PRELOAD".into(), "bad.so".into()),
            ("SUNCODE_BROWSER_PROFILE".into(), "/safe/profile".into()),
        ]));
        assert!(!values.contains_key("NODE_OPTIONS"));
        assert!(!values.contains_key("DYLD_INSERT_LIBRARIES"));
        assert!(!values.contains_key("LD_PRELOAD"));
        assert_eq!(
            values.get("SUNCODE_BROWSER_PROFILE").map(String::as_str),
            Some("/safe/profile")
        );
    }

    #[cfg(unix)]
    #[test]
    fn runtime_tree_rejects_absolute_symlinks() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().unwrap();
        let outside = tempfile::NamedTempFile::new().unwrap();
        symlink(outside.path(), directory.path().join("escaped-link")).unwrap();

        assert_eq!(
            tree_hash(directory.path()).unwrap_err().code,
            "browser_runtime_invalid"
        );
    }
}
