use super::{collect_files, glob_matches, require_project, CoreFailure};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub(super) fn glob(project_root: Option<&Path>, params: &Value) -> Result<Value, CoreFailure> {
    let root = require_project(project_root)?;
    let pattern = params
        .get("pattern")
        .and_then(Value::as_str)
        .ok_or(CoreFailure {
            code: "invalid_arguments",
            message: "pattern is required",
            retryable: false,
        })?;
    if pattern.is_empty()
        || Path::new(pattern).is_absolute()
        || pattern.split('/').any(|part| part == "..")
    {
        return Err(CoreFailure {
            code: "scope_denied",
            message: "pattern must remain inside the project",
            retryable: false,
        });
    }
    let max_results = params
        .get("max_results")
        .and_then(Value::as_u64)
        .unwrap_or(200)
        .clamp(1, 1000) as usize;
    let mut paths = Vec::new();
    collect_files(root, root, &mut |relative, _| {
        if glob_matches(pattern, relative) && paths.len() <= max_results {
            paths.push(relative.to_string());
        }
    })?;
    let truncated = paths.len() > max_results;
    paths.truncate(max_results);
    paths.sort();
    Ok(json!({"pattern": pattern, "paths": paths, "truncated": truncated}))
}

pub(super) fn find(project_root: Option<&Path>, params: &Value) -> Result<Value, CoreFailure> {
    let root = require_project(project_root)?;
    let query = params
        .get("query")
        .and_then(Value::as_str)
        .ok_or(CoreFailure {
            code: "invalid_arguments",
            message: "query is required",
            retryable: false,
        })?;
    if query.is_empty() || query.len() > 256 {
        return Err(CoreFailure {
            code: "invalid_arguments",
            message: "query must contain 1-256 bytes",
            retryable: false,
        });
    }
    let pattern = params
        .get("pattern")
        .and_then(Value::as_str)
        .unwrap_or("**/*");
    let max_results = params
        .get("max_results")
        .and_then(Value::as_u64)
        .unwrap_or(100)
        .clamp(1, 500) as usize;
    let mut matches = Vec::new();
    collect_files(root, root, &mut |relative, absolute| {
        if matches.len() > max_results || !glob_matches(pattern, relative) {
            return;
        }
        let Ok(bytes) = fs::read(absolute) else {
            return;
        };
        if bytes.len() > 2 * 1024 * 1024 || bytes.iter().take(4096).any(|byte| *byte == 0) {
            return;
        }
        let Ok(text) = String::from_utf8(bytes) else {
            return;
        };
        for (line_index, line) in text.lines().enumerate() {
            let mut offset = 0;
            while let Some(found) = line[offset..].find(query) {
                if matches.len() >= max_results {
                    return;
                }
                let column = offset + found;
                let preview: String = line.chars().skip(column).take(240).collect();
                matches.push(json!({"path": relative, "line": line_index + 1, "column": column + 1, "preview": preview}));
                offset += found + query.len();
                if offset >= line.len() {
                    break;
                }
            }
        }
    })?;
    let truncated = matches.len() >= max_results;
    Ok(json!({"query": query, "matches": matches, "truncated": truncated}))
}
