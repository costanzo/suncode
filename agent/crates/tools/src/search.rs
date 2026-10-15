use crate::arguments::{GlobArguments, GrepArguments};
use crate::error::fatal;
use crate::scope::require_project;
use globset::{GlobBuilder, GlobMatcher};
use grep_matcher::Matcher;
use grep_regex::{RegexMatcher, RegexMatcherBuilder};
use grep_searcher::{BinaryDetection, SearcherBuilder, Sink, SinkMatch};
use ignore::WalkBuilder;
use serde_json::{json, Value};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use suncode_common::BusinessError;

pub(crate) fn glob(
    project_root: Option<&Path>,
    args: &GlobArguments,
) -> Result<Value, BusinessError> {
    let root = require_project(project_root)?;
    let pattern = args.pattern.as_str();
    if pattern.is_empty()
        || Path::new(pattern).is_absolute()
        || pattern.split('/').any(|part| part == "..")
    {
        return Err(fatal(
            "scope_denied",
            "pattern must remain inside the project",
        ));
    }
    let matcher = compile_glob(pattern, "pattern is invalid")?;
    let max_results = args.max_results.unwrap_or(100).clamp(1, 1000);
    let mut paths = Vec::new();
    for (relative, _) in project_walk(root) {
        if matcher.is_match(&relative) {
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

fn project_files(root: &Path, pattern: &str) -> Result<Vec<(String, PathBuf)>, BusinessError> {
    let matcher = compile_glob(pattern, "include pattern is invalid")?;
    let mut files = project_walk(root)
        .filter(|(relative, _)| matcher.is_match(relative))
        .collect::<Vec<_>>();
    files.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(files)
}

/// Compiles a project-relative glob where `*` does not cross `/`.
fn compile_glob(
    pattern: &str,
    invalid_message: &'static str,
) -> Result<GlobMatcher, BusinessError> {
    GlobBuilder::new(pattern)
        .literal_separator(true)
        .build()
        .map(|glob| glob.compile_matcher())
        .map_err(|_| fatal("invalid_arguments", invalid_message))
}

/// Regular files under `root` honoring ignore files, without following links,
/// as `(forward-slash relative path, absolute path)` pairs in walk order.
fn project_walk(root: &Path) -> impl Iterator<Item = (String, PathBuf)> + '_ {
    WalkBuilder::new(root)
        .standard_filters(true)
        .follow_links(false)
        .build()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_some_and(|kind| kind.is_file()))
        .filter_map(move |entry| {
            let absolute = entry.into_path();
            let relative = absolute
                .strip_prefix(root)
                .ok()?
                .to_string_lossy()
                .replace(std::path::MAIN_SEPARATOR, "/");
            Some((relative, absolute))
        })
}

pub(crate) fn find(
    project_root: Option<&Path>,
    args: &GrepArguments,
) -> Result<Value, BusinessError> {
    let root = require_project(project_root)?;
    let query = args.query.as_str();
    if query.is_empty() || query.len() > 256 {
        return Err(fatal("invalid_arguments", "query must contain 1-256 bytes"));
    }
    let pattern = args.pattern.as_str();
    if pattern.is_empty()
        || Path::new(pattern).is_absolute()
        || pattern.split('/').any(|part| part == "..")
    {
        return Err(fatal(
            "scope_denied",
            "pattern must remain inside the project",
        ));
    }
    let max_results = args.max_results.unwrap_or(100).clamp(1, 500);
    let matcher = RegexMatcherBuilder::new().build(query).map_err(|_| {
        fatal(
            "invalid_arguments",
            "query is not a valid regular expression",
        )
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
