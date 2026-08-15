//! Audited in-process operations for the SunCode runtime.

#[cfg(test)]
use base64::engine::general_purpose::STANDARD;
use serde::Deserialize;
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::process::Child;
use std::sync::atomic::AtomicU64;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static CHECKPOINT_SEQUENCE: AtomicU64 = AtomicU64::new(1);
static PROCESSES: OnceLock<Mutex<std::collections::HashMap<String, Child>>> = OnceLock::new();

mod artifacts;
mod checkpoint;
mod filesystem;
mod git;
mod mutations;
mod process;
mod search;
mod tools;
mod write;

#[derive(Debug)]
struct CoreFailure {
    code: &'static str,
    message: &'static str,
    retryable: bool,
}

fn execute_operation(
    method: &str,
    params: &Value,
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
) -> Result<Value, CoreFailure> {
    if let Some(result) = tools::dispatch(method, params, project_root, checkpoint_root) {
        return result;
    }
    match method {
        "git/status" => git::status(project_root, params),
        "git/diff-file" => git::diff_file(project_root, params),
        "sandbox/profiles" => sandbox_profiles(),
        "capability/check" => capability_check(project_root, params),
        "capability/execute" => capability_execute(project_root, checkpoint_root, params),
        _ => Err(CoreFailure {
            code: "method_unavailable",
            message: "method unavailable",
            retryable: false,
        }),
    }
}

fn open_project(path: &Path) -> Result<(PathBuf, Value), CoreFailure> {
    let requested = path;
    if !requested.is_absolute() {
        return Err(CoreFailure {
            code: "invalid_arguments",
            message: "project path must be absolute",
            retryable: false,
        });
    }
    let root = requested.canonicalize().map_err(|_| CoreFailure {
        code: "project_unavailable",
        message: "project root is unavailable",
        retryable: false,
    })?;
    if !root.is_dir() {
        return Err(CoreFailure {
            code: "project_unavailable",
            message: "project root is not a directory",
            retryable: false,
        });
    }
    let canonical_path = root.to_str().ok_or(CoreFailure {
        code: "project_unavailable",
        message: "project path is not valid UTF-8",
        retryable: false,
    })?;
    let display_name = root
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("Project");
    let value = json!({"canonical_path": canonical_path, "display_name": display_name});
    Ok((root, value))
}

fn sandbox_profiles() -> Result<Value, CoreFailure> {
    Ok(json!({"profiles": [
        {"id": "project-default", "network": "not_enforced", "filesystem": "project-cwd", "environment": "filtered", "os_isolation": false},
        {"id": "project-readonly", "network": "not_enforced", "filesystem": "project-cwd", "environment": "filtered", "os_isolation": false}
    ]}))
}

fn capability_check(project_root: Option<&Path>, params: &Value) -> Result<Value, CoreFailure> {
    let operation = params
        .get("operation")
        .and_then(Value::as_str)
        .ok_or(CoreFailure {
            code: "invalid_arguments",
            message: "operation is required",
            retryable: false,
        })?;
    let known = [
        "fs.read",
        "fs.metadata",
        "fs.write",
        "fs.edit",
        "fs.patch",
        "fs.move",
        "fs.delete",
        "process.run",
        "process.start",
        "artifact.read",
        "checkpoint.restore",
    ];
    if !known.contains(&operation) {
        return Err(CoreFailure {
            code: "capability_denied",
            message: "operation is not available",
            retryable: false,
        });
    }
    if !matches!(operation, "artifact.read") && project_root.is_none() {
        return Err(CoreFailure {
            code: "project_unconfigured",
            message: "project root is not configured",
            retryable: false,
        });
    }
    if let Some(resource) = params.get("resource").and_then(Value::as_object) {
        for key in ["path", "from", "to"] {
            if let Some(path) = resource.get(key).and_then(Value::as_str) {
                safe_relative_path(path)?;
            }
        }
    }
    let grant_id = params
        .get("grant_id")
        .and_then(Value::as_str)
        .ok_or(CoreFailure {
            code: "capability_denied",
            message: "grant_id is required",
            retryable: false,
        })?;
    let assertion_id = sha256_hex(
        format!(
            "{}:{}:{}",
            operation,
            grant_id,
            params
                .get("expires_at")
                .and_then(Value::as_str)
                .unwrap_or("session")
        )
        .as_bytes(),
    );
    Ok(
        json!({"allowed": true, "operation": operation, "assertion_id": assertion_id, "sandbox_profile": params.get("sandbox_profile").and_then(Value::as_str).unwrap_or("project-default")}),
    )
}

fn capability_execute(
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
    params: &Value,
) -> Result<Value, CoreFailure> {
    let operation = params
        .get("operation")
        .and_then(Value::as_str)
        .ok_or(CoreFailure {
            code: "invalid_arguments",
            message: "operation is required",
            retryable: false,
        })?;
    capability_check(project_root, params)?;
    let arguments = params.get("arguments").unwrap_or(params);
    match operation {
        "fs.read" => filesystem::read(project_root, arguments),
        "fs.metadata" => filesystem::metadata(project_root, arguments),
        "fs.write" => write::write(project_root, checkpoint_root, arguments),
        "fs.edit" => mutations::edit(project_root, checkpoint_root, arguments),
        "fs.patch" => mutations::patch(project_root, checkpoint_root, arguments),
        "fs.move" => mutations::move_file(project_root, checkpoint_root, arguments),
        "fs.delete" => mutations::delete(project_root, checkpoint_root, arguments),
        "process.run" => process::run(project_root, checkpoint_root, arguments),
        "process.start" => process::start(project_root, arguments),
        "artifact.read" => artifacts::read(checkpoint_root, arguments),
        "checkpoint.restore" => checkpoint::restore(project_root, checkpoint_root, arguments),
        _ => Err(CoreFailure {
            code: "capability_denied",
            message: "operation is not executable",
            retryable: false,
        }),
    }
}

fn failure_value(failure: CoreFailure) -> Value {
    json!({"code":failure.code,"message":failure.message,"retryable":failure.retryable})
}

fn project_inspect(project_root: Option<&Path>, params: &Value) -> Result<Value, CoreFailure> {
    let root = project_root.ok_or(CoreFailure {
        code: "project_unconfigured",
        message: "project root is not configured",
        retryable: false,
    })?;
    let metadata = fs::metadata(root).map_err(|_| CoreFailure {
        code: "project_unavailable",
        message: "project root is unavailable",
        retryable: false,
    })?;
    if !metadata.is_dir() {
        return Err(CoreFailure {
            code: "project_unavailable",
            message: "project root is not a directory",
            retryable: false,
        });
    }
    let requested = params
        .get("max_entries")
        .and_then(Value::as_u64)
        .unwrap_or(100)
        .clamp(1, 1000) as usize;
    let mut entries = Vec::new();
    for item in fs::read_dir(root).map_err(|_| CoreFailure {
        code: "project_read_failed",
        message: "project directory could not be read",
        retryable: true,
    })? {
        let item = item.map_err(|_| CoreFailure {
            code: "project_read_failed",
            message: "project directory entry could not be read",
            retryable: true,
        })?;
        let kind = item.file_type().map_err(|_| CoreFailure {
            code: "project_read_failed",
            message: "project entry type could not be read",
            retryable: true,
        })?;
        entries.push(json!({
            "name": item.file_name().to_string_lossy(),
            "kind": if kind.is_dir() {"directory"} else if kind.is_file() {"file"} else if kind.is_symlink() {"symlink"} else {"other"}
        }));
        if entries.len() > requested {
            break;
        }
    }
    entries.sort_by(|left, right| left["name"].as_str().cmp(&right["name"].as_str()));
    let truncated = entries.len() > requested;
    entries.truncate(requested);
    Ok(json!({"path": ".", "kind": "directory", "entries": entries, "truncated": truncated}))
}

fn existing_file(root: &Path, path: &str) -> Result<(PathBuf, Vec<u8>), CoreFailure> {
    let relative = safe_relative_path(path)?;
    let candidate = root.join(relative);
    let metadata = fs::symlink_metadata(&candidate).map_err(|_| CoreFailure {
        code: "path_unavailable",
        message: "path is unavailable",
        retryable: false,
    })?;
    if metadata.file_type().is_symlink() {
        return Err(CoreFailure {
            code: "scope_denied",
            message: "symbolic links are not allowed",
            retryable: false,
        });
    }
    let canonical = candidate.canonicalize().map_err(|_| CoreFailure {
        code: "path_unavailable",
        message: "path is unavailable",
        retryable: false,
    })?;
    if !canonical.starts_with(root) {
        return Err(CoreFailure {
            code: "scope_denied",
            message: "path is outside the project",
            retryable: false,
        });
    }
    if !metadata.is_file() {
        return Err(CoreFailure {
            code: "not_a_file",
            message: "path is not a regular file",
            retryable: false,
        });
    }
    let bytes = fs::read(&canonical).map_err(|_| CoreFailure {
        code: "read_failed",
        message: "file could not be read",
        retryable: true,
    })?;
    Ok((canonical, bytes))
}

fn require_project(project_root: Option<&Path>) -> Result<&Path, CoreFailure> {
    let root = project_root.ok_or(CoreFailure {
        code: "project_unconfigured",
        message: "project root is not configured",
        retryable: false,
    })?;
    if !root.is_dir() {
        return Err(CoreFailure {
            code: "project_unavailable",
            message: "project root is unavailable",
            retryable: false,
        });
    }
    Ok(root)
}

fn collect_files(
    root: &Path,
    current: &Path,
    visit: &mut impl FnMut(&str, &Path),
) -> Result<(), CoreFailure> {
    let mut entries = fs::read_dir(current)
        .map_err(|_| CoreFailure {
            code: "project_read_failed",
            message: "project directory could not be read",
            retryable: true,
        })?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let file_type = entry.file_type().map_err(|_| CoreFailure {
            code: "project_read_failed",
            message: "project entry type could not be read",
            retryable: true,
        })?;
        let absolute = entry.path();
        let Ok(relative_path) = absolute.strip_prefix(root) else {
            continue;
        };
        let relative = relative_path
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/");
        if file_type.is_dir() {
            collect_files(root, &absolute, visit)?;
        } else if file_type.is_file() {
            visit(&relative, &absolute);
        }
    }
    Ok(())
}

fn glob_matches(pattern: &str, value: &str) -> bool {
    let pattern_parts = pattern.split('/').collect::<Vec<_>>();
    let value_parts = value.split('/').collect::<Vec<_>>();
    glob_segments(&pattern_parts, &value_parts)
}

fn glob_segments(pattern: &[&str], value: &[&str]) -> bool {
    if pattern.is_empty() {
        return value.is_empty();
    }
    if pattern[0] == "**" {
        return glob_segments(&pattern[1..], value)
            || (!value.is_empty() && glob_segments(pattern, &value[1..]));
    }
    !value.is_empty()
        && segment_matches(pattern[0], value[0])
        && glob_segments(&pattern[1..], &value[1..])
}

fn segment_matches(pattern: &str, value: &str) -> bool {
    let pattern = pattern.as_bytes();
    let value = value.as_bytes();
    let mut p = 0;
    let mut v = 0;
    let mut star = None;
    let mut mark = 0;
    while v < value.len() {
        if p < pattern.len() && (pattern[p] == value[v] || pattern[p] == b'?') {
            p += 1;
            v += 1;
        } else if p < pattern.len() && pattern[p] == b'*' {
            star = Some(p);
            p += 1;
            mark = v;
        } else if let Some(star_at) = star {
            p = star_at + 1;
            mark += 1;
            v = mark;
        } else {
            return false;
        }
    }
    while p < pattern.len() && pattern[p] == b'*' {
        p += 1;
    }
    p == pattern.len()
}

#[derive(Serialize, Deserialize)]
struct JournalRecord {
    operation_id: String,
    operation: String,
    path: Option<String>,
    pre_image_sha256: Option<String>,
    post_image_sha256: Option<String>,
    status: String,
    #[serde(default)]
    result: Option<Value>,
    created_at: String,
    updated_at: String,
}

fn journal_directory(checkpoint_root: &Path) -> PathBuf {
    checkpoint_root
        .parent()
        .unwrap_or(checkpoint_root)
        .join("journal")
}

fn journal_id(params: &Value) -> Option<String> {
    params
        .get("idempotency_key")
        .or_else(|| params.get("operation_id"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(|value| sha256_hex(value.as_bytes()))
}

fn journal_path(checkpoint_root: &Path, id: &str) -> PathBuf {
    journal_directory(checkpoint_root).join(format!("{}.json", id))
}

fn load_journal(checkpoint_root: &Path, id: &str) -> Option<JournalRecord> {
    serde_json::from_slice(&fs::read(journal_path(checkpoint_root, id)).ok()?).ok()
}

fn save_journal(checkpoint_root: &Path, record: &JournalRecord) -> Result<(), CoreFailure> {
    fs::create_dir_all(journal_directory(checkpoint_root)).map_err(|_| CoreFailure {
        code: "journal_failed",
        message: "operation journal could not be created",
        retryable: true,
    })?;
    fs::write(
        journal_path(checkpoint_root, &record.operation_id),
        serde_json::to_vec(record).map_err(|_| CoreFailure {
            code: "journal_failed",
            message: "operation journal could not be encoded",
            retryable: false,
        })?,
    )
    .map_err(|_| CoreFailure {
        code: "journal_failed",
        message: "operation journal could not be written",
        retryable: true,
    })
}

fn journal_intent(
    checkpoint_root: &Path,
    params: &Value,
    operation: &str,
    path: Option<&str>,
    pre: Option<&[u8]>,
    post: Option<&[u8]>,
) -> Result<Option<String>, CoreFailure> {
    let Some(id) = journal_id(params) else {
        return Ok(None);
    };
    if let Some(existing) = load_journal(checkpoint_root, &id) {
        if existing.status == "succeeded" {
            return Err(CoreFailure {
                code: "operation_already_completed",
                message: "operation already completed",
                retryable: false,
            });
        }
        if existing.status == "pending" {
            return Err(CoreFailure {
                code: "unknown_completion",
                message: "operation completion is unknown and must be reconciled",
                retryable: false,
            });
        }
    }
    let now = now_string();
    save_journal(
        checkpoint_root,
        &JournalRecord {
            operation_id: id.clone(),
            operation: operation.to_string(),
            path: path.map(str::to_string),
            pre_image_sha256: pre.map(sha256_hex),
            post_image_sha256: post.map(sha256_hex),
            status: "pending".to_string(),
            result: None,
            created_at: now.clone(),
            updated_at: now,
        },
    )?;
    Ok(Some(id))
}

fn journal_finish(checkpoint_root: &Path, id: Option<&str>, status: &str, result: Option<Value>) {
    let Some(id) = id else {
        return;
    };
    let Some(mut record) = load_journal(checkpoint_root, id) else {
        return;
    };
    record.status = status.to_string();
    record.result = result;
    record.updated_at = now_string();
    let _ = save_journal(checkpoint_root, &record);
}

fn now_string() -> String {
    chrono_like_now()
}

fn chrono_like_now() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", seconds)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn safe_relative_path(path: &str) -> Result<PathBuf, CoreFailure> {
    let value = Path::new(path);
    if value.as_os_str().is_empty() || value.is_absolute() {
        return Err(CoreFailure {
            code: "scope_denied",
            message: "path must be relative to the project",
            retryable: false,
        });
    }
    for component in value.components() {
        if matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        ) {
            return Err(CoreFailure {
                code: "scope_denied",
                message: "path must remain inside the project",
                retryable: false,
            });
        }
    }
    Ok(value.to_path_buf())
}

pub struct Operations {
    checkpoint_root: PathBuf,
}

impl Operations {
    pub fn new(checkpoint_root: PathBuf) -> io::Result<Self> {
        fs::create_dir_all(&checkpoint_root)?;
        Ok(Self {
            checkpoint_root: checkpoint_root.canonicalize()?,
        })
    }

    pub fn open_project(&self, project_path: &Path) -> Result<Value, Value> {
        open_project(project_path)
            .map(|(_, value)| value)
            .map_err(failure_value)
    }

    pub fn execute_in_project(
        &self,
        project_path: &Path,
        method: &str,
        params: Value,
    ) -> Result<Value, Value> {
        let canonical = project_path.canonicalize().map_err(|_| json!({"code":"project_unavailable","message":"project root is unavailable","retryable":false}))?;
        if !canonical.is_dir() {
            return Err(
                json!({"code":"project_unavailable","message":"project root is not a directory","retryable":false}),
            );
        }
        execute_operation(
            method,
            &params,
            Some(&canonical),
            Some(&self.checkpoint_root),
        )
        .map_err(failure_value)
    }
}

#[cfg(test)]
mod tests {
    use super::{execute_operation, failure_value, open_project, safe_relative_path};
    use base64::Engine;
    use serde::Deserialize;
    use serde_json::json;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Deserialize)]
    struct Request {
        method: String,
        #[serde(default)]
        params: serde_json::Value,
    }

    fn dispatch(
        request: Request,
        project_root: Option<&Path>,
        checkpoint_root: Option<&Path>,
    ) -> Option<serde_json::Value> {
        if request.method == "core/health" {
            return Some(
                json!({"result":{"status":"ready","project_configured":project_root.is_some()}}),
            );
        }
        Some(
            match execute_operation(
                &request.method,
                &request.params,
                project_root,
                checkpoint_root,
            ) {
                Ok(value) => json!({"result":value}),
                Err(failure) => json!({"error":failure_value(failure)}),
            },
        )
    }

    fn dispatch_with_project(
        request: Request,
        project_root: &mut Option<PathBuf>,
        checkpoint_root: Option<&Path>,
    ) -> Option<serde_json::Value> {
        if request.method == "project/open" {
            let path = request.params.get("path")?.as_str()?;
            return Some(match open_project(Path::new(path)) {
                Ok((root, value)) => {
                    *project_root = Some(root);
                    json!({"result":value})
                }
                Err(failure) => json!({"error":failure_value(failure)}),
            });
        }
        dispatch(request, project_root.as_deref(), checkpoint_root)
    }

    #[test]
    fn rejects_parent_paths() {
        assert!(safe_relative_path("../outside").is_err());
        assert!(safe_relative_path("/outside").is_err());
    }

    #[test]
    fn responds_to_health_without_project() {
        let request = serde_json::from_value(
            json!({"jsonrpc":"2.0","id":"1","method":"core/health","params":{}}),
        )
        .unwrap();
        let response = dispatch(request, None, None).unwrap();
        assert_eq!(response["result"]["project_configured"], false);
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
        let mut project_root = None;
        let open = serde_json::from_value(json!({
            "jsonrpc":"2.0","id":"1","method":"project/open","params":{"path":root}
        }))
        .unwrap();
        let opened = dispatch_with_project(open, &mut project_root, None).unwrap();
        assert!(opened["result"]["canonical_path"].is_string());
        let inspect = serde_json::from_value(json!({
            "jsonrpc":"2.0","id":"2","method":"project/inspect","params":{}
        }))
        .unwrap();
        let inspected = dispatch_with_project(inspect, &mut project_root, None).unwrap();
        assert_eq!(inspected["result"]["entries"][0]["name"], "README.md");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn searches_bounded_globs_and_text_without_following_links() {
        let (root, checkpoints) = temporary_roots("search");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), b"needle here\nsecond").unwrap();
        fs::write(root.join("README.md"), b"needle docs").unwrap();
        let glob = request("search/glob", json!({"pattern":"**/*.rs","max_results":10}));
        let glob_result = dispatch(glob, Some(&root), Some(&checkpoints)).unwrap();
        assert_eq!(glob_result["result"]["paths"], json!(["src/main.rs"]));
        let find = request(
            "search/find",
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
            "search/find",
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
            "search/find",
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
            "search/find",
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
        let request = serde_json::from_value(
            json!({"jsonrpc":"2.0","id":"1","method":"project/inspect","params":{}}),
        )
        .unwrap();
        let response = dispatch(request, Some(Path::new(".")), None).unwrap();
        assert!(
            response["result"].get("path").is_some() || response["error"].get("code").is_some()
        );
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
        let expected = super::STANDARD.encode(b"before");
        let request = serde_json::from_value(json!({
            "jsonrpc":"2.0","id":"1","method":"fs/write",
            "params":{"path":"file.txt","content_base64":super::STANDARD.encode(b"after"),"expected_base64":expected}
        })).unwrap();
        let response = dispatch(request, Some(&root), Some(&checkpoints)).unwrap();
        assert_eq!(response["result"]["created"], false);
        assert!(response["result"]["checkpoint_id"].is_string());
        assert_eq!(fs::read_to_string(root.join("file.txt")).unwrap(), "after");
        let conflict = serde_json::from_value(json!({
            "jsonrpc":"2.0","id":"2","method":"fs/write",
            "params":{"path":"file.txt","content_base64":super::STANDARD.encode(b"third"),"expected_base64":expected}
        })).unwrap();
        assert_eq!(
            dispatch(conflict, Some(&root), Some(&checkpoints)).unwrap()["error"]["code"],
            "conflict"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn restores_existing_file_and_consumes_checkpoint() {
        let (root, checkpoints) = temporary_roots("restore-existing");
        fs::write(root.join("file.txt"), b"before").unwrap();
        let write = request(
            "fs/write",
            json!({
                "path":"file.txt",
                "content_base64":super::STANDARD.encode(b"after"),
                "expected_base64":super::STANDARD.encode(b"before")
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
            "fs/write",
            json!({
                "path":"created.txt",
                "content_base64":super::STANDARD.encode(b"created"),
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
            "fs/write",
            json!({
                "path":"file.txt",
                "content_base64":super::STANDARD.encode(b"after"),
                "expected_base64":super::STANDARD.encode(b"before")
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
            "fs/write",
            json!({
                "path":"created.txt",
                "content_base64":super::STANDARD.encode(b"created"),
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
            "fs/write",
            json!({
                "path":"created.txt",
                "content_base64":super::STANDARD.encode(b"changed"),
                "expected_base64":super::STANDARD.encode(b"created")
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
    fn supports_preconditioned_edit_patch_move_delete_and_artifacts() {
        let (root, checkpoints) = temporary_roots("catalog");
        fs::write(root.join("file.txt"), b"one\ntwo\n").unwrap();
        let base = super::STANDARD.encode(b"one\ntwo\n");
        let edit = dispatch(request("fs/edit", json!({"path":"file.txt","expected_base64":base,"replacements":[{"old":"two","new":"three"}]})), Some(&root), Some(&checkpoints)).unwrap();
        assert_eq!(
            fs::read_to_string(root.join("file.txt")).unwrap(),
            "one\nthree\n"
        );
        let patch = dispatch(request("fs/patch", json!({"path":"file.txt","expected_base64":super::STANDARD.encode(b"one\nthree\n"),"patch":"@@\n-one\n+ONE"})), Some(&root), Some(&checkpoints)).unwrap();
        assert!(patch["result"]["checkpoint_id"].is_string());
        assert_eq!(
            fs::read_to_string(root.join("file.txt")).unwrap(),
            "ONE\nthree\n"
        );
        let move_result = dispatch(request("fs/move", json!({"from":"file.txt","to":"moved.txt","expected_base64":super::STANDARD.encode(b"ONE\nthree\n")})), Some(&root), Some(&checkpoints)).unwrap();
        assert_eq!(
            move_result["result"]["checkpoint_ids"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert!(!root.join("file.txt").exists());
        let delete = dispatch(request("fs/delete", json!({"path":"moved.txt","expected_base64":super::STANDARD.encode(b"ONE\nthree\n")})), Some(&root), Some(&checkpoints)).unwrap();
        assert_eq!(delete["result"]["deleted"], true);
        let artifact_id =
            super::artifacts::write_artifact(&checkpoints, &vec![b'x'; 70_000]).unwrap();
        let artifact = dispatch(
            request("artifact/read", json!({"artifact_id": artifact_id})),
            Some(&root),
            Some(&checkpoints),
        )
        .unwrap();
        assert_eq!(artifact["result"]["truncated"], false);
        assert!(edit["result"]["checkpoint_id"].is_string());
        cleanup(&root, &checkpoints);
    }

    #[test]
    fn journals_write_and_reconciles_known_completion() {
        let (root, checkpoints) = temporary_roots("journal");
        let params = json!({"path":"file.txt","content_base64":super::STANDARD.encode(b"after"),"expected_base64":null,"idempotency_key":"write-once"});
        let first = dispatch(
            request("fs/write", params.clone()),
            Some(&root),
            Some(&checkpoints),
        )
        .unwrap();
        let second =
            dispatch(request("fs/write", params), Some(&root), Some(&checkpoints)).unwrap();
        assert_eq!(
            first["result"]["checkpoint_id"],
            second["result"]["checkpoint_id"]
        );
        let status = dispatch(
            request("operation/status", json!({"operation_id":"write-once"})),
            Some(&root),
            Some(&checkpoints),
        )
        .unwrap();
        assert_eq!(status["result"]["status"], "succeeded");
        cleanup(&root, &checkpoints);
    }

    fn request(method: &str, params: serde_json::Value) -> Request {
        serde_json::from_value(json!({
            "jsonrpc":"2.0",
            "id":"test",
            "method":method,
            "params":params
        }))
        .unwrap()
    }

    fn temporary_roots(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("suncode-core-{}-{}", name, nonce));
        let checkpoints =
            std::env::temp_dir().join(format!("suncode-checkpoints-{}-{}", name, nonce));
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
}
