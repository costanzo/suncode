use std::{collections::BTreeMap, path::PathBuf, time::Duration};
use suncode_browser::{verify_runtime_tree, RuntimeLayout, WorkerLaunch, WorkerProcess};

#[tokio::main]
async fn main() {
    let root = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .expect("usage: cargo run -p suncode-browser --example probe -- <runtime-directory>");
    verify_runtime_tree(&root).expect("bundled browser runtime integrity verification failed");
    let scratch = tempfile::tempdir().expect("probe scratch directory could not be created");
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
        request_timeout: Duration::from_secs(15),
        environment: BTreeMap::new(),
    })
    .await
    .expect("bundled browser worker probe failed");
    println!("{}", serde_json::to_string(&worker.hello).unwrap());
    worker.close().await;
}
