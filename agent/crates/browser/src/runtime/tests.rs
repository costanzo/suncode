//! End-to-end runtime tests against a real Chromium standing in for CEF.
//!
//! Ignored by default. Run with a Chromium executable, for example:
//! `SUNCODE_TEST_CHROMIUM=/path/to/Chromium cargo test -p suncode-browser -- --ignored`

use super::*;
use futures_util::StreamExt;
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

const PAGE: &str = r#"<!doctype html><title>Fixture</title>
<label for="email">Email</label><input id="email">
<input placeholder="Search &quot;docs&quot;">
<button data-testid="go" onclick="document.title='clicked'">Go</button>
<p id="out">ready</p>
<script>
document.getElementById('email').addEventListener('change', e => {
  document.getElementById('out').innerText = 'email=' + e.target.value;
});
</script>"#;

struct Chromium {
    child: Child,
    port: u16,
    _profile: tempfile::TempDir,
}

impl Drop for Chromium {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// Starts headless Chromium with no pages, like CEF before the host opens one.
fn launch_chromium() -> Option<Chromium> {
    let executable = std::env::var("SUNCODE_TEST_CHROMIUM").ok()?;
    let port = free_port();
    let profile = tempfile::tempdir().unwrap();
    let mut child = Command::new(executable)
        .args([
            "--headless=new",
            "--no-first-run",
            "--no-default-browser-check",
            "--disable-gpu",
            "--no-startup-window",
            &format!("--remote-debugging-port={port}"),
            &format!("--user-data-dir={}", profile.path().display()),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    for _ in 0..100 {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return Some(Chromium {
                child,
                port,
                _profile: profile,
            });
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = child.kill();
    let _ = child.wait();
    panic!("Chromium did not expose CDP on port {port}");
}

/// Opens a page the way the desktop host does, through a separate CDP client.
async fn open_host_page(port: u16, url: String) -> Result<(), BusinessError> {
    let host_error = |error: chromiumoxide::error::CdpError| {
        BusinessError::unavailable(format!("host could not open page: {error}"))
    };
    let (browser, mut handler) =
        chromiumoxide::Browser::connect(format!("http://127.0.0.1:{port}/"))
            .await
            .map_err(host_error)?;
    let driver = tokio::spawn(async move { while handler.next().await.is_some() {} });
    let result = browser.new_page(url).await.map(|_| ()).map_err(host_error);
    driver.abort();
    result
}

/// Serves `PAGE` for every request until the test process exits.
fn serve_fixture() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            let mut buffer = [0u8; 2048];
            let _ = stream.read(&mut buffer);
            let _ = write!(
                stream,
                "HTTP/1.0 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{PAGE}",
                PAGE.len()
            );
        }
    });
    url
}

/// Plays the desktop host: opens the project's Browser Use page on request.
fn runtime_with_host(
    chromium: &Chromium,
    data_dir: PathBuf,
) -> (BrowserRuntime, Arc<Mutex<usize>>) {
    let runtime = BrowserRuntime::with_cdp_port(data_dir, chromium.port, true, true);
    let starts = Arc::new(Mutex::new(0usize));
    let port = chromium.port;
    let counter = starts.clone();
    runtime.set_host_callback(Some(Arc::new(move |scope: &str| {
        *counter.lock().unwrap() += 1;
        let url = connection::browser_target_url(scope);
        tokio::runtime::Handle::current().block_on(open_host_page(port, url))
    })));
    (runtime, starts)
}

async fn call(
    runtime: &BrowserRuntime,
    name: &str,
    arguments: Value,
) -> Result<Value, BusinessError> {
    runtime
        .call("project-1", name, arguments, CancellationToken::new())
        .await
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires SUNCODE_TEST_CHROMIUM"]
async fn tools_drive_a_real_page() {
    let Some(chromium) = launch_chromium() else {
        return;
    };
    let data = tempfile::tempdir().unwrap();
    let (runtime, starts) = runtime_with_host(&chromium, data.path().to_path_buf());
    let fixture = serve_fixture();

    let opened = call(&runtime, "browser_open", json!({"url": fixture}))
        .await
        .unwrap();
    assert_eq!(opened["url"], fixture);
    assert_eq!(*starts.lock().unwrap(), 1);

    let snapshot = call(&runtime, "browser_snapshot", json!({})).await.unwrap();
    assert!(snapshot["text"].as_str().unwrap().contains("ready"));

    call(
        &runtime,
        "browser_fill",
        json!({"target": {"kind": "label", "value": "Email"}, "value": "a@b.test"}),
    )
    .await
    .unwrap();
    let snapshot = call(&runtime, "browser_snapshot", json!({})).await.unwrap();
    assert!(snapshot["text"]
        .as_str()
        .unwrap()
        .contains("email=a@b.test"));

    // Quoted attribute values must not break the selector.
    call(
        &runtime,
        "browser_click",
        json!({"target": {"kind": "placeholder", "value": "Search \"docs\""}}),
    )
    .await
    .unwrap();
    call(&runtime, "browser_press", json!({"key": "Tab"}))
        .await
        .unwrap();
    call(
        &runtime,
        "browser_click",
        json!({"target": {"kind": "test_id", "value": "go"}}),
    )
    .await
    .unwrap();

    let missing = call(
        &runtime,
        "browser_click",
        json!({"target": {"kind": "text", "value": "Nope"}}),
    )
    .await
    .unwrap_err();
    assert_eq!(missing.code, crate::error::CDP_EVALUATION_FAILED);

    let tabs = call(&runtime, "browser_tabs", json!({})).await.unwrap();
    assert_eq!(tabs["pages"].as_array().unwrap().len(), 1);

    let screenshot = call(&runtime, "browser_screenshot", json!({}))
        .await
        .unwrap();
    assert_eq!(screenshot["mime_type"], "image/png");
    assert!(screenshot["bytes"].as_u64().unwrap() > 0);

    let info = runtime.info(Some("project-1")).await.unwrap();
    assert_eq!(info.runtime_state, BrowserRuntimeState::Background);
    assert_eq!(info.active_page_count, 1);
    runtime.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires SUNCODE_TEST_CHROMIUM"]
async fn concurrent_starts_share_one_page() {
    let Some(chromium) = launch_chromium() else {
        return;
    };
    let data = tempfile::tempdir().unwrap();
    let (runtime, starts) = runtime_with_host(&chromium, data.path().to_path_buf());

    let results =
        futures_util::future::join_all((0..4).map(|_| runtime.start_project("project-1"))).await;
    assert!(results.iter().all(Result::is_ok));
    assert_eq!(*starts.lock().unwrap(), 1);
    runtime.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires SUNCODE_TEST_CHROMIUM"]
async fn reconnects_to_a_navigated_page_after_stop() {
    let Some(chromium) = launch_chromium() else {
        return;
    };
    let data = tempfile::tempdir().unwrap();
    let (runtime, starts) = runtime_with_host(&chromium, data.path().to_path_buf());
    let fixture = serve_fixture();

    call(&runtime, "browser_navigate", json!({"url": fixture}))
        .await
        .unwrap();
    runtime.stop("project-1").await.unwrap();

    // The page no longer shows its initial URL; the known target id finds it.
    let snapshot = call(&runtime, "browser_snapshot", json!({})).await.unwrap();
    assert_eq!(snapshot["url"], fixture);
    assert_eq!(*starts.lock().unwrap(), 1);
    runtime.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires SUNCODE_TEST_CHROMIUM"]
async fn cancellation_drops_the_session_and_shutdown_refuses_new_work() {
    let Some(chromium) = launch_chromium() else {
        return;
    };
    let data = tempfile::tempdir().unwrap();
    let (runtime, _) = runtime_with_host(&chromium, data.path().to_path_buf());
    runtime.start_project("project-1").await.unwrap();

    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let cancelled = runtime
        .call("project-1", "browser_tabs", json!({}), cancellation)
        .await
        .unwrap_err();
    assert_eq!(cancelled.code, "cancelled");

    runtime.shutdown().await;
    let refused = runtime.start_project("project-1").await.unwrap_err();
    assert_eq!(refused.code, "agent_shutting_down");
    let info = runtime.info(Some("project-1")).await.unwrap();
    assert_eq!(info.active_page_count, 0);
}
