use crate::arguments::ProcessArguments;
use crate::artifacts;
use crate::context::ProcessOutputCallback;
use crate::error::{fatal, retryable};
use crate::journal::journal_id;
use crate::scope::{require_project, safe_relative_path};
use crate::support::{now_string, sha256_hex};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::fs;
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Mutex,
};
use suncode_common::BusinessError;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

const PREVIEW_BYTES: usize = 64 * 1024;
static OUTPUT_SEQUENCE: AtomicU64 = AtomicU64::new(1);

fn command_arguments(args: &ProcessArguments) -> Result<(String, Vec<String>), BusinessError> {
    let program = args.program.as_str();
    if program.is_empty() || program.len() > 4096 {
        return Err(fatal("invalid_arguments", "program is invalid"));
    }
    if args.args.len() > 256 {
        return Err(fatal("resource_limit", "too many process arguments"));
    }
    Ok((program.to_string(), args.args.clone()))
}

fn process_cwd(root: &Path, args: &ProcessArguments) -> Result<std::path::PathBuf, BusinessError> {
    let requested = args.cwd.as_deref().unwrap_or(".");
    let requested_path = Path::new(requested);
    let candidate = if requested_path.is_absolute() {
        requested_path.to_path_buf()
    } else {
        root.join(safe_relative_path(requested)?)
    };
    let path = candidate.canonicalize().map_err(|_| {
        fatal(
            "process_working_directory_unavailable",
            "working directory is unavailable",
        )
    })?;
    if !path.starts_with(root) || !path.is_dir() {
        return Err(fatal(
            "scope_denied",
            "working directory is outside the project",
        ));
    }
    Ok(path)
}

fn configure_command(mut command: Command, args: &ProcessArguments) -> Command {
    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW);
    #[cfg(unix)]
    command.process_group(0);
    // Tool commands must never read or steal the host's terminal input.
    command.stdin(Stdio::null());
    command.env_clear();
    for key in [
        "PATH",
        "HOME",
        "TMPDIR",
        "TMP",
        "TEMP",
        "SystemRoot",
        "ComSpec",
        "PATHEXT",
        "USERPROFILE",
        "http_proxy",
        "https_proxy",
        "no_proxy",
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "NO_PROXY",
    ] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    for (key, value) in &args.env {
        if key.len() <= 128
            && key
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            command.env(key, value);
        }
    }
    command
}

fn terminate_process_tree(child: &mut Child) {
    #[cfg(target_os = "windows")]
    {
        let pid = child.id();
        let _ = Command::new("taskkill")
            .creation_flags(CREATE_NO_WINDOW)
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .status();
    }
    #[cfg(unix)]
    {
        let pid = child.id() as i32;
        // The child is placed in its own process group by configure_command.
        unsafe {
            let _ = libc::kill(-pid, libc::SIGKILL);
        }
    }
    let _ = child.kill();
}

/// Kills processes left in the leader's process group after the leader exited,
/// such as `cmd &`, so they cannot keep the output pipes open.
fn reap_process_group(child: &Child) {
    #[cfg(unix)]
    {
        // The group id stays reserved while any member is alive, so this cannot
        // reach an unrelated group.
        unsafe {
            let _ = libc::kill(-(child.id() as i32), libc::SIGKILL);
        }
    }
    #[cfg(not(unix))]
    let _ = child;
}

/// How long output readers may keep draining after the process has exited.
const OUTPUT_DRAIN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);

/// Joins an output reader, giving up once `deadline` passes so a descendant
/// that still holds the pipe cannot block the tool call forever.
fn join_output(
    reader: Option<std::thread::JoinHandle<std::io::Result<CapturedOutput>>>,
    deadline: std::time::Instant,
) -> Option<std::thread::Result<std::io::Result<CapturedOutput>>> {
    let reader = reader?;
    while !reader.is_finished() {
        if std::time::Instant::now() >= deadline {
            // The detached thread ends when the remaining writer closes the pipe.
            return None;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    Some(reader.join())
}

/// A project-scoped process that remains alive until the host explicitly stops it.
pub struct BackgroundProcess {
    child: Mutex<Child>,
}

impl BackgroundProcess {
    pub fn stop(&self) {
        if let Ok(mut child) = self.child.lock() {
            terminate_process_tree(&mut child);
            let _ = child.wait();
        }
    }

    pub fn is_running(&self) -> bool {
        self.child
            .lock()
            .map(|mut child| child.try_wait().ok().flatten().is_none())
            .unwrap_or(false)
    }
}

impl Drop for BackgroundProcess {
    fn drop(&mut self) {
        self.stop();
    }
}

pub fn start_background(
    project_root: &Path,
    args: &ProcessArguments,
) -> Result<BackgroundProcess, BusinessError> {
    let root = require_project(Some(project_root))?;
    let (program, command_args) = command_arguments(args)?;
    let cwd = process_cwd(root, args)?;
    let child = configure_command(Command::new(&program), args)
        .args(command_args)
        .current_dir(cwd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(process_start_failure)?;
    Ok(BackgroundProcess {
        child: Mutex::new(child),
    })
}

fn process_start_failure(error: std::io::Error) -> BusinessError {
    match error.kind() {
        std::io::ErrorKind::NotFound => fatal(
            "process_executable_not_found",
            "process executable could not be found",
        ),
        std::io::ErrorKind::PermissionDenied => fatal(
            "process_permission_denied",
            "process executable could not be started because permission was denied",
        ),
        _ => fatal("process_start_failed", "process could not be started"),
    }
}

struct CapturedOutput {
    preview: Vec<u8>,
    total_bytes: usize,
    path: Option<PathBuf>,
}

impl Drop for CapturedOutput {
    fn drop(&mut self) {
        if let Some(path) = self.path.take() {
            let _ = fs::remove_file(path);
        }
    }
}

struct OutputCapture {
    preview: VecDeque<u8>,
    total_bytes: usize,
    path: Option<PathBuf>,
    file: Option<fs::File>,
    label: &'static str,
}

impl OutputCapture {
    fn new(label: &'static str) -> Self {
        Self {
            preview: VecDeque::with_capacity(PREVIEW_BYTES),
            total_bytes: 0,
            path: None,
            file: None,
            label,
        }
    }

    fn create_full_output(&mut self) -> std::io::Result<()> {
        let path = std::env::temp_dir().join(format!(
            "suncode-process-{}-{}-{}.tmp",
            std::process::id(),
            OUTPUT_SEQUENCE.fetch_add(1, Ordering::Relaxed),
            self.label
        ));
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)?;
        file.write_all(self.preview.make_contiguous())?;
        self.preview.clear();
        self.path = Some(path);
        self.file = Some(file);
        Ok(())
    }

    fn push(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        self.total_bytes = self.total_bytes.saturating_add(bytes.len());
        if self.file.is_none() && self.total_bytes > PREVIEW_BYTES {
            self.create_full_output()?;
        }
        if let Some(file) = self.file.as_mut() {
            file.write_all(bytes)?;
        }
        self.preview.extend(bytes);
        while self.preview.len() > PREVIEW_BYTES {
            self.preview.pop_front();
        }
        Ok(())
    }

    fn finish(mut self) -> std::io::Result<CapturedOutput> {
        if let Some(file) = self.file.as_mut() {
            file.flush()?;
        }
        Ok(CapturedOutput {
            preview: self.preview.into_iter().collect(),
            total_bytes: self.total_bytes,
            path: self.path.take(),
        })
    }
}

fn collect_output(
    mut reader: impl Read,
    label: &'static str,
    callback: Option<ProcessOutputCallback>,
) -> std::io::Result<CapturedOutput> {
    let mut output = OutputCapture::new(label);
    let mut buffer = [0u8; 8192];
    loop {
        let bytes = reader.read(&mut buffer)?;
        if bytes == 0 {
            break;
        }
        if let Some(callback) = callback.as_ref() {
            // Chunks are bounded by the fixed read buffer; callbacks are best effort and
            // must never interfere with process capture or completion.
            callback(label, &buffer[..bytes]);
        }
        output.push(&buffer[..bytes])?;
    }
    output.finish()
}

fn copy_capture(
    destination: &mut fs::File,
    hasher: &mut Sha256,
    output: &CapturedOutput,
) -> std::io::Result<()> {
    if let Some(path) = output.path.as_ref() {
        let mut source = fs::File::open(path)?;
        let mut buffer = [0u8; 8192];
        loop {
            let bytes = source.read(&mut buffer)?;
            if bytes == 0 {
                break;
            }
            destination.write_all(&buffer[..bytes])?;
            hasher.update(&buffer[..bytes]);
        }
    } else {
        destination.write_all(&output.preview)?;
        hasher.update(&output.preview);
    }
    Ok(())
}

fn write_process_artifact(
    root: &Path,
    stdout: &CapturedOutput,
    stderr: &CapturedOutput,
) -> Result<String, BusinessError> {
    let directory = artifacts::artifact_directory(root);
    fs::create_dir_all(&directory)
        .map_err(|_| retryable("artifact_failed", "artifact directory could not be created"))?;
    let temporary = directory.join(format!(
        ".process-{}-{}.tmp",
        std::process::id(),
        OUTPUT_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    let mut temporary_guard = TemporaryArtifact::new(temporary.clone());
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .map_err(|_| retryable("artifact_failed", "artifact could not be created"))?;
    let mut hasher = Sha256::new();
    copy_capture(&mut file, &mut hasher, stdout).map_err(|_| {
        retryable(
            "artifact_failed",
            "process output artifact could not be written",
        )
    })?;
    file.write_all(b"\n--- stderr ---\n").map_err(|_| {
        retryable(
            "artifact_failed",
            "process output artifact could not be written",
        )
    })?;
    hasher.update(b"\n--- stderr ---\n");
    copy_capture(&mut file, &mut hasher, stderr).map_err(|_| {
        retryable(
            "artifact_failed",
            "process output artifact could not be written",
        )
    })?;
    file.flush().map_err(|_| {
        retryable(
            "artifact_failed",
            "process output artifact could not be flushed",
        )
    })?;
    drop(file);
    let id = format!("{:x}", hasher.finalize());
    let destination = directory.join(format!("{id}.bin"));
    if destination.exists() {
        let _ = fs::remove_file(&temporary);
        temporary_guard.committed = true;
    } else if let Err(error) = fs::rename(&temporary, &destination) {
        return Err(retryable(
            "artifact_failed",
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                "artifact already exists"
            } else {
                "artifact could not be finalized"
            },
        ));
    } else {
        temporary_guard.committed = true;
    }
    Ok(id)
}

struct TemporaryArtifact {
    path: PathBuf,
    committed: bool,
}

impl TemporaryArtifact {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            committed: false,
        }
    }
}

impl Drop for TemporaryArtifact {
    fn drop(&mut self) {
        if !self.committed {
            let _ = fs::remove_file(&self.path);
        }
    }
}

pub(crate) fn run(
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
    args: &ProcessArguments,
    cancellation: Option<&AtomicBool>,
    output_callback: Option<ProcessOutputCallback>,
) -> Result<Value, BusinessError> {
    let root = require_project(project_root)?;
    let (command, command_args) = command_arguments(args)?;
    let cwd = process_cwd(root, args)?;
    let timeout_ms = args.timeout_ms.unwrap_or(120_000).clamp(1, 600_000);
    let operation_id = journal_id(
        args.idempotency_key.as_deref(),
        args.operation_id.as_deref(),
    )
    .unwrap_or_else(|| sha256_hex(format!("{}:{}", command, now_string()).as_bytes()));
    let mut child = configure_command(Command::new(&command), args)
        .args(&command_args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(process_start_failure)?;
    let stdout_callback = output_callback.clone();
    let stderr_callback = output_callback;
    let stdout = child
        .stdout
        .take()
        .map(|reader| std::thread::spawn(|| collect_output(reader, "stdout", stdout_callback)));
    let stderr = child
        .stderr
        .take()
        .map(|reader| std::thread::spawn(|| collect_output(reader, "stderr", stderr_callback)));
    let started = std::time::Instant::now();
    let mut timed_out = false;
    let mut cancelled = false;
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|_| retryable("process_status_failed", "process status could not be read"))?
        {
            break status;
        }
        if cancellation.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
            cancelled = true;
            terminate_process_tree(&mut child);
            break child.wait().map_err(|_| {
                retryable("process_status_failed", "process status could not be read")
            })?;
        }
        if started.elapsed().as_millis() >= u128::from(timeout_ms) {
            timed_out = true;
            terminate_process_tree(&mut child);
            break child.wait().map_err(|_| {
                retryable("process_status_failed", "process status could not be read")
            })?;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    reap_process_group(&child);
    let drain_deadline = std::time::Instant::now() + OUTPUT_DRAIN_TIMEOUT;
    let out = join_output(stdout, drain_deadline)
        .and_then(Result::ok)
        .transpose()
        .map_err(|_| retryable("process_output_failed", "stdout could not be collected"))?
        .unwrap_or(CapturedOutput {
            preview: Vec::new(),
            total_bytes: 0,
            path: None,
        });
    let err = join_output(stderr, drain_deadline)
        .and_then(Result::ok)
        .transpose()
        .map_err(|_| retryable("process_output_failed", "stderr could not be collected"))?
        .unwrap_or(CapturedOutput {
            preview: Vec::new(),
            total_bytes: 0,
            path: None,
        });
    let artifact_id = if out.total_bytes + err.total_bytes > PREVIEW_BYTES {
        checkpoint_root
            .map(|root| write_process_artifact(root, &out, &err))
            .transpose()?
    } else {
        None
    };
    let success = status.success() && !cancelled && !timed_out;
    let status_name = if cancelled {
        "cancelled"
    } else if timed_out {
        "timed_out"
    } else if success {
        "completed"
    } else {
        "failed"
    };
    let mut result = json!({"operation_id": operation_id, "status": status_name, "exit_code": status.code(), "success": success, "stdout_base64": STANDARD.encode(&out.preview), "stderr_base64": STANDARD.encode(&err.preview), "truncated": out.total_bytes + err.total_bytes > PREVIEW_BYTES, "sandbox": {"profile": args.sandbox_profile.as_deref().unwrap_or("project-default"), "network": "not_enforced", "environment": "filtered", "os_isolation": false}});
    if let Some(id) = artifact_id {
        result["artifact_id"] = json!(id);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{command_arguments, process_cwd, process_start_failure, run};
    use crate::arguments::ProcessArguments;
    use base64::{engine::general_purpose::STANDARD, Engine};
    use std::sync::atomic::AtomicBool;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn structured_process_arguments_remain_separate() {
        let process_args = ProcessArguments {
            program: "git".into(),
            args: vec!["status".into(), "--short".into()],
            cwd: None,
            env: Default::default(),
            timeout_ms: None,
            sandbox_profile: None,
            idempotency_key: None,
            operation_id: None,
        };
        let (program, args) = command_arguments(&process_args).unwrap();
        assert_eq!(program, "git");
        assert_eq!(args, ["status", "--short"]);
    }

    #[test]
    fn missing_executable_has_a_stable_error_code() {
        let failure = process_start_failure(std::io::Error::from(std::io::ErrorKind::NotFound));
        assert_eq!(failure.code, "process_executable_not_found");
    }

    #[test]
    fn absolute_working_directory_inside_project_is_allowed() {
        let root = std::env::temp_dir().join(format!(
            "suncode-process-cwd-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let nested = root.join("nested");
        std::fs::create_dir_all(&nested).unwrap();
        let root = root.canonicalize().unwrap();
        let nested = nested.canonicalize().unwrap();
        let params = ProcessArguments {
            program: "echo".into(),
            args: Vec::new(),
            cwd: Some(nested.to_string_lossy().into_owned()),
            env: Default::default(),
            timeout_ms: None,
            sandbox_profile: None,
            idempotency_key: None,
            operation_id: None,
        };

        assert_eq!(process_cwd(&root, &params).unwrap(), nested);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn absolute_working_directory_outside_project_is_denied() {
        let root = std::env::temp_dir().join(format!(
            "suncode-process-cwd-root-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let outside = std::env::temp_dir().join(format!(
            "suncode-process-cwd-outside-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        let root = root.canonicalize().unwrap();
        let params = ProcessArguments {
            program: "echo".into(),
            args: Vec::new(),
            cwd: Some(outside.to_string_lossy().into_owned()),
            env: Default::default(),
            timeout_ms: None,
            sandbox_profile: None,
            idempotency_key: None,
            operation_id: None,
        };

        assert_eq!(
            process_cwd(&root, &params).unwrap_err().code,
            "scope_denied"
        );
        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_dir_all(outside).unwrap();
    }

    #[test]
    fn platform_shell_process_starts_and_captures_output() {
        let root = std::env::temp_dir().join(format!(
            "suncode-process-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let root = root.canonicalize().unwrap();
        let checkpoint = root.join("checkpoints");
        std::fs::create_dir_all(&checkpoint).unwrap();
        #[cfg(target_os = "windows")]
        let params = ProcessArguments {
            program: "powershell.exe".into(),
            args: vec![
                "-NoLogo".into(),
                "-NoProfile".into(),
                "-NonInteractive".into(),
                "-Command".into(),
                "Write-Output suncode-ready".into(),
            ],
            cwd: None,
            env: Default::default(),
            timeout_ms: None,
            sandbox_profile: None,
            idempotency_key: None,
            operation_id: None,
        };
        #[cfg(not(target_os = "windows"))]
        let params = ProcessArguments {
            program: "/bin/sh".into(),
            args: vec!["-lc".into(), "printf suncode-ready".into()],
            cwd: None,
            env: Default::default(),
            timeout_ms: None,
            sandbox_profile: None,
            idempotency_key: None,
            operation_id: None,
        };
        let result = run(Some(&root), Some(&checkpoint), &params, None, None).unwrap();
        assert_eq!(result["success"], true);
        let output = STANDARD
            .decode(result["stdout_base64"].as_str().unwrap())
            .unwrap();
        assert!(String::from_utf8(output).unwrap().contains("suncode-ready"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    fn shell_test_root(label: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "suncode-process-{label}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        root.canonicalize().unwrap()
    }

    #[cfg(unix)]
    fn shell(script: &str) -> ProcessArguments {
        ProcessArguments {
            program: "/bin/sh".into(),
            args: vec!["-c".into(), script.into()],
            cwd: None,
            env: Default::default(),
            timeout_ms: Some(20_000),
            sandbox_profile: None,
            idempotency_key: None,
            operation_id: None,
        }
    }

    #[cfg(unix)]
    #[test]
    fn background_descendant_does_not_hold_the_call_open() {
        let root = shell_test_root("background");
        let started = std::time::Instant::now();
        let result = run(
            Some(&root),
            None,
            &shell("sleep 30 & printf done"),
            None,
            None,
        )
        .unwrap();
        assert!(started.elapsed() < std::time::Duration::from_secs(10));
        assert_eq!(result["status"], "completed");
        let output = STANDARD
            .decode(result["stdout_base64"].as_str().unwrap())
            .unwrap();
        assert_eq!(output, b"done");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn stdin_is_not_inherited() {
        let root = shell_test_root("stdin");
        // `cat` exits immediately on a null stdin instead of waiting for input.
        let result = run(Some(&root), None, &shell("cat; printf eof"), None, None).unwrap();
        assert_eq!(result["status"], "completed");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn non_zero_exit_is_reported_as_failed_status() {
        let root = std::env::temp_dir().join(format!(
            "suncode-process-failure-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let root = root.canonicalize().unwrap();
        let checkpoint = root.join("checkpoints");
        std::fs::create_dir_all(&checkpoint).unwrap();
        #[cfg(target_os = "windows")]
        let params = ProcessArguments {
            program: "cmd.exe".into(),
            args: vec!["/C".into(), "exit".into(), "7".into()],
            cwd: None,
            env: Default::default(),
            timeout_ms: None,
            sandbox_profile: None,
            idempotency_key: None,
            operation_id: None,
        };
        #[cfg(not(target_os = "windows"))]
        let params = ProcessArguments {
            program: "/bin/sh".into(),
            args: vec!["-lc".into(), "exit 7".into()],
            cwd: None,
            env: Default::default(),
            timeout_ms: None,
            sandbox_profile: None,
            idempotency_key: None,
            operation_id: None,
        };
        let result = run(Some(&root), Some(&checkpoint), &params, None, None).unwrap();
        assert_eq!(result["status"], "failed");
        assert_eq!(result["success"], false);
        assert_eq!(result["exit_code"], 7);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn large_output_is_drained_and_saved_as_an_artifact() {
        let root = std::env::temp_dir().join(format!(
            "suncode-process-large-output-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let root = root.canonicalize().unwrap();
        let checkpoint = root.join("checkpoints");
        std::fs::create_dir_all(&checkpoint).unwrap();
        #[cfg(target_os = "windows")]
        let params = ProcessArguments {
            program: "powershell.exe".into(),
            args: vec![
                "-NoLogo".into(),
                "-NoProfile".into(),
                "-NonInteractive".into(),
                "-Command".into(),
                "[Console]::OpenStandardOutput().Write((New-Object byte[] 1048576), 0, 1048576)"
                    .into(),
            ],
            cwd: None,
            env: Default::default(),
            timeout_ms: None,
            sandbox_profile: None,
            idempotency_key: None,
            operation_id: None,
        };
        #[cfg(not(target_os = "windows"))]
        let params = ProcessArguments {
            program: "/bin/sh".into(),
            args: vec!["-lc".into(), "head -c 1048576 /dev/zero".into()],
            cwd: None,
            env: Default::default(),
            timeout_ms: None,
            sandbox_profile: None,
            idempotency_key: None,
            operation_id: None,
        };
        let result = run(Some(&root), Some(&checkpoint), &params, None, None).unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["truncated"], true);
        assert!(result["artifact_id"].as_str().is_some());
        let artifact_id = result["artifact_id"].as_str().unwrap();
        let artifact =
            super::artifacts::artifact_directory(&checkpoint).join(format!("{artifact_id}.bin"));
        assert!(artifact.exists());
        assert!(std::fs::metadata(artifact).unwrap().len() > 1024 * 1024);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cancellation_terminates_a_running_process() {
        let root = std::env::temp_dir().join(format!(
            "suncode-process-cancel-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let root = root.canonicalize().unwrap();
        let checkpoint = root.join("checkpoints");
        std::fs::create_dir_all(&checkpoint).unwrap();
        #[cfg(target_os = "windows")]
        let params = ProcessArguments {
            program: "powershell.exe".into(),
            args: vec![
                "-NoLogo".into(),
                "-NoProfile".into(),
                "-NonInteractive".into(),
                "-Command".into(),
                "Start-Sleep -Seconds 30".into(),
            ],
            cwd: None,
            env: Default::default(),
            timeout_ms: None,
            sandbox_profile: None,
            idempotency_key: None,
            operation_id: None,
        };
        #[cfg(not(target_os = "windows"))]
        let params = ProcessArguments {
            program: "/bin/sh".into(),
            args: vec!["-lc".into(), "sleep 30".into()],
            cwd: None,
            env: Default::default(),
            timeout_ms: None,
            sandbox_profile: None,
            idempotency_key: None,
            operation_id: None,
        };
        let cancellation = AtomicBool::new(true);
        let result = run(
            Some(&root),
            Some(&checkpoint),
            &params,
            Some(&cancellation),
            None,
        )
        .unwrap();
        assert_eq!(result["status"], "cancelled");
        assert_eq!(result["success"], false);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_processes_use_the_no_window_creation_flag() {
        assert_eq!(super::CREATE_NO_WINDOW, 0x0800_0000);
    }
}
