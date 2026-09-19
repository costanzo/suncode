use serde_json::{json, Value};
use std::{collections::BTreeMap, path::PathBuf, time::Duration};
use suncode_browser::{verify_runtime_tree, RuntimeLayout, WorkerLaunch, WorkerProcess};

#[tokio::main]
async fn main() {
    let root = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .expect("usage: cargo run -p suncode-browser --example smoke -- <runtime-directory>")
        .canonicalize()
        .expect("runtime directory could not be resolved");
    verify_runtime_tree(&root).expect("bundled browser runtime integrity verification failed");
    let scratch = tempfile::tempdir().expect("smoke scratch directory could not be created");
    let worker = WorkerProcess::spawn(WorkerLaunch {
        layout: RuntimeLayout {
            node_path: if cfg!(windows) {
                root.join("node/node.exe")
            } else {
                root.join("node/bin/node")
            },
            worker_path: root.join("worker/index.mjs"),
            runtime_lock_path: root.join("runtime-lock.json"),
        },
        working_directory: scratch.path().join("runtime"),
        startup_timeout: Duration::from_secs(15),
        request_timeout: Duration::from_secs(60),
        environment: BTreeMap::from([(
            "PLAYWRIGHT_BROWSERS_PATH".into(),
            root.join("browsers").to_string_lossy().into_owned(),
        )]),
    })
    .await
    .expect("bundled browser worker probe failed");
    let _: Value = worker
        .request(
            "start",
            json!({
                "profilePath": scratch.path().join("profile"),
                "downloadsPath": scratch.path().join("downloads"),
                "headless": true
            }),
        )
        .await
        .expect("Chromium failed to start");
    let snapshot: Value = worker
        .request(
            "open",
            json!({"url":"https://example.com","timeoutMs":30000}),
        )
        .await
        .expect("fixture page failed to open");
    assert_eq!(
        snapshot.get("url").and_then(Value::as_str),
        Some("https://example.com/")
    );
    let failure = worker
        .request::<Value>(
            "click",
            json!({
                "pageId": snapshot.get("pageId"),
                "target":{"kind":"text","value":"secret-marker-that-must-not-be-echoed"},
                "timeoutMs":1000
            }),
        )
        .await
        .expect_err("missing locator should fail safely");
    assert_eq!(failure.code, "browser_action_timeout");
    assert_eq!(failure.message, "Browser operation timed out");
    println!("{}", serde_json::to_string(&snapshot).unwrap());
    worker.close().await;
}
