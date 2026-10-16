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
#[cfg(target_os = "windows")]
mod job;
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
use tree::ProcessTree;

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
    .unwrap_or_else(|| {
        let sequence = OUTPUT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let seed = format!(
            "{command}:{}:{}:{sequence}",
            now_string(),
            std::process::id()
        );
        sha256_hex(seed.as_bytes())
    });
    let child = configure_command(Command::new(&command), args)
        .args(&command_args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(process_start_failure)?;
    let mut tree = ProcessTree::new(child);
    let stdout_callback = output_callback.clone();
    let stderr_callback = output_callback;
    let stdout = tree
        .child
        .stdout
        .take()
        .map(|reader| std::thread::spawn(|| collect_output(reader, "stdout", stdout_callback)));
    let stderr = tree
        .child
        .stderr
        .take()
        .map(|reader| std::thread::spawn(|| collect_output(reader, "stderr", stderr_callback)));
    let started = std::time::Instant::now();
    let mut timed_out = false;
    let mut cancelled = false;
    let status =
        loop {
            if let Some(status) = tree.child.try_wait().map_err(|_| {
                retryable("process_status_failed", "process status could not be read")
            })? {
                break status;
            }
            if cancellation.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
                cancelled = true;
                tree.terminate();
                break tree.child.wait().map_err(|_| {
                    retryable("process_status_failed", "process status could not be read")
                })?;
            }
            if started.elapsed().as_millis() >= u128::from(timeout_ms) {
                timed_out = true;
                tree.terminate();
                break tree.child.wait().map_err(|_| {
                    retryable("process_status_failed", "process status could not be read")
                })?;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        };
    tree.reap();
    let drain_deadline = std::time::Instant::now() + OUTPUT_DRAIN_TIMEOUT;
    let (out, out_complete) = joined_output(join_output(stdout, drain_deadline), "stdout")?;
    let (err, err_complete) = joined_output(join_output(stderr, drain_deadline), "stderr")?;
    let output_complete = out_complete && err_complete && out.spill_complete && err.spill_complete;
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
    let mut result = json!({"operation_id": operation_id, "status": status_name, "exit_code": status.code(), "success": success, "stdout_base64": STANDARD.encode(&out.preview), "stderr_base64": STANDARD.encode(&err.preview), "truncated": out.total_bytes + err.total_bytes > PREVIEW_BYTES || !output_complete, "output_complete": output_complete, "sandbox": {"profile": args.sandbox_profile.as_deref().unwrap_or("project-default"), "network": "not_enforced", "environment": "filtered", "os_isolation": false}});
    if let Some(id) = artifact_id {
        result["artifact_id"] = json!(id);
    }
    Ok(result)
}

/// Resolves a joined reader. A reader still running at the drain deadline
/// (a descendant holds the pipe) yields empty, incomplete output; a reader
/// panic is an internal failure.
fn joined_output(
    joined: Option<std::thread::Result<CapturedOutput>>,
    label: &str,
) -> Result<(CapturedOutput, bool), BusinessError> {
    match joined {
        Some(Ok(output)) => Ok((output, true)),
        Some(Err(_)) => Err(retryable(
            "process_output_failed",
            format!("{label} could not be collected"),
        )),
        None => Ok((CapturedOutput::default(), false)),
    }
}

#[cfg(test)]
mod tests;
