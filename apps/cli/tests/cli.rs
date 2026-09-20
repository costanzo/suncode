use std::process::{Command, Stdio};

fn command(data_directory: &std::path::Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_suncode"));
    command
        .env("SUNCODE_DATA_DIRECTORY", data_directory)
        .env("SUNCODE_USER_ID", "cli-integration-test")
        .env("SUNCODE_NON_INTERACTIVE", "false")
        .env_remove("SUNCODE_DATABASE_PATH")
        .env_remove("SUNCODE_OUTPUT")
        .env_remove("SUNCODE_COLOR");
    command
}

#[test]
fn doctor_emits_the_versioned_jsonl_envelope() {
    let directory = tempfile::tempdir().unwrap();
    let output = command(directory.path())
        .args(["--output", "jsonl", "doctor"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["type"], "doctor.result");
    assert_eq!(value["data"]["host_capabilities"]["browser_use"], false);
    assert_eq!(value["data"]["host_capabilities"]["computer_use"], false);
}

#[test]
fn models_runs_against_a_fresh_embedded_agent() {
    let directory = tempfile::tempdir().unwrap();
    let output = command(directory.path()).arg("models").output().unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("deepseek-v4-flash"));
    assert!(stdout.contains("claude-sonnet-5"));
}

#[test]
fn auth_set_fails_closed_without_an_interactive_terminal() {
    let directory = tempfile::tempdir().unwrap();
    let output = command(directory.path())
        .args(["auth", "set", "openai"])
        .stdin(Stdio::null())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("interactive terminal"));
}

#[test]
fn help_succeeds_without_opening_the_sdk() {
    let directory = tempfile::tempdir().unwrap();
    let output = command(directory.path()).arg("--help").output().unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("SunCode coding agent CLI"));
    assert!(!directory.path().join("data/sqlite/agent.sqlite3").exists());
}

#[test]
fn unimplemented_conversation_command_is_an_argument_error() {
    let directory = tempfile::tempdir().unwrap();
    let output = command(directory.path()).arg("run").output().unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(!directory.path().join("data/sqlite/agent.sqlite3").exists());
}
