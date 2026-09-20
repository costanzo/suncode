use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};

use axum::{extract::State, http::header, response::IntoResponse, routing::post, Json, Router};
use suncode_sdk::AsyncAgentSdk;

static ENVIRONMENT: OnceLock<Mutex<()>> = OnceLock::new();

const TEXT_RESPONSE: &str = concat!(
    "data: {\"id\":\"chatcmpl-cli\",\"choices\":[{\"delta\":{\"content\":\"hello\"}}]}\n\n",
    "data: {\"id\":\"chatcmpl-cli\",\"choices\":[{\"delta\":{\"content\":\" world\"},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":3,\"completion_tokens\":2,\"total_tokens\":5}}\n\n",
    "data: [DONE]\n\n"
);

const APPROVAL_RESPONSE: &str = concat!(
    "data: {\"id\":\"chatcmpl-approval\",\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"write-call\",\"function\":{\"name\":\"write\",\"arguments\":\"{\\\"path\\\":\\\"README.md\\\",\\\"content\\\":\\\"updated\\\"}\"}}]},\"finish_reason\":\"tool_calls\"}]}\n\n",
    "data: [DONE]\n\n"
);

const QUESTION_RESPONSE: &str = concat!(
    "data: {\"id\":\"chatcmpl-question\",\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"question-call\",\"function\":{\"name\":\"question\",\"arguments\":\"{\\\"questions\\\":[{\\\"question\\\":\\\"Choose a mode\\\",\\\"header\\\":\\\"Mode\\\",\\\"options\\\":[{\\\"label\\\":\\\"Fast\\\",\\\"description\\\":\\\"Quick\\\"}],\\\"custom\\\":false}]}\"}}]},\"finish_reason\":\"tool_calls\"}]}\n\n",
    "data: [DONE]\n\n"
);

#[derive(Clone)]
struct MockProviderState {
    requests: Arc<Mutex<Vec<serde_json::Value>>>,
    response: &'static str,
}

type MockProvider = (
    String,
    tokio::sync::oneshot::Sender<()>,
    std::thread::JoinHandle<()>,
    Arc<Mutex<Vec<serde_json::Value>>>,
);

fn command(data_directory: &std::path::Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_suncode"));
    command
        .env("SUNCODE_DATA_DIRECTORY", data_directory)
        .env("SUNCODE_USER_ID", "cli-integration-test")
        .env("SUNCODE_NON_INTERACTIVE", "false")
        .env_remove("SUNCODE_DATABASE_PATH")
        .env_remove("SUNCODE_OUTPUT")
        .env_remove("SUNCODE_COLOR")
        .env_remove("SUNCODE_MODEL")
        .env_remove("SUNCODE_REASONING_EFFORT");
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
fn run_without_a_prompt_source_is_an_argument_error() {
    let directory = tempfile::tempdir().unwrap();
    let output = command(directory.path()).arg("run").output().unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(!directory.path().join("data/sqlite/agent.sqlite3").exists());
}

#[test]
fn run_streams_typed_events_and_returns_a_completed_result() {
    let _guard = ENVIRONMENT.get_or_init(|| Mutex::new(())).lock().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let project = directory.path().join("project");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(project.join("README.md"), "sample\n").unwrap();
    let (endpoint, shutdown, server, _requests) = mock_provider();

    std::env::set_var("SUNCODE_DATA_DIRECTORY", directory.path());
    std::env::remove_var("SUNCODE_DATABASE_PATH");
    std::env::set_var("SUNCODE_NON_INTERACTIVE", "false");
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let sdk = AsyncAgentSdk::open_default("cli-integration-test")
            .await
            .unwrap();
        sdk.set_credential("deepseek", "test-key").unwrap();
        sdk.set_provider_endpoint("deepseek", &endpoint).unwrap();
        sdk.set_credential("openai", "test-key").unwrap();
        sdk.set_provider_endpoint("openai", &endpoint).unwrap();
        sdk.shutdown().await.unwrap();
    });
    std::env::remove_var("SUNCODE_DATA_DIRECTORY");
    std::env::remove_var("SUNCODE_NON_INTERACTIVE");

    let output = command(directory.path())
        .args([
            "--output",
            "jsonl",
            "run",
            project.to_str().unwrap(),
            "--prompt",
            "Say hello",
            "--model",
            "deepseek-v4-flash",
        ])
        .output()
        .unwrap();

    let mut text_command = command(directory.path());
    text_command
        .env("SUNCODE_MODEL", "gpt-5.6-sol")
        .env("SUNCODE_REASONING_EFFORT", "medium")
        .args(["run", project.to_str().unwrap(), "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = text_command.spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"Say hello from stdin")
        .unwrap();
    let text_output = child.wait_with_output().unwrap();

    let _ = shutdown.send(());
    server.join().unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let values = stdout
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert!(
        values
            .iter()
            .any(|value| value["type"] == "assistant.delta"),
        "{stdout}"
    );
    let result = values
        .iter()
        .find(|value| value["type"] == "run.result")
        .unwrap();
    assert_eq!(result["data"]["response"]["status"], "completed");
    assert_eq!(
        result["data"]["response"]["message"]["content"][0]["text"],
        "hello world"
    );
    assert!(
        text_output.status.success(),
        "{}",
        String::from_utf8_lossy(&text_output.stderr)
    );
    assert_eq!(
        String::from_utf8(text_output.stdout).unwrap(),
        "hello world\n"
    );
    let text_stderr = String::from_utf8(text_output.stderr).unwrap();
    assert!(text_stderr.contains("turn: completed"));
    assert!(!text_stderr.contains("hello world"));
}

#[test]
fn session_list_and_archive_use_the_sdk_lifecycle() {
    let _guard = ENVIRONMENT.get_or_init(|| Mutex::new(())).lock().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let project_path = directory.path().join("project");
    std::fs::create_dir_all(&project_path).unwrap();

    std::env::set_var("SUNCODE_DATA_DIRECTORY", directory.path());
    std::env::remove_var("SUNCODE_DATABASE_PATH");
    std::env::set_var("SUNCODE_NON_INTERACTIVE", "false");
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let archived_session_id = runtime.block_on(async {
        let sdk = AsyncAgentSdk::open_default("cli-integration-test")
            .await
            .unwrap();
        let project = sdk
            .open_project(project_path.to_str().unwrap(), None)
            .unwrap();
        let first = sdk
            .create_session(&project.project_id, Some("First session"), None)
            .unwrap();
        sdk.create_session(&project.project_id, Some("Second session"), None)
            .unwrap();
        sdk.shutdown().await.unwrap();
        first.session_id
    });
    std::env::remove_var("SUNCODE_DATA_DIRECTORY");
    std::env::remove_var("SUNCODE_NON_INTERACTIVE");

    let list_output = command(directory.path())
        .args([
            "--output",
            "jsonl",
            "session",
            "list",
            project_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        list_output.status.success(),
        "{}",
        String::from_utf8_lossy(&list_output.stderr)
    );
    let list: serde_json::Value = serde_json::from_slice(&list_output.stdout).unwrap();
    assert_eq!(list["type"], "session.list");
    assert_eq!(list["data"]["sessions"].as_array().unwrap().len(), 2);

    let archive_output = command(directory.path())
        .args([
            "--output",
            "jsonl",
            "session",
            "archive",
            &archived_session_id,
        ])
        .output()
        .unwrap();
    assert!(
        archive_output.status.success(),
        "{}",
        String::from_utf8_lossy(&archive_output.stderr)
    );
    let archived: serde_json::Value = serde_json::from_slice(&archive_output.stdout).unwrap();
    assert_eq!(archived["type"], "session.archived");
    assert_eq!(archived["data"]["sessionId"], archived_session_id);
    assert_eq!(archived["data"]["status"], "archived");
    assert!(archived["data"]["archivedAt"].is_string());

    let listed_again = command(directory.path())
        .args([
            "--output",
            "jsonl",
            "session",
            "list",
            project_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(listed_again.status.success());
    let listed_again: serde_json::Value = serde_json::from_slice(&listed_again.stdout).unwrap();
    let archived = listed_again["data"]["sessions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|session| session["sessionId"] == archived_session_id)
        .unwrap();
    assert_eq!(archived["status"], "archived");
}

#[test]
fn session_resume_reopens_and_uses_durable_conversation_context() {
    let _guard = ENVIRONMENT.get_or_init(|| Mutex::new(())).lock().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let project_path = directory.path().join("project");
    std::fs::create_dir_all(&project_path).unwrap();
    let (endpoint, shutdown, server, requests) = mock_provider();

    std::env::set_var("SUNCODE_DATA_DIRECTORY", directory.path());
    std::env::remove_var("SUNCODE_DATABASE_PATH");
    std::env::set_var("SUNCODE_NON_INTERACTIVE", "false");
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let session_id = runtime.block_on(async {
        let sdk = AsyncAgentSdk::open_default("cli-integration-test")
            .await
            .unwrap();
        sdk.set_credential("deepseek", "test-key").unwrap();
        sdk.set_provider_endpoint("deepseek", &endpoint).unwrap();
        let project = sdk
            .open_project(project_path.to_str().unwrap(), None)
            .unwrap();
        let session = sdk
            .create_session(
                &project.project_id,
                Some("Resume session"),
                Some("deepseek-v4-flash"),
            )
            .unwrap();
        sdk.submit_turn(
            &session.session_id,
            "Remember alpha",
            "initial-turn",
            None,
            None,
        )
        .await
        .unwrap();
        sdk.archive_session(&session.session_id).unwrap();
        sdk.shutdown().await.unwrap();
        session.session_id
    });
    std::env::remove_var("SUNCODE_DATA_DIRECTORY");
    std::env::remove_var("SUNCODE_NON_INTERACTIVE");
    requests.lock().unwrap().clear();

    let output = command(directory.path())
        .args([
            "--output",
            "jsonl",
            "session",
            "resume",
            &session_id,
            "--prompt",
            "What did I ask?",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let values = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    let result = values
        .iter()
        .find(|value| value["type"] == "session.resume.result")
        .unwrap();
    assert_eq!(result["data"]["response"]["status"], "completed");
    let requests_after_prompt = requests.lock().unwrap().clone();
    assert_eq!(requests_after_prompt.len(), 1);
    let request_text = serde_json::to_string(&requests_after_prompt[0]).unwrap();
    assert!(request_text.contains("Remember alpha"));
    assert!(request_text.contains("hello world"));
    assert!(request_text.contains("What did I ask?"));

    let mut stdin_command = command(directory.path());
    stdin_command
        .args(["session", "resume", &session_id, "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = stdin_command.spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"Continue from stdin")
        .unwrap();
    let stdin_output = child.wait_with_output().unwrap();
    assert!(
        stdin_output.status.success(),
        "{}",
        String::from_utf8_lossy(&stdin_output.stderr)
    );
    assert_eq!(
        String::from_utf8(stdin_output.stdout).unwrap(),
        "hello world\n"
    );

    let list_output = command(directory.path())
        .args([
            "--output",
            "jsonl",
            "session",
            "list",
            project_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let listed: serde_json::Value = serde_json::from_slice(&list_output.stdout).unwrap();
    let resumed = listed["data"]["sessions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|session| session["sessionId"] == session_id)
        .unwrap();
    assert_eq!(resumed["status"], "active");

    let _ = shutdown.send(());
    server.join().unwrap();
}

#[test]
fn session_resume_fails_closed_when_an_approval_is_pending() {
    let _guard = ENVIRONMENT.get_or_init(|| Mutex::new(())).lock().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let project_path = directory.path().join("project");
    std::fs::create_dir_all(&project_path).unwrap();
    std::fs::write(project_path.join("README.md"), "original\n").unwrap();
    let (endpoint, shutdown, server, requests) = mock_provider_with_response(APPROVAL_RESPONSE);

    std::env::set_var("SUNCODE_DATA_DIRECTORY", directory.path());
    std::env::remove_var("SUNCODE_DATABASE_PATH");
    std::env::set_var("SUNCODE_NON_INTERACTIVE", "false");
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let sdk = AsyncAgentSdk::open_default("cli-integration-test")
            .await
            .unwrap();
        sdk.set_credential("deepseek", "test-key").unwrap();
        sdk.set_provider_endpoint("deepseek", &endpoint).unwrap();
        sdk.shutdown().await.unwrap();
    });
    std::env::remove_var("SUNCODE_DATA_DIRECTORY");
    std::env::remove_var("SUNCODE_NON_INTERACTIVE");

    let run_output = command(directory.path())
        .args([
            "--output",
            "jsonl",
            "run",
            project_path.to_str().unwrap(),
            "--prompt",
            "Update the file",
            "--model",
            "deepseek-v4-flash",
        ])
        .output()
        .unwrap();
    assert_eq!(run_output.status.code(), Some(4));
    let run_values = String::from_utf8(run_output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    let approval = run_values
        .iter()
        .find(|value| value["type"] == "approval.requested")
        .unwrap();
    let session_id = approval["session_id"].as_str().unwrap();

    let resume_output = command(directory.path())
        .args([
            "--output", "jsonl", "session", "resume", session_id, "--prompt", "Continue",
        ])
        .output()
        .unwrap();
    assert_eq!(resume_output.status.code(), Some(4));
    let error: serde_json::Value = serde_json::from_slice(&resume_output.stdout).unwrap();
    assert_eq!(error["type"], "error");
    assert_eq!(error["data"]["code"], "approval_required");
    assert_eq!(requests.lock().unwrap().len(), 1);
    assert_eq!(
        std::fs::read_to_string(project_path.join("README.md")).unwrap(),
        "original\n"
    );

    let _ = shutdown.send(());
    server.join().unwrap();
}

#[test]
fn session_resume_fails_closed_when_a_question_is_pending() {
    let _guard = ENVIRONMENT.get_or_init(|| Mutex::new(())).lock().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let project_path = directory.path().join("project");
    std::fs::create_dir_all(&project_path).unwrap();
    let (endpoint, shutdown, server, requests) = mock_provider_with_response(QUESTION_RESPONSE);

    std::env::set_var("SUNCODE_DATA_DIRECTORY", directory.path());
    std::env::remove_var("SUNCODE_DATABASE_PATH");
    std::env::set_var("SUNCODE_NON_INTERACTIVE", "false");
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let sdk = AsyncAgentSdk::open_default("cli-integration-test")
            .await
            .unwrap();
        sdk.set_credential("deepseek", "test-key").unwrap();
        sdk.set_provider_endpoint("deepseek", &endpoint).unwrap();
        sdk.shutdown().await.unwrap();
    });
    std::env::remove_var("SUNCODE_DATA_DIRECTORY");
    std::env::remove_var("SUNCODE_NON_INTERACTIVE");

    let run_output = command(directory.path())
        .args([
            "--output",
            "jsonl",
            "run",
            project_path.to_str().unwrap(),
            "--prompt",
            "Ask me",
            "--model",
            "deepseek-v4-flash",
        ])
        .output()
        .unwrap();
    assert_eq!(run_output.status.code(), Some(4));
    let run_values = String::from_utf8(run_output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    let question = run_values
        .iter()
        .find(|value| value["type"] == "question.asked")
        .unwrap();
    let session_id = question["session_id"].as_str().unwrap();

    let resume_output = command(directory.path())
        .args([
            "--output", "jsonl", "session", "resume", session_id, "--prompt", "Continue",
        ])
        .output()
        .unwrap();
    assert_eq!(resume_output.status.code(), Some(4));
    let error: serde_json::Value = serde_json::from_slice(&resume_output.stdout).unwrap();
    assert_eq!(error["data"]["code"], "question_required");
    assert_eq!(requests.lock().unwrap().len(), 1);

    let _ = shutdown.send(());
    server.join().unwrap();
}

fn mock_provider() -> MockProvider {
    mock_provider_with_response(TEXT_RESPONSE)
}

fn mock_provider_with_response(response: &'static str) -> MockProvider {
    let (address_sender, address_receiver) = std::sync::mpsc::channel();
    let (shutdown_sender, shutdown_receiver) = tokio::sync::oneshot::channel();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let state = MockProviderState {
        requests: requests.clone(),
        response,
    };
    let server = std::thread::spawn(move || {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
                address_sender.send(listener.local_addr().unwrap()).unwrap();
                axum::serve(
                    listener,
                    Router::new()
                        .route("/chat/completions", post(mock_chat))
                        .with_state(state),
                )
                .with_graceful_shutdown(async {
                    let _ = shutdown_receiver.await;
                })
                .await
                .unwrap();
            });
    });
    let address = address_receiver.recv().unwrap();
    (
        format!("http://{address}"),
        shutdown_sender,
        server,
        requests,
    )
}

async fn mock_chat(
    State(state): State<MockProviderState>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    state.requests.lock().unwrap().push(request);
    (
        [(header::CONTENT_TYPE, "text/event-stream")],
        state.response,
    )
}
