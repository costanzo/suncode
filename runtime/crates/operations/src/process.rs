use super::{artifacts, journal_id, now_string, require_project, safe_relative_path, CoreFailure};
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
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

const PREVIEW_BYTES: usize = 64 * 1024;
static OUTPUT_SEQUENCE: AtomicU64 = AtomicU64::new(1);

fn command_arguments(params: &Value) -> Result<(String, Vec<String>), CoreFailure> {
    let program = params
        .get("program")
        .or_else(|| params.get("command"))
        .and_then(Value::as_str)
        .ok_or(CoreFailure {
            code: "invalid_arguments",
            message: "program is required",
            retryable: false,
        })?;
    if program.is_empty() || program.len() > 4096 {
        return Err(CoreFailure {
            code: "invalid_arguments",
            message: "program is invalid",
            retryable: false,
        });
    }
    let args = params
        .get("args")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .map(|value| {
                    value.as_str().map(str::to_string).ok_or(CoreFailure {
                        code: "invalid_arguments",
                        message: "process arguments must be strings",
                        retryable: false,
                    })
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();
    if args.len() > 256 {
        return Err(CoreFailure {
            code: "resource_limit",
            message: "too many process arguments",
            retryable: false,
        });
    }
    Ok((program.to_string(), args))
}

fn process_cwd(root: &Path, params: &Value) -> Result<std::path::PathBuf, CoreFailure> {
    let relative = params.get("cwd").and_then(Value::as_str).unwrap_or(".");
    let path = root
        .join(safe_relative_path(relative)?)
        .canonicalize()
        .map_err(|_| CoreFailure {
            code: "process_working_directory_unavailable",
            message: "working directory is unavailable",
            retryable: false,
        })?;
    if !path.starts_with(root) || !path.is_dir() {
        return Err(CoreFailure {
            code: "scope_denied",
            message: "working directory is outside the project",
            retryable: false,
        });
    }
    Ok(path)
}

fn configure_command(mut command: Command, params: &Value) -> Command {
    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW);
    #[cfg(unix)]
    command.process_group(0);
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
    ] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    if let Some(env) = params.get("env").and_then(Value::as_object) {
        for (key, value) in env {
            if key.len() <= 128
                && key
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '_')
            {
                if let Some(value) = value.as_str() {
                    command.env(key, value);
                }
            }
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

fn process_start_failure(error: std::io::Error) -> CoreFailure {
    match error.kind() {
        std::io::ErrorKind::NotFound => CoreFailure {
            code: "process_executable_not_found",
            message: "process executable could not be found",
            retryable: false,
        },
        std::io::ErrorKind::PermissionDenied => CoreFailure {
            code: "process_permission_denied",
            message: "process executable could not be started because permission was denied",
            retryable: false,
        },
        _ => CoreFailure {
            code: "process_start_failed",
            message: "process could not be started",
            retryable: false,
        },
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

fn collect_output(mut reader: impl Read, label: &'static str) -> std::io::Result<CapturedOutput> {
    let mut output = OutputCapture::new(label);
    let mut buffer = [0u8; 8192];
    loop {
        let bytes = reader.read(&mut buffer)?;
        if bytes == 0 {
            break;
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
) -> Result<String, CoreFailure> {
    let directory = artifacts::artifact_directory(root);
    fs::create_dir_all(&directory).map_err(|_| CoreFailure {
        code: "artifact_failed",
        message: "artifact directory could not be created",
        retryable: true,
    })?;
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
        .map_err(|_| CoreFailure {
            code: "artifact_failed",
            message: "artifact could not be created",
            retryable: true,
        })?;
    let mut hasher = Sha256::new();
    copy_capture(&mut file, &mut hasher, stdout).map_err(|_| CoreFailure {
        code: "artifact_failed",
        message: "process output artifact could not be written",
        retryable: true,
    })?;
    file.write_all(b"\n--- stderr ---\n")
        .map_err(|_| CoreFailure {
            code: "artifact_failed",
            message: "process output artifact could not be written",
            retryable: true,
        })?;
    hasher.update(b"\n--- stderr ---\n");
    copy_capture(&mut file, &mut hasher, stderr).map_err(|_| CoreFailure {
        code: "artifact_failed",
        message: "process output artifact could not be written",
        retryable: true,
    })?;
    file.flush().map_err(|_| CoreFailure {
        code: "artifact_failed",
        message: "process output artifact could not be flushed",
        retryable: true,
    })?;
    drop(file);
    let id = format!("{:x}", hasher.finalize());
    let destination = directory.join(format!("{id}.bin"));
    if destination.exists() {
        let _ = fs::remove_file(&temporary);
        temporary_guard.committed = true;
    } else if let Err(error) = fs::rename(&temporary, &destination) {
        return Err(CoreFailure {
            code: "artifact_failed",
            message: if error.kind() == std::io::ErrorKind::AlreadyExists {
                "artifact already exists"
            } else {
                "artifact could not be finalized"
            },
            retryable: true,
        });
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

pub(super) fn run(
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
    params: &Value,
    cancellation: Option<&AtomicBool>,
) -> Result<Value, CoreFailure> {
    let root = require_project(project_root)?;
    let (command, args) = command_arguments(params)?;
    let cwd = process_cwd(root, params)?;
    let timeout_ms = params
        .get("timeout_ms")
        .and_then(Value::as_u64)
        .unwrap_or(120_000)
        .clamp(1, 600_000);
    let operation_id = journal_id(params)
        .unwrap_or_else(|| super::sha256_hex(format!("{}:{}", command, now_string()).as_bytes()));
    let mut child = configure_command(Command::new(&command), params)
        .args(&args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(process_start_failure)?;
    let stdout = child
        .stdout
        .take()
        .map(|reader| std::thread::spawn(|| collect_output(reader, "stdout")));
    let stderr = child
        .stderr
        .take()
        .map(|reader| std::thread::spawn(|| collect_output(reader, "stderr")));
    let started = std::time::Instant::now();
    let mut timed_out = false;
    let mut cancelled = false;
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|_| CoreFailure {
            code: "process_status_failed",
            message: "process status could not be read",
            retryable: true,
        })? {
            break status;
        }
        if cancellation.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
            cancelled = true;
            terminate_process_tree(&mut child);
            break child.wait().map_err(|_| CoreFailure {
                code: "process_status_failed",
                message: "process status could not be read",
                retryable: true,
            })?;
        }
        if started.elapsed().as_millis() >= u128::from(timeout_ms) {
            timed_out = true;
            terminate_process_tree(&mut child);
            break child.wait().map_err(|_| CoreFailure {
                code: "process_status_failed",
                message: "process status could not be read",
                retryable: true,
            })?;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    let out = stdout
        .and_then(|thread| thread.join().ok())
        .transpose()
        .map_err(|_| CoreFailure {
            code: "process_output_failed",
            message: "stdout could not be collected",
            retryable: true,
        })?
        .unwrap_or(CapturedOutput {
            preview: Vec::new(),
            total_bytes: 0,
            path: None,
        });
    let err = stderr
        .and_then(|thread| thread.join().ok())
        .transpose()
        .map_err(|_| CoreFailure {
            code: "process_output_failed",
            message: "stderr could not be collected",
            retryable: true,
        })?
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
    let mut result = json!({"operation_id": operation_id, "status": if cancelled {"cancelled"} else if timed_out {"timed_out"} else {"completed"}, "exit_code": status.code(), "success": status.success() && !cancelled && !timed_out, "stdout_base64": STANDARD.encode(&out.preview), "stderr_base64": STANDARD.encode(&err.preview), "truncated": out.total_bytes + err.total_bytes > PREVIEW_BYTES, "sandbox": {"profile": params.get("sandbox_profile").and_then(Value::as_str).unwrap_or("project-default"), "network": "not_enforced", "environment": "filtered", "os_isolation": false}});
    if let Some(id) = artifact_id {
        result["artifact_id"] = json!(id);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{command_arguments, process_start_failure, run};
    use base64::{engine::general_purpose::STANDARD, Engine};
    use serde_json::json;
    use std::sync::atomic::AtomicBool;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn structured_process_arguments_remain_separate() {
        let (program, args) = command_arguments(&json!({
            "program": "git",
            "args": ["status", "--short"]
        }))
        .unwrap();
        assert_eq!(program, "git");
        assert_eq!(args, ["status", "--short"]);
    }

    #[test]
    fn legacy_command_is_one_executable_not_shell_text() {
        let (program, args) = command_arguments(&json!({"command": "git status"})).unwrap();
        assert_eq!(program, "git status");
        assert!(args.is_empty());
    }

    #[test]
    fn missing_executable_has_a_stable_error_code() {
        let failure = process_start_failure(std::io::Error::from(std::io::ErrorKind::NotFound));
        assert_eq!(failure.code, "process_executable_not_found");
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
        let params = json!({
            "program":"powershell.exe",
            "args":["-NoLogo","-NoProfile","-NonInteractive","-Command","Write-Output suncode-ready"]
        });
        #[cfg(not(target_os = "windows"))]
        let params = json!({"program":"/bin/sh","args":["-lc","printf suncode-ready"]});
        let result = run(Some(&root), Some(&checkpoint), &params, None).unwrap();
        assert_eq!(result["success"], true);
        let output = STANDARD
            .decode(result["stdout_base64"].as_str().unwrap())
            .unwrap();
        assert!(String::from_utf8(output).unwrap().contains("suncode-ready"));
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
        let params = json!({
            "program":"powershell.exe",
            "args":["-NoLogo","-NoProfile","-NonInteractive","-Command","[Console]::OpenStandardOutput().Write((New-Object byte[] 1048576), 0, 1048576)"]
        });
        #[cfg(not(target_os = "windows"))]
        let params = json!({"program":"/bin/sh","args":["-lc","head -c 1048576 /dev/zero"]});
        let result = run(Some(&root), Some(&checkpoint), &params, None).unwrap();
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
        let params = json!({
            "program":"powershell.exe",
            "args":["-NoLogo","-NoProfile","-NonInteractive","-Command","Start-Sleep -Seconds 30"]
        });
        #[cfg(not(target_os = "windows"))]
        let params = json!({"program":"/bin/sh","args":["-lc","sleep 30"]});
        let cancellation = AtomicBool::new(true);
        let result = run(Some(&root), Some(&checkpoint), &params, Some(&cancellation)).unwrap();
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
