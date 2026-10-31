//! Cross-tool dispatch tests.

use crate::context::{ExecutionContext, NetworkSettings};
use crate::scope::{open_project, safe_relative_path};
use crate::Operations;
use suncode_documents::Documents;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Dispatches `method` and wraps the outcome as `{"result"}` or `{"error"}`.
fn dispatch(
    (method, params): (&str, Value),
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
) -> Option<Value> {
    let context = ExecutionContext {
        project_root,
        checkpoint_root,
        cancellation: None,
        output_callback: None,
        network: NetworkSettings::default(),
        documents: &Documents::new(),
    };
    Some(match crate::dispatch::dispatch(method, &params, context) {
        Ok(value) => json!({"result":value}),
        Err(failure) => json!({"error":{"code":failure.code,"message":failure.message}}),
    })
}

fn request(method: &str, params: Value) -> (&str, Value) {
    (method, params)
}

#[test]
fn rejects_parent_paths() {
    assert!(safe_relative_path("../outside").is_err());
    assert!(safe_relative_path("/outside").is_err());
}

#[test]
fn lists_directories_without_following_symlinks() {
    let (root, checkpoints) = temporary_roots("directory-list");
    fs::create_dir_all(root.join("folder")).unwrap();
    fs::write(root.join("z.txt"), b"z").unwrap();
    fs::write(root.join("a.txt"), b"a").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(root.join("folder"), root.join("linked-folder")).unwrap();

    let operations = Operations::new(checkpoints.clone()).unwrap();
    let result = operations.list_directory(&root, ".", 2).unwrap();
    assert_eq!(result["entries"][0]["name"], "folder");
    assert_eq!(result["entries"][1]["name"], "a.txt");
    assert_eq!(result["truncated"], true);
    #[cfg(unix)]
    assert!(result["entries"]
        .as_array()
        .unwrap()
        .iter()
        .all(|entry| entry["name"] != "linked-folder"));
    cleanup(&root, &checkpoints);
}

#[test]
fn reads_bounded_utf8_files_without_following_symlinks() {
    let (root, checkpoints) = temporary_roots("text-file-read");
    fs::write(root.join("source.rs"), "pub fn main() {}\n").unwrap();
    fs::write(root.join("large.txt"), "12345").unwrap();
    fs::write(root.join("binary.bin"), [0xff, 0xfe]).unwrap();
    fs::write(root.join("nul.bin"), b"text\0binary").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(root.join("source.rs"), root.join("linked.rs")).unwrap();

    let operations = Operations::new(checkpoints.clone()).unwrap();
    let result = operations.read_text_file(&root, "source.rs", 1024).unwrap();
    assert_eq!(result["path"], "source.rs");
    assert_eq!(result["content"], "pub fn main() {}\n");
    assert_eq!(result["bytes"], 17);
    assert_eq!(
        operations
            .read_text_file(&root, "large.txt", 4)
            .unwrap_err()
            .code,
        "file_too_large"
    );
    assert_eq!(
        operations
            .read_text_file(&root, "binary.bin", 1024)
            .unwrap_err()
            .code,
        "encoding_unsupported"
    );
    assert_eq!(
        operations
            .read_text_file(&root, "nul.bin", 1024)
            .unwrap_err()
            .code,
        "encoding_unsupported"
    );
    #[cfg(unix)]
    assert_eq!(
        operations
            .read_text_file(&root, "linked.rs", 1024)
            .unwrap_err()
            .code,
        "scope_denied"
    );
    cleanup(&root, &checkpoints);
}

#[test]
fn retired_git_operations_are_unavailable() {
    for method in ["git/status", "git/diff-file"] {
        let response = dispatch(request(method, json!({"path": "file.txt"})), None, None).unwrap();
        assert_eq!(response["error"]["code"], "method_unavailable");
    }
}

#[test]
fn opens_and_canonicalizes_project_for_later_operations() {
    let root = std::env::temp_dir().join(format!(
        "suncode-core-open-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("README.md"), b"hello").unwrap();
    let (canonical, opened) = open_project(&root).unwrap();
    assert_eq!(canonical, root.canonicalize().unwrap());
    assert!(opened["canonical_path"].is_string());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn searches_bounded_globs_and_text_without_following_links() {
    let (root, checkpoints) = temporary_roots("search");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::write(root.join("src/main.rs"), b"needle here\nsecond").unwrap();
    fs::write(root.join("README.md"), b"needle docs").unwrap();
    fs::write(root.join(".gitignore"), b"ignored.rs\n").unwrap();
    fs::write(root.join("ignored.rs"), b"ignored").unwrap();
    fs::write(root.join(".hidden.rs"), b"hidden").unwrap();
    let glob = request("tool/glob", json!({"pattern":"**/*.rs","max_results":10}));
    let glob_result = dispatch(glob, Some(&root), Some(&checkpoints)).unwrap();
    assert_eq!(glob_result["result"]["paths"], json!(["src/main.rs"]));
    let find = request(
        "tool/grep",
        json!({"query":"needle","pattern":"**/*","max_results":10}),
    );
    let find_result = dispatch(find, Some(&root), Some(&checkpoints)).unwrap();
    assert_eq!(
        find_result["result"]["matches"].as_array().unwrap().len(),
        2
    );
    cleanup(&root, &checkpoints);
}

#[test]
fn read_applies_line_offset_and_limit() {
    let (root, checkpoints) = temporary_roots("read-range");
    fs::write(root.join("file.txt"), b"one\ntwo\nthree\nfour\n").unwrap();
    let response = dispatch(
        request("tool/read", json!({"path":"file.txt","offset":2,"limit":2})),
        Some(&root),
        Some(&checkpoints),
    )
    .unwrap();
    let bytes = STANDARD
        .decode(response["result"]["data_base64"].as_str().unwrap())
        .unwrap();
    assert_eq!(bytes, b"two\nthree\n");
    assert_eq!(response["result"]["offset"], 2);
    assert_eq!(response["result"]["limit"], 2);
    let invalid = dispatch(
        request("tool/read", json!({"path":"file.txt","offset":0})),
        Some(&root),
        Some(&checkpoints),
    )
    .unwrap();
    assert_eq!(invalid["error"]["code"], "invalid_arguments");
    cleanup(&root, &checkpoints);
}

#[test]
fn searches_with_rust_regexes_and_ripgrep_filters() {
    let (root, checkpoints) = temporary_roots("search-ripgrep");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::write(root.join(".gitignore"), b"ignored.rs\n").unwrap();
    fs::write(
        root.join("src/main.rs"),
        b"value 123 and value 456\nneedle\nneedle\n",
    )
    .unwrap();
    fs::write(root.join("ignored.rs"), b"value 999\n").unwrap();
    fs::write(root.join(".hidden.rs"), b"value 888\n").unwrap();

    let regex = request(
        "tool/grep",
        json!({"query":"value \\d+","pattern":"**/*.rs","max_results":10}),
    );
    let regex_result = dispatch(regex, Some(&root), Some(&checkpoints)).unwrap();
    assert_eq!(
        regex_result["result"]["matches"],
        json!([
            {"path":"src/main.rs","line":1,"column":1,"preview":"value 123 and value 456"},
            {"path":"src/main.rs","line":1,"column":15,"preview":"value 456"}
        ])
    );
    assert_eq!(regex_result["result"]["truncated"], false);

    let limited = request(
        "tool/grep",
        json!({"query":"needle","pattern":"**/*.rs","max_results":1}),
    );
    let limited_result = dispatch(limited, Some(&root), Some(&checkpoints)).unwrap();
    assert_eq!(
        limited_result["result"]["matches"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(limited_result["result"]["truncated"], true);

    let invalid = request(
        "tool/grep",
        json!({"query":"[unterminated","pattern":"**/*.rs"}),
    );
    assert_eq!(
        dispatch(invalid, Some(&root), Some(&checkpoints)).unwrap()["error"]["code"],
        "invalid_arguments"
    );
    cleanup(&root, &checkpoints);
}

#[test]
fn does_not_expose_absolute_project_root() {
    let response = dispatch(
        request("tool/read", json!({"path":"."})),
        Some(Path::new(".")),
        None,
    )
    .unwrap();
    assert!(response["result"].get("path").is_some() || response["error"].get("code").is_some());
    assert!(response.to_string().find("/Users/").is_none());
}

#[test]
fn writes_only_when_the_precondition_matches() {
    let root = std::env::temp_dir().join(format!(
        "suncode-core-write-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let checkpoints = root.join("checkpoint-storage");
    fs::create_dir_all(&checkpoints).unwrap();
    fs::write(root.join("file.txt"), b"before").unwrap();
    let expected = STANDARD.encode(b"before");
    let write = request(
        "tool/write",
        json!({"path":"file.txt","content_base64":STANDARD.encode(b"after"),"expected_base64":expected}),
    );
    let response = dispatch(write, Some(&root), Some(&checkpoints)).unwrap();
    assert_eq!(response["result"]["created"], false);
    assert!(response["result"]["checkpoint_id"].is_string());
    assert_eq!(fs::read_to_string(root.join("file.txt")).unwrap(), "after");
    let conflict = request(
        "tool/write",
        json!({"path":"file.txt","content_base64":STANDARD.encode(b"third"),"expected_base64":expected}),
    );
    assert_eq!(
        dispatch(conflict, Some(&root), Some(&checkpoints)).unwrap()["error"]["code"],
        "conflict"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn write_creates_nested_parent_and_edit_preserves_bom_and_crlf() {
    let (root, checkpoints) = temporary_roots("mutation-format");
    let write = dispatch(
        request(
            "tool/write",
            json!({
                "path":"nested/file.txt",
                "content_base64":STANDARD.encode(b"\xef\xbb\xbfone\r\ntwo\r\n"),
                "expected_base64":null
            }),
        ),
        Some(&root),
        Some(&checkpoints),
    )
    .unwrap();
    assert_eq!(write["result"]["created"], true);
    let current = fs::read(root.join("nested/file.txt")).unwrap();
    let edit = dispatch(
        request(
            "tool/edit",
            json!({
                "path":"nested/file.txt",
                "expected_base64":STANDARD.encode(&current),
                "replacements":[
                    {"old":"one","new":"uno","replace_all":false},
                    {"old":"two","new":"dos","replace_all":false}
                ]
            }),
        ),
        Some(&root),
        Some(&checkpoints),
    )
    .unwrap();
    assert!(edit["result"]["checkpoint_id"].is_string());
    assert_eq!(
        fs::read(root.join("nested/file.txt")).unwrap(),
        b"\xef\xbb\xbfuno\r\ndos\r\n"
    );
    let overlap = dispatch(
        request(
            "tool/edit",
            json!({
                "path":"nested/file.txt",
                "expected_base64":STANDARD.encode(b"\xef\xbb\xbfuno\r\ndos\r\n"),
                "replacements":[
                    {"old":"uno","new":"x","replace_all":false},
                    {"old":"uno\r\ndos","new":"y","replace_all":false}
                ]
            }),
        ),
        Some(&root),
        Some(&checkpoints),
    )
    .unwrap();
    assert_eq!(overlap["error"]["code"], "edit_conflict");
    cleanup(&root, &checkpoints);
}

#[test]
fn restores_existing_file_and_consumes_checkpoint() {
    let (root, checkpoints) = temporary_roots("restore-existing");
    fs::write(root.join("file.txt"), b"before").unwrap();
    let write = request(
        "tool/write",
        json!({
            "path":"file.txt",
            "content_base64":STANDARD.encode(b"after"),
            "expected_base64":STANDARD.encode(b"before")
        }),
    );
    let response = dispatch(write, Some(&root), Some(&checkpoints)).unwrap();
    let checkpoint_id = response["result"]["checkpoint_id"].as_str().unwrap();
    let restore = request(
        "checkpoint/restore",
        json!({"checkpoint_id": checkpoint_id}),
    );
    let restored = dispatch(restore, Some(&root), Some(&checkpoints)).unwrap();
    assert_eq!(restored["result"]["removed"], false);
    assert_eq!(fs::read(root.join("file.txt")).unwrap(), b"before");

    let repeated = request(
        "checkpoint/restore",
        json!({"checkpoint_id": checkpoint_id}),
    );
    assert_eq!(
        dispatch(repeated, Some(&root), Some(&checkpoints)).unwrap()["error"]["code"],
        "checkpoint_unavailable"
    );
    cleanup(&root, &checkpoints);
}

#[test]
fn restore_removes_file_created_by_write() {
    let (root, checkpoints) = temporary_roots("restore-created");
    let write = request(
        "tool/write",
        json!({
            "path":"created.txt",
            "content_base64":STANDARD.encode(b"created"),
            "expected_base64":null
        }),
    );
    let response = dispatch(write, Some(&root), Some(&checkpoints)).unwrap();
    let checkpoint_id = response["result"]["checkpoint_id"].as_str().unwrap();
    let restore = request(
        "checkpoint/restore",
        json!({"checkpoint_id": checkpoint_id}),
    );
    let restored = dispatch(restore, Some(&root), Some(&checkpoints)).unwrap();
    assert_eq!(restored["result"]["removed"], true);
    assert!(!root.join("created.txt").exists());
    cleanup(&root, &checkpoints);
}

#[test]
fn restore_rejects_external_post_write_change() {
    let (root, checkpoints) = temporary_roots("restore-conflict");
    fs::write(root.join("file.txt"), b"before").unwrap();
    let write = request(
        "tool/write",
        json!({
            "path":"file.txt",
            "content_base64":STANDARD.encode(b"after"),
            "expected_base64":STANDARD.encode(b"before")
        }),
    );
    let response = dispatch(write, Some(&root), Some(&checkpoints)).unwrap();
    let checkpoint_id = response["result"]["checkpoint_id"].as_str().unwrap();
    fs::write(root.join("file.txt"), b"external change").unwrap();
    let restore = request(
        "checkpoint/restore",
        json!({"checkpoint_id": checkpoint_id}),
    );
    assert_eq!(
        dispatch(restore, Some(&root), Some(&checkpoints)).unwrap()["error"]["code"],
        "restore_conflict"
    );
    assert_eq!(fs::read(root.join("file.txt")).unwrap(), b"external change");
    cleanup(&root, &checkpoints);
}

#[cfg(unix)]
#[test]
fn restore_rejects_target_replaced_by_external_symlink() {
    use std::os::unix::fs::symlink;

    let (root, checkpoints) = temporary_roots("restore-symlink");
    let write = request(
        "tool/write",
        json!({
            "path":"created.txt",
            "content_base64":STANDARD.encode(b"created"),
            "expected_base64":null
        }),
    );
    let response = dispatch(write, Some(&root), Some(&checkpoints)).unwrap();
    let checkpoint_id = response["result"]["checkpoint_id"].as_str().unwrap();
    let outside = std::env::temp_dir().join(format!("suncode-outside-{}", checkpoint_id));
    fs::write(&outside, b"created").unwrap();
    fs::remove_file(root.join("created.txt")).unwrap();
    symlink(&outside, root.join("created.txt")).unwrap();

    let restore = request(
        "checkpoint/restore",
        json!({"checkpoint_id": checkpoint_id}),
    );
    assert_eq!(
        dispatch(restore, Some(&root), Some(&checkpoints)).unwrap()["error"]["code"],
        "restore_conflict"
    );
    let write_through_link = request(
        "tool/write",
        json!({
            "path":"created.txt",
            "content_base64":STANDARD.encode(b"changed"),
            "expected_base64":STANDARD.encode(b"created")
        }),
    );
    assert_eq!(
        dispatch(write_through_link, Some(&root), Some(&checkpoints)).unwrap()["error"]["code"],
        "scope_denied"
    );
    assert_eq!(fs::read(&outside).unwrap(), b"created");
    cleanup(&root, &checkpoints);
    fs::remove_file(outside).unwrap();
}

#[test]
fn journals_write_and_reconciles_known_completion() {
    let (root, checkpoints) = temporary_roots("journal");
    let params = json!({"path":"file.txt","content_base64":STANDARD.encode(b"after"),"expected_base64":null,"idempotency_key":"write-once"});
    let first = dispatch(
        request("tool/write", params.clone()),
        Some(&root),
        Some(&checkpoints),
    )
    .unwrap();
    let second = dispatch(
        request("tool/write", params),
        Some(&root),
        Some(&checkpoints),
    )
    .unwrap();
    assert_eq!(
        first["result"]["checkpoint_id"],
        second["result"]["checkpoint_id"]
    );
    cleanup(&root, &checkpoints);
}

fn temporary_roots(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("suncode-core-{}-{}", name, nonce));
    let checkpoints = std::env::temp_dir().join(format!("suncode-checkpoints-{}-{}", name, nonce));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&checkpoints).unwrap();
    (
        root.canonicalize().unwrap(),
        checkpoints.canonicalize().unwrap(),
    )
}

fn cleanup(root: &Path, checkpoints: &Path) {
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(checkpoints).unwrap();
}

#[test]
fn truncated_read_stores_full_artifact_and_keeps_utf8_boundary() {
    let (root, checkpoints) = temporary_roots("read-artifact");
    // "é" is two bytes, so a 3-byte preview must stop after "aé" without
    // splitting the second "é".
    fs::write(root.join("file.txt"), "aéé".as_bytes()).unwrap();
    let response = dispatch(
        request("tool/read", json!({"path":"file.txt","max_bytes":4})),
        Some(&root),
        Some(&checkpoints),
    )
    .unwrap();
    let result = &response["result"];
    let preview = STANDARD
        .decode(result["data_base64"].as_str().unwrap())
        .unwrap();
    assert_eq!(String::from_utf8(preview).unwrap(), "aé");
    assert_eq!(result["bytes"], 3);
    assert_eq!(result["total_bytes"], 5);
    assert_eq!(result["truncated"], true);
    let artifact_id = result["artifact_id"].as_str().unwrap();
    let artifact =
        crate::artifacts::artifact_directory(&checkpoints).join(format!("{artifact_id}.bin"));
    assert_eq!(fs::read(&artifact).unwrap(), "aéé".as_bytes());
    fs::remove_file(artifact).ok();
    cleanup(&root, &checkpoints);
}

#[test]
fn glob_and_grep_share_globset_syntax() {
    let (root, checkpoints) = temporary_roots("glob-syntax");
    fs::create_dir_all(root.join("src/ui")).unwrap();
    fs::write(root.join("main.ts"), b"needle").unwrap();
    fs::write(root.join("src/ui/view.tsx"), b"needle").unwrap();
    fs::write(root.join("src/ui/style.css"), b"needle").unwrap();
    let glob = dispatch(
        request("tool/glob", json!({"pattern":"**/*.{ts,tsx}"})),
        Some(&root),
        Some(&checkpoints),
    )
    .unwrap();
    assert_eq!(
        glob["result"]["paths"],
        json!(["main.ts", "src/ui/view.tsx"])
    );
    let grep = dispatch(
        request(
            "tool/grep",
            json!({"query":"needle","pattern":"**/*.{ts,tsx}"}),
        ),
        Some(&root),
        Some(&checkpoints),
    )
    .unwrap();
    assert_eq!(grep["result"]["matches"].as_array().unwrap().len(), 2);
    let invalid = dispatch(
        request("tool/glob", json!({"pattern":"src/[ui"})),
        Some(&root),
        Some(&checkpoints),
    )
    .unwrap();
    assert_eq!(invalid["error"]["code"], "invalid_arguments");
    cleanup(&root, &checkpoints);
}

#[test]
fn bracket_escaped_directory_matches_literally() {
    let (root, checkpoints) = temporary_roots("glob-literal-directory");
    fs::create_dir_all(root.join("app/[id]")).unwrap();
    fs::create_dir_all(root.join("app/i")).unwrap();
    fs::write(root.join("app/[id]/page.tsx"), b"x").unwrap();
    fs::write(root.join("app/i/page.tsx"), b"x").unwrap();
    let glob = dispatch(
        request("tool/glob", json!({"pattern":"app/[[]id[]]/**/*.tsx"})),
        Some(&root),
        Some(&checkpoints),
    )
    .unwrap();
    assert_eq!(glob["result"]["paths"], json!(["app/[id]/page.tsx"]));
    cleanup(&root, &checkpoints);
}
