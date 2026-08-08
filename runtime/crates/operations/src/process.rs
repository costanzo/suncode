use super::{
    artifacts, existing_file, journal_directory, journal_id, load_journal, now_string,
    require_project, safe_relative_path, save_journal, CoreFailure, PROCESSES,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};

fn command_arguments(params: &Value) -> Result<(String, Vec<String>), CoreFailure> {
    let command = params
        .get("command")
        .and_then(Value::as_str)
        .ok_or(CoreFailure {
            code: "invalid_arguments",
            message: "command is required",
            retryable: false,
        })?;
    if command.is_empty() || command.len() > 4096 {
        return Err(CoreFailure {
            code: "invalid_arguments",
            message: "command is invalid",
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
    Ok((command.to_string(), args))
}

fn process_cwd(root: &Path, params: &Value) -> Result<std::path::PathBuf, CoreFailure> {
    let relative = params.get("cwd").and_then(Value::as_str).unwrap_or(".");
    let path = root
        .join(safe_relative_path(relative)?)
        .canonicalize()
        .map_err(|_| CoreFailure {
            code: "path_unavailable",
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
    command.env_clear();
    for key in [
        "PATH",
        "HOME",
        "TMPDIR",
        "TMP",
        "TEMP",
        "SystemRoot",
        "ComSpec",
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

fn collect_bounded(mut reader: impl Read) -> Vec<u8> {
    let mut output = Vec::new();
    let mut buffer = [0u8; 8192];
    while let Ok(bytes) = reader.read(&mut buffer) {
        if bytes == 0 {
            break;
        }
        let remaining = 256 * 1024usize - output.len().min(256 * 1024);
        output.extend_from_slice(&buffer[..bytes.min(remaining)]);
        if output.len() >= 256 * 1024 {
            break;
        }
    }
    output
}

pub(super) fn run(
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
    params: &Value,
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
        .map_err(|_| CoreFailure {
            code: "process_start_failed",
            message: "process could not be started",
            retryable: false,
        })?;
    let stdout = child
        .stdout
        .take()
        .map(|reader| std::thread::spawn(|| collect_bounded(reader)));
    let stderr = child
        .stderr
        .take()
        .map(|reader| std::thread::spawn(|| collect_bounded(reader)));
    let started = std::time::Instant::now();
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|_| CoreFailure {
            code: "process_status_failed",
            message: "process status could not be read",
            retryable: true,
        })? {
            break status;
        }
        if started.elapsed().as_millis() >= u128::from(timeout_ms) {
            timed_out = true;
            let _ = child.kill();
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
        .unwrap_or_default();
    let err = stderr
        .and_then(|thread| thread.join().ok())
        .unwrap_or_default();
    let artifact_id = if out.len() + err.len() > 64 * 1024 {
        checkpoint_root.and_then(|root| {
            artifacts::write_artifact(
                root,
                &[out.as_slice(), b"\n--- stderr ---\n", err.as_slice()].concat(),
            )
            .ok()
        })
    } else {
        None
    };
    let mut result = json!({"operation_id": operation_id, "status": if timed_out {"timed_out"} else {"completed"}, "exit_code": status.code(), "success": status.success(), "stdout_base64": STANDARD.encode(&out[..out.len().min(64 * 1024)]), "stderr_base64": STANDARD.encode(&err[..err.len().min(64 * 1024)]), "truncated": out.len() > 64 * 1024 || err.len() > 64 * 1024, "sandbox": {"profile": params.get("sandbox_profile").and_then(Value::as_str).unwrap_or("project-default"), "network": "not_enforced", "environment": "filtered", "os_isolation": false}});
    if let Some(id) = artifact_id {
        result["artifact_id"] = json!(id);
    }
    Ok(result)
}

pub(super) fn start(project_root: Option<&Path>, params: &Value) -> Result<Value, CoreFailure> {
    let root = require_project(project_root)?;
    let (command, args) = command_arguments(params)?;
    let cwd = process_cwd(root, params)?;
    let operation_id = journal_id(params)
        .unwrap_or_else(|| super::sha256_hex(format!("{}:{}", command, now_string()).as_bytes()));
    let child = configure_command(Command::new(&command), params)
        .args(&args)
        .current_dir(cwd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| CoreFailure {
            code: "process_start_failed",
            message: "process could not be started",
            retryable: false,
        })?;
    let processes =
        PROCESSES.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    processes
        .lock()
        .map_err(|_| CoreFailure {
            code: "process_status_failed",
            message: "process table is unavailable",
            retryable: true,
        })?
        .insert(operation_id.clone(), child);
    Ok(
        json!({"operation_id": operation_id, "status": "running", "sandbox": {"profile": params.get("sandbox_profile").and_then(Value::as_str).unwrap_or("project-default"), "network": "not_enforced", "environment": "filtered", "os_isolation": false}}),
    )
}

pub(super) fn status(params: &Value) -> Result<Value, CoreFailure> {
    let id = params
        .get("operation_id")
        .and_then(Value::as_str)
        .ok_or(CoreFailure {
            code: "invalid_arguments",
            message: "operation_id is required",
            retryable: false,
        })?;
    let processes =
        PROCESSES.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    let mut table = processes.lock().map_err(|_| CoreFailure {
        code: "process_status_failed",
        message: "process table is unavailable",
        retryable: true,
    })?;
    let Some(child) = table.get_mut(id) else {
        return Ok(json!({"operation_id": id, "status": "unknown"}));
    };
    let status = child.try_wait().map_err(|_| CoreFailure {
        code: "process_status_failed",
        message: "process status could not be read",
        retryable: true,
    })?;
    if let Some(status) = status {
        table.remove(id);
        return Ok(
            json!({"operation_id": id, "status": "completed", "exit_code": status.code(), "success": status.success()}),
        );
    }
    Ok(json!({"operation_id": id, "status": "running"}))
}

pub(super) fn cancel(params: &Value) -> Result<Value, CoreFailure> {
    let id = params
        .get("operation_id")
        .and_then(Value::as_str)
        .ok_or(CoreFailure {
            code: "invalid_arguments",
            message: "operation_id is required",
            retryable: false,
        })?;
    let processes =
        PROCESSES.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    let mut table = processes.lock().map_err(|_| CoreFailure {
        code: "operation_cancel_failed",
        message: "process table is unavailable",
        retryable: true,
    })?;
    let Some(mut child) = table.remove(id) else {
        return Ok(json!({"operation_id": id, "status": "unknown", "confirmed": false}));
    };
    let killed = child.kill().is_ok();
    let _ = child.wait();
    Ok(
        json!({"operation_id": id, "status": if killed {"cancelled"} else {"unknown"}, "confirmed": killed}),
    )
}

pub(super) fn operation_status(
    checkpoint_root: Option<&Path>,
    params: &Value,
) -> Result<Value, CoreFailure> {
    let root = checkpoint_root.ok_or(CoreFailure {
        code: "operation_unavailable",
        message: "operation journal is not configured",
        retryable: false,
    })?;
    let key = journal_id(params).ok_or(CoreFailure {
        code: "invalid_arguments",
        message: "operation_id is required",
        retryable: false,
    })?;
    let Some(record) = load_journal(root, &key) else {
        return Ok(json!({"status": "unknown", "operation_id": key}));
    };
    Ok(serde_json::to_value(record)
        .unwrap_or_else(|_| json!({"status": "unknown", "operation_id": key})))
}

pub(super) fn reconcile(
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
    params: &Value,
) -> Result<Value, CoreFailure> {
    let root = require_project(project_root)?;
    let journal_root = checkpoint_root.ok_or(CoreFailure {
        code: "operation_unavailable",
        message: "operation journal is not configured",
        retryable: false,
    })?;
    let key = journal_id(params).ok_or(CoreFailure {
        code: "invalid_arguments",
        message: "operation_id is required",
        retryable: false,
    })?;
    let Some(mut record) = load_journal(journal_root, &key) else {
        return Ok(json!({"operation_id": key, "status": "unknown"}));
    };
    if record.status != "pending" {
        return Ok(json!({"operation_id": key, "status": record.status, "result": record.result}));
    }
    let observed = record
        .path
        .as_deref()
        .and_then(|path| {
            existing_file(root, path)
                .ok()
                .map(|(_, bytes)| super::sha256_hex(&bytes))
        })
        .or_else(|| record.path.as_deref().map(|_| super::sha256_hex(&[])));
    let status = if observed == record.post_image_sha256 {
        "succeeded"
    } else if observed == record.pre_image_sha256 {
        "not_started"
    } else {
        "unknown_completion"
    };
    record.status = status.to_string();
    record.updated_at = now_string();
    save_journal(journal_root, &record)?;
    Ok(json!({"operation_id": key, "status": status, "result": record.result}))
}

pub(super) fn recovery(
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
) -> Result<Value, CoreFailure> {
    let root = checkpoint_root.ok_or(CoreFailure {
        code: "recovery_unavailable",
        message: "core data directory is not configured",
        retryable: false,
    })?;
    let mut pending = Vec::new();
    if let Ok(entries) = fs::read_dir(journal_directory(root)) {
        for entry in entries.flatten() {
            if let Ok(record) = serde_json::from_slice::<super::JournalRecord>(
                &fs::read(entry.path()).unwrap_or_default(),
            ) {
                if record.status == "pending" {
                    pending.push(json!({"operation_id": record.operation_id, "operation": record.operation, "path": record.path}));
                }
            }
        }
    }
    Ok(
        json!({"project_configured": project_root.is_some(), "pending_operations": pending, "managed_artifacts": count_artifacts(root)}),
    )
}

fn count_artifacts(root: &Path) -> usize {
    fs::read_dir(artifacts::artifact_directory(root))
        .map(|entries| entries.flatten().count())
        .unwrap_or(0)
}
