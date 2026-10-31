//! Model-facing argument translation and result normalization for the
//! built-in tools. Model arguments follow the OpenCode-compatible schemas in
//! `definitions`; the operations layer takes the internal shapes produced here.

use super::shell::{shell_command, validate_bash_arguments};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::path::Path;
use suncode_common::BusinessError;

#[cfg(test)]
pub(crate) fn translate_arguments(
    name: &str,
    value: &Value,
    scope_root: Option<&Path>,
) -> Result<Value, BusinessError> {
    let shell = suncode_common::shell::ResolvedShell {
        path: if cfg!(windows) {
            r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"
        } else {
            "/bin/sh"
        }
        .into(),
        family: if cfg!(windows) { "powershell" } else { "sh" }.into(),
        version: "test host".into(),
        interactive_args: vec![],
    };
    translate_arguments_with_shell(name, value, scope_root, Some(&shell))
}

pub(crate) fn translate_arguments_with_shell(
    name: &str,
    value: &Value,
    scope_root: Option<&Path>,
    shell: Option<&suncode_common::shell::ResolvedShell>,
) -> Result<Value, BusinessError> {
    let mut result = value.clone();
    if name == "bash" {
        validate_bash_arguments(&result)?;
    }
    if name == "webfetch" {
        validate_webfetch_arguments(&result)?;
    }
    if name == "write" {
        let content = result
            .get("content")
            .and_then(Value::as_str)
            .ok_or_else(|| BusinessError::new("invalid_arguments", "content is required"))?;
        result["content_base64"] = json!(STANDARD.encode(content));
        if let Some(object) = result.as_object_mut() {
            object.remove("content");
        }
    }
    if name == "edit" {
        let replacements = if let Some(edits) = result.get("edits").and_then(Value::as_array) {
            if edits.is_empty() {
                return Err(BusinessError::new(
                    "invalid_arguments",
                    "edits must not be empty",
                ));
            }
            edits
                .iter()
                .map(|edit| {
                    let object = edit.as_object().ok_or_else(|| {
                        BusinessError::new("invalid_arguments", "each edit must be an object")
                    })?;
                    let old = object
                        .get("oldText")
                        .and_then(Value::as_str)
                        .ok_or_else(|| {
                            BusinessError::new("invalid_arguments", "oldText is required")
                        })?;
                    let new = object
                        .get("newText")
                        .and_then(Value::as_str)
                        .ok_or_else(|| {
                            BusinessError::new("invalid_arguments", "newText is required")
                        })?;
                    Ok(json!({"old": old, "new": new, "replace_all": false}))
                })
                .collect::<Result<Vec<_>, BusinessError>>()?
        } else {
            let old = result
                .get("oldString")
                .and_then(Value::as_str)
                .ok_or_else(|| BusinessError::new("invalid_arguments", "oldString is required"))?;
            let new = result
                .get("newString")
                .and_then(Value::as_str)
                .ok_or_else(|| BusinessError::new("invalid_arguments", "newString is required"))?;
            let replace_all = result
                .get("replaceAll")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            vec![json!({"old": old, "new": new, "replace_all": replace_all})]
        };
        result["replacements"] = json!(replacements);
        if let Some(object) = result.as_object_mut() {
            object.remove("oldString");
            object.remove("newString");
            object.remove("replaceAll");
            object.remove("edits");
        }
    }
    if name == "glob" {
        if let Some(path) = result.get("path").and_then(Value::as_str) {
            let pattern = result
                .get("pattern")
                .and_then(Value::as_str)
                .ok_or_else(|| BusinessError::new("invalid_arguments", "pattern is required"))?;
            result["pattern"] = json!(scoped_glob(scope_root, path, pattern));
        }
        if let Some(limit) = result.get("limit").cloned() {
            result["max_results"] = limit;
        }
        if let Some(object) = result.as_object_mut() {
            object.remove("limit");
            object.remove("path");
        }
    }
    if name == "grep" {
        let query = result
            .get("pattern")
            .or_else(|| result.get("query"))
            .and_then(Value::as_str)
            .ok_or_else(|| BusinessError::new("invalid_arguments", "pattern is required"))?;
        result["query"] = json!(query);
        if let Some(include) = result.get("include").cloned() {
            result["pattern"] = include;
        } else {
            result["pattern"] = json!("**/*");
        }
        if let Some(path) = result.get("path").and_then(Value::as_str) {
            let pattern = result
                .get("pattern")
                .and_then(Value::as_str)
                .ok_or_else(|| BusinessError::new("invalid_arguments", "pattern is required"))?;
            result["pattern"] = json!(scoped_glob(scope_root, path, pattern));
        }
        if let Some(limit) = result.get("limit").cloned() {
            result["max_results"] = limit;
        }
        if let Some(object) = result.as_object_mut() {
            object.remove("include");
            object.remove("limit");
            object.remove("path");
        }
    }
    if name == "bash" {
        let command = result
            .get("command")
            .and_then(Value::as_str)
            .filter(|command| !command.trim().is_empty())
            .map(str::to_string)
            .ok_or_else(|| {
                BusinessError::new(
                    "invalid_arguments",
                    "bash command must be a non-empty string",
                )
            })?;
        let (program, args) = shell_command(&command, shell)?;
        result["program"] = json!(program);
        result["args"] = json!(args);
        if let Some(workdir) = result.get("workdir").or_else(|| result.get("cwd")).cloned() {
            result["cwd"] = workdir;
        }
        if let Some(timeout) = result.get("timeout").cloned() {
            result["timeout_ms"] = json!(timeout_millis(&timeout)?);
        }
        if let Some(object) = result.as_object_mut() {
            object.remove("command");
            object.remove("workdir");
            object.remove("timeout");
        }
    }
    Ok(result)
}

pub(crate) fn validate_webfetch_arguments(value: &Value) -> Result<(), BusinessError> {
    let raw_url = value
        .get("url")
        .and_then(Value::as_str)
        .filter(|url| !url.trim().is_empty())
        .ok_or_else(|| BusinessError::new("invalid_arguments", "url is required"))?;
    let url = url::Url::parse(raw_url).map_err(|_| {
        BusinessError::new("invalid_arguments", "url must be a valid HTTP or HTTPS URL")
    })?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(BusinessError::new(
            "invalid_arguments",
            "url must be a valid HTTP or HTTPS URL",
        ));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(BusinessError::new(
            "invalid_arguments",
            "url must not contain embedded credentials",
        ));
    }
    if let Some(format) = value.get("format") {
        if !matches!(format.as_str(), Some("text" | "markdown" | "html")) {
            return Err(BusinessError::new(
                "invalid_arguments",
                "format must be text, markdown, or html",
            ));
        }
    }
    if let Some(timeout) = value.get("timeout") {
        let seconds = timeout
            .as_f64()
            .filter(|value| value.is_finite())
            .ok_or_else(|| {
                BusinessError::new("invalid_arguments", "timeout must be a number of seconds")
            })?;
        if seconds <= 0.0 || seconds > 120.0 {
            return Err(BusinessError::new(
                "invalid_arguments",
                "timeout must be greater than zero and no more than 120 seconds",
            ));
        }
    }
    Ok(())
}

fn timeout_millis(value: &Value) -> Result<u64, BusinessError> {
    let milliseconds = value.as_u64().ok_or_else(|| {
        BusinessError::new(
            "invalid_arguments",
            "timeout must be a positive integer number of milliseconds",
        )
    })?;
    if milliseconds == 0 || milliseconds > 600_000 {
        return Err(BusinessError::new(
            "invalid_arguments",
            "timeout must be greater than zero and no more than 600000 milliseconds",
        ));
    }
    Ok(milliseconds)
}

pub(crate) fn normalize_result(name: &str, mut value: Value, dependency_id: Option<&str>) -> Value {
    if name == "read" {
        if let Some(encoded) = value
            .get("data_base64")
            .and_then(Value::as_str)
            .map(str::to_string)
        {
            if let Ok(bytes) = STANDARD.decode(&encoded) {
                if let Ok(text) = String::from_utf8(bytes) {
                    value["content"] = json!(text);
                    let complete = value.get("truncated").and_then(Value::as_bool) != Some(true)
                        && value.get("offset").and_then(Value::as_u64).unwrap_or(1) == 1
                        && value.get("limit").is_none();
                    if complete && value.get("precondition_base64").is_none() {
                        value["precondition_base64"] = json!(encoded);
                    }
                }
            }
        }
        if let Some(object) = value.as_object_mut() {
            object.remove("data_base64");
        }
    }
    if name == "bash" {
        for (encoded_key, text_key) in [("stdout_base64", "stdout"), ("stderr_base64", "stderr")] {
            let mut decoded_text = false;
            if let Some(encoded) = value
                .get(encoded_key)
                .and_then(Value::as_str)
                .map(str::to_string)
            {
                if let Ok(bytes) = STANDARD.decode(&encoded) {
                    if let Ok(text) = String::from_utf8(bytes) {
                        value[text_key] = json!(text);
                        decoded_text = true;
                    }
                }
            }
            if decoded_text {
                if let Some(object) = value.as_object_mut() {
                    object.remove(encoded_key);
                }
            } else if value.get(encoded_key).is_some() {
                value["binary_output"] = json!(true);
            }
        }
    }
    if let Some(dependency_id) = dependency_id {
        if matches!(
            name,
            "read" | "document_inspect" | "document_read" | "document_search" | "spreadsheet_read"
        ) {
            prefix_result_path(&mut value, "path", dependency_id);
        }
        if name == "glob" {
            if let Some(paths) = value.get_mut("paths").and_then(Value::as_array_mut) {
                for path in paths {
                    if let Some(relative) = path.as_str() {
                        *path = json!(dependency_alias(dependency_id, relative));
                    }
                }
            }
        }
        if name == "grep" {
            if let Some(matches) = value.get_mut("matches").and_then(Value::as_array_mut) {
                for matched in matches {
                    prefix_result_path(matched, "path", dependency_id);
                }
            }
        }
    }
    value
}

fn prefix_result_path(value: &mut Value, key: &str, dependency_id: &str) {
    let Some(relative) = value.get(key).and_then(Value::as_str) else {
        return;
    };
    value[key] = json!(dependency_alias(dependency_id, relative));
}

fn scoped_glob(scope_root: Option<&Path>, path: &str, pattern: &str) -> String {
    let base = path.trim_matches('/');
    if base.is_empty() || base == "." {
        return pattern.to_string();
    }
    let target_is_file = scope_root
        .map(|root| root.join(base).is_file())
        .unwrap_or(false);
    if target_is_file {
        return base.to_string();
    }
    let pattern = pattern.trim_start_matches('/');
    // The base is a literal directory; brackets in names such as `app/[id]`
    // must not become glob character classes.
    let base = escape_glob(base);
    if pattern == "**/*" || pattern.starts_with("**/") {
        format!("{base}/{pattern}")
    } else {
        format!("{base}/**/{pattern}")
    }
}

/// Escapes globset metacharacters so `value` matches only itself.
fn escape_glob(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        if matches!(character, '*' | '?' | '[' | ']' | '{' | '}') {
            escaped.push('[');
            escaped.push(character);
            escaped.push(']');
        } else {
            escaped.push(character);
        }
    }
    escaped
}

fn dependency_alias(dependency_id: &str, relative: &str) -> String {
    let relative = relative.trim_start_matches('/');
    if relative.is_empty() || relative == "." {
        format!("dependency:{dependency_id}")
    } else {
        format!("dependency:{dependency_id}/{relative}")
    }
}
