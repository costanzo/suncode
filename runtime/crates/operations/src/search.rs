use super::{glob_matches, require_project, CoreFailure};
use globset::GlobBuilder;
use grep_matcher::Matcher;
use grep_regex::{RegexMatcher, RegexMatcherBuilder};
use grep_searcher::{BinaryDetection, SearcherBuilder, Sink, SinkMatch};
use ignore::WalkBuilder;
use serde_json::{json, Value};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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
        .unwrap_or(100)
        .clamp(1, 1000) as usize;
    let mut paths = Vec::new();
    let walker = WalkBuilder::new(root)
        .standard_filters(true)
        .follow_links(false)
        .build();
    for entry in walker {
        let Ok(entry) = entry else { continue };
        let Some(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_file() {
            continue;
        }
        let absolute = entry.into_path();
        let Ok(relative_path) = absolute.strip_prefix(root) else {
            continue;
        };
        let relative = relative_path
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/");
        if glob_matches(pattern, &relative) {
            paths.push(relative);
            if paths.len() > max_results {
                break;
            }
        }
    }
    let truncated = paths.len() > max_results;
    paths.truncate(max_results);
    paths.sort();
    Ok(json!({"pattern": pattern, "paths": paths, "truncated": truncated}))
}

struct MatchSink<'a> {
    matcher: &'a RegexMatcher,
    relative: &'a str,
    matches: &'a mut Vec<Value>,
    max_results: usize,
    truncated: bool,
}

impl Sink for MatchSink<'_> {
    type Error = io::Error;

    fn matched(
        &mut self,
        _searcher: &grep_searcher::Searcher,
        mat: &SinkMatch<'_>,
    ) -> Result<bool, Self::Error> {
        let line_number = mat
            .line_number()
            .ok_or_else(|| io::Error::other("line numbers were not enabled"))?;
        let mut line = mat.bytes();
        if line.ends_with(b"\n") {
            line = &line[..line.len() - 1];
            if line.ends_with(b"\r") {
                line = &line[..line.len() - 1];
            }
        }

        let mut stop = false;
        self.matcher
            .find_iter(line, |found| {
                if self.matches.len() >= self.max_results {
                    self.truncated = true;
                    stop = true;
                    return false;
                }
                let start = found.start();
                let preview: String = String::from_utf8_lossy(&line[start..])
                    .chars()
                    .take(240)
                    .collect();
                self.matches.push(json!({
                    "path": self.relative,
                    "line": line_number,
                    "column": start + 1,
                    "preview": preview,
                }));
                true
            })
            .map_err(|error| io::Error::other(error.to_string()))?;
        Ok(!stop)
    }
}

fn project_files(root: &Path, pattern: &str) -> Result<Vec<(String, PathBuf)>, CoreFailure> {
    let glob = GlobBuilder::new(pattern)
        .literal_separator(true)
        .build()
        .map_err(|_| CoreFailure {
            code: "invalid_arguments",
            message: "include pattern is invalid",
            retryable: false,
        })?
        .compile_matcher();
    let mut files = Vec::new();
    let mut walker = WalkBuilder::new(root);
    walker.standard_filters(true).follow_links(false);
    for entry in walker.build() {
        let Ok(entry) = entry else {
            continue;
        };
        let Some(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_file() {
            continue;
        }
        let absolute = entry.into_path();
        let Ok(relative_path) = absolute.strip_prefix(root) else {
            continue;
        };
        let relative = relative_path
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/");
        if glob.is_match(&relative) {
            files.push((relative, absolute));
        }
    }
    files.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(files)
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
        .unwrap_or(100)
        .clamp(1, 500) as usize;
    let matcher = RegexMatcherBuilder::new()
        .build(query)
        .map_err(|_| CoreFailure {
            code: "invalid_arguments",
            message: "query is not a valid regular expression",
            retryable: false,
        })?;
    let files = project_files(root, pattern)?;
    let mut matches = Vec::new();
    let mut truncated = false;
    let mut searcher = SearcherBuilder::new()
        .line_number(true)
        .binary_detection(BinaryDetection::quit(0))
        .build();

    for (relative, absolute) in files {
        let Ok(metadata) = fs::metadata(&absolute) else {
            continue;
        };
        if metadata.len() > 2 * 1024 * 1024 {
            continue;
        }
        let Ok(bytes) = fs::read(&absolute) else {
            continue;
        };
        if bytes.len() > 2 * 1024 * 1024
            || bytes.iter().take(4096).any(|byte| *byte == 0)
            || std::str::from_utf8(&bytes).is_err()
        {
            continue;
        }
        let mut sink = MatchSink {
            matcher: &matcher,
            relative: &relative,
            matches: &mut matches,
            max_results,
            truncated: false,
        };
        if searcher.search_slice(&matcher, &bytes, &mut sink).is_err() {
            continue;
        }
        truncated |= sink.truncated;
        if truncated {
            break;
        }
    }
    Ok(json!({"query": query, "matches": matches, "truncated": truncated}))
}
