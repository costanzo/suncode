//! Project-scoped process execution for the `bash` tool and host background processes.
//!
//! - `spawn`: argument bounds, working directory, filtered environment, spawn flags.
//! - `capture`: continuous output draining with a bounded preview.
//! - `artifact`: complete output persisted as a managed artifact.
//! - `tree`: process-tree termination and post-exit group reaping.
//! - `background`: long-lived host-owned processes.

mod artifact;
mod background;
mod capture;
mod spawn;
mod tree;

pub use background::{start_background, BackgroundProcess};

use crate::arguments::ProcessArguments;
use crate::context::ProcessOutputCallback;
use crate::error::retryable;
use crate::journal::journal_id;
use crate::scope::require_project;
use crate::support::{now_string, sha256_hex};
use artifact::write_process_artifact;
use base64::{engine::general_purpose::STANDARD, Engine};
use capture::{collect_output, join_output, CapturedOutput, OUTPUT_DRAIN_TIMEOUT};
use serde_json::{json, Value};
use spawn::{command_arguments, configure_command, process_cwd, process_start_failure};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use suncode_common::BusinessError;
use tree::{reap_process_group, terminate_process_tree};

/// Bytes of each stream kept inline; beyond this, output spills to a file.
const PREVIEW_BYTES: usize = 64 * 1024;
static OUTPUT_SEQUENCE: AtomicU64 = AtomicU64::new(1);

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
mod tests;
