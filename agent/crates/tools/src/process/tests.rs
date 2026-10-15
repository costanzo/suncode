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
            "[Console]::OpenStandardOutput().Write((New-Object byte[] 1048576), 0, 1048576)".into(),
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
        crate::artifacts::artifact_directory(&checkpoint).join(format!("{artifact_id}.bin"));
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
    assert_eq!(super::spawn::CREATE_NO_WINDOW, 0x0800_0000);
}

#[cfg(unix)]
#[test]
fn generated_operation_ids_are_unique_and_output_is_marked_complete() {
    let root = shell_test_root("operation-id");
    let first = run(Some(&root), None, &shell("echo same"), None, None).unwrap();
    let second = run(Some(&root), None, &shell("echo same"), None, None).unwrap();
    assert_ne!(first["operation_id"], second["operation_id"]);
    assert_eq!(first["output_complete"], true);
    assert_eq!(first["truncated"], false);
    let _ = std::fs::remove_dir_all(root);
}
