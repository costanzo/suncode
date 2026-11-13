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
        validate_edit_arguments(&result)?;
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
                        .get("oldString")
                        .and_then(Value::as_str)
                        .ok_or_else(|| {
                            BusinessError::new("invalid_arguments", "oldString is required")
                        })?;
                    let new = object
                        .get("newString")
                        .and_then(Value::as_str)
                        .ok_or_else(|| {
                            BusinessError::new("invalid_arguments", "newString is required")
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

pub(crate) fn validate_edit_arguments(value: &Value) -> Result<(), BusinessError> {
    let invalid = |message| BusinessError::new("invalid_arguments", message);
    let object = value
        .as_object()
        .ok_or_else(|| invalid("edit arguments must be an object"))?;
    if object.contains_key("oldText") || object.contains_key("newText") {
        return Err(invalid(
            "use oldString/newString instead of oldText/newText, both at the top level and inside edits",
        ));
    }
    if object.keys().any(|key| {
        !matches!(
            key.as_str(),
            "path" | "oldString" | "newString" | "replaceAll" | "edits"
        )
    }) {
        return Err(invalid("unknown edit argument"));
    }
    if object
        .get("path")
        .and_then(Value::as_str)
        .is_none_or(|path| path.trim().is_empty())
    {
        return Err(invalid("path must be a non-empty string"));
    }
    if let Some(edits) = object.get("edits") {
        if ["oldString", "newString", "replaceAll"]
            .iter()
            .any(|key| object.contains_key(*key))
        {
            return Err(invalid(
                "use either edits or oldString/newString/replaceAll, never both",
            ));
        }
        let edits = edits
            .as_array()
            .ok_or_else(|| invalid("edits must be an array"))?;
        if edits.is_empty() || edits.len() > 200 {
            return Err(invalid("edits must contain between 1 and 200 entries"));
        }
        for (index, edit) in edits.iter().enumerate() {
            let valid = edit.as_object().is_some_and(|entry| {
                entry.len() == 2
                    && entry
                        .get("oldString")
                        .and_then(Value::as_str)
                        .is_some_and(|text| !text.is_empty())
                    && entry.get("newString").and_then(Value::as_str).is_some()
            });
            if !valid {
                return Err(invalid(
                    "each edit requires non-empty oldString and string newString, with no other fields",
                )
                .with_details(json!({"edit_index": index})));
            }
            validate_edit_text(
                edit["oldString"].as_str().unwrap(),
                edit["newString"].as_str().unwrap(),
            )?;
        }
    } else {
        if object
            .get("oldString")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
            || object.get("newString").and_then(Value::as_str).is_none()
        {
            return Err(invalid(
                "oldString must be non-empty and newString must be a string",
            ));
        }
        if object
            .get("replaceAll")
            .is_some_and(|value| !value.is_boolean())
        {
            return Err(invalid("replaceAll must be a boolean"));
        }
        validate_edit_text(
            object["oldString"].as_str().unwrap(),
            object["newString"].as_str().unwrap(),
        )?;
    }
    Ok(())
}

fn validate_edit_text(old: &str, new: &str) -> Result<(), BusinessError> {
    if old.contains('\0') || new.contains('\0') {
        return Err(BusinessError::invalid("edit text must not contain NUL"));
    }
    if old.len() > crate::text::MAX_EDIT_BYTES * 2 || new.len() > crate::text::MAX_EDIT_BYTES * 2 {
        return Err(BusinessError::new(
            "resource_limit",
            "replacement text exceeds 32 MiB",
        ));
    }
    Ok(())
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

pub(crate) fn normalize_result(
    name: &str,
    mut value: Value,
    dependency: Option<(&str, &Path)>,
) -> Value {
    if name == "read" {
        if let Some(encoded) = value
            .get("data_base64")
            .and_then(Value::as_str)
            .map(str::to_string)
        {
            if let Ok(bytes) = STANDARD.decode(&encoded) {
                if let Ok(text) = String::from_utf8(bytes) {
                    value["content"] = json!(text);
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
    if let Some((dependency_id, root)) = dependency {
        if matches!(
            name,
            "read"
                | "write"
                | "edit"
                | "delete"
                | "document_inspect"
                | "document_read"
                | "document_search"
                | "spreadsheet_read"
        ) {
            qualify_result_path(&mut value, "path", root);
        }
        if let Some(checkpoint) = value.get_mut("checkpoint") {
            if let Some(path) = checkpoint.get("path").and_then(Value::as_str) {
                checkpoint["path"] = json!(dependency_alias(dependency_id, path));
            }
        }
        if name == "glob" {
            if let Some(paths) = value.get_mut("paths").and_then(Value::as_array_mut) {
                for path in paths {
                    if let Some(relative) = path.as_str() {
                        *path = json!(root.join(relative).to_string_lossy());
                    }
                }
            }
        }
        if name == "grep" {
            if let Some(matches) = value.get_mut("matches").and_then(Value::as_array_mut) {
                for matched in matches {
                    qualify_result_path(matched, "path", root);
                }
            }
        }
    }
    value
}

fn qualify_result_path(value: &mut Value, key: &str, root: &Path) {
    let Some(relative) = value.get(key).and_then(Value::as_str) else {
        return;
    };
    value[key] = json!(root.join(relative).to_string_lossy());
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
