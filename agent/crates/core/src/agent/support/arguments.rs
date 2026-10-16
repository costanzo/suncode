//! Tool-call argument translation, validation, and result normalization.

use super::super::*;
use super::dependencies::*;
use super::shell::*;

pub(in crate::agent) fn method_name(name: &str) -> Option<&'static str> {
    match name {
        "read" => Some("tool/read"),
        "glob" => Some("tool/glob"),
        "grep" => Some("tool/grep"),
        "write" => Some("tool/write"),
        "edit" => Some("tool/edit"),
        "bash" => Some("tool/bash"),
        "webfetch" => Some("tool/webfetch"),
        _ => None,
    }
}

pub(in crate::agent) fn tool_signature(call: &ToolCall) -> String {
    format!(
        "{}:{}",
        call.name,
        serde_json::to_string(&call.arguments).unwrap_or_default()
    )
}

pub(in crate::agent) fn translate_arguments(
    name: &str,
    value: &Value,
) -> Result<Value, BusinessError> {
    translate_arguments_with_root(name, value, None)
}

pub(in crate::agent) fn translate_arguments_with_root(
    name: &str,
    value: &Value,
    scope_root: Option<&Path>,
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
        let (program, args) = shell_command(&command)?;
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

pub(in crate::agent) fn validate_before_policy(
    name: &str,
    value: &Value,
) -> Result<(), BusinessError> {
    if name == "bash" {
        return validate_bash_arguments(value);
    }
    if lsp::is_lsp_tool(name) {
        return validate_lsp_arguments(name, value);
    }
    if name == "question" {
        return validate_question_arguments(value);
    }
    if name == "todowrite" {
        return validate_todowrite_arguments(value);
    }
    if name == "webfetch" {
        validate_webfetch_arguments(value)?;
    }
    Ok(())
}

pub(in crate::agent) fn validate_lsp_arguments(
    name: &str,
    value: &Value,
) -> Result<(), BusinessError> {
    let path = value
        .get("path")
        .and_then(Value::as_str)
        .filter(|path| !path.trim().is_empty() && path.chars().count() <= 4_096)
        .ok_or_else(|| {
            BusinessError::invalid("path is required and must be at most 4096 characters")
        })?;
    if path.contains('\0') {
        return Err(BusinessError::invalid("path contains an invalid character"));
    }
    if let Some(language_id) = value.get("languageId").and_then(Value::as_str) {
        if language_id.is_empty()
            || language_id.chars().count() > 64
            || !language_id
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "_.+-".contains(character))
        {
            return Err(BusinessError::invalid("languageId is invalid"));
        }
    }
    if matches!(name, "lsp_definition" | "lsp_references" | "lsp_hover") {
        for key in ["line", "column"] {
            if value
                .get(key)
                .and_then(Value::as_u64)
                .is_none_or(|value| value == 0)
            {
                return Err(BusinessError::invalid(format!(
                    "{key} must be a one-based positive integer"
                )));
            }
        }
    }
    Ok(())
}

pub(in crate::agent) fn validate_todowrite_arguments(value: &Value) -> Result<(), BusinessError> {
    let todos = value
        .get("todos")
        .and_then(Value::as_array)
        .filter(|items| items.len() <= 100)
        .ok_or_else(|| {
            BusinessError::new(
                "invalid_arguments",
                "todos must be an array of at most 100 items",
            )
        })?;
    let mut in_progress = 0;
    for todo in todos {
        let object = todo.as_object().ok_or_else(|| {
            BusinessError::new("invalid_arguments", "each todo must be an object")
        })?;
        let content = object
            .get("content")
            .and_then(Value::as_str)
            .filter(|content| !content.trim().is_empty())
            .ok_or_else(|| BusinessError::new("invalid_arguments", "todo content is required"))?;
        if content.chars().count() > 500 {
            return Err(BusinessError::new(
                "invalid_arguments",
                "todo content must be at most 500 characters",
            ));
        }
        match object.get("status").and_then(Value::as_str) {
            Some("pending" | "completed" | "cancelled") => {}
            Some("in_progress") => in_progress += 1,
            _ => {
                return Err(BusinessError::new(
                    "invalid_arguments",
                    "todo status must be pending, in_progress, completed, or cancelled",
                ))
            }
        }
        if !matches!(
            object.get("priority").and_then(Value::as_str),
            Some("high" | "medium" | "low")
        ) {
            return Err(BusinessError::new(
                "invalid_arguments",
                "todo priority must be high, medium, or low",
            ));
        }
    }
    if in_progress > 1 {
        return Err(BusinessError::new(
            "invalid_arguments",
            "only one todo may be in_progress",
        ));
    }
    Ok(())
}

pub(in crate::agent) fn parse_todos(value: &Value) -> Result<Vec<TodoEntry>, BusinessError> {
    validate_todowrite_arguments(value)?;
    serde_json::from_value(
        value
            .get("todos")
            .cloned()
            .ok_or_else(|| BusinessError::new("invalid_arguments", "todos is required"))?,
    )
    .map_err(|_| BusinessError::new("invalid_arguments", "todos contain invalid values"))
}

pub(in crate::agent) fn validate_question_arguments(value: &Value) -> Result<(), BusinessError> {
    let questions = value
        .get("questions")
        .and_then(Value::as_array)
        .filter(|items| !items.is_empty() && items.len() <= 8)
        .ok_or_else(|| {
            BusinessError::new("invalid_arguments", "questions must contain 1 to 8 items")
        })?;
    for question in questions {
        let object = question.as_object().ok_or_else(|| {
            BusinessError::new("invalid_arguments", "each question must be an object")
        })?;
        for field in ["question", "header"] {
            if object
                .get(field)
                .and_then(Value::as_str)
                .is_none_or(str::is_empty)
            {
                return Err(BusinessError::new(
                    "invalid_arguments",
                    format!("{field} is required"),
                ));
            }
        }
        if object
            .get("header")
            .and_then(Value::as_str)
            .is_some_and(|value| value.chars().count() > 30)
        {
            return Err(BusinessError::new(
                "invalid_arguments",
                "header must be at most 30 characters",
            ));
        }
        let options = object
            .get("options")
            .and_then(Value::as_array)
            .filter(|items| !items.is_empty() && items.len() <= 12)
            .ok_or_else(|| {
                BusinessError::new("invalid_arguments", "options must contain 1 to 12 items")
            })?;
        let mut labels = std::collections::HashSet::new();
        for option in options {
            let option = option.as_object().ok_or_else(|| {
                BusinessError::new("invalid_arguments", "each option must be an object")
            })?;
            let label = option
                .get("label")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    BusinessError::new("invalid_arguments", "option label is required")
                })?;
            if !labels.insert(label) {
                return Err(BusinessError::new(
                    "invalid_arguments",
                    "option labels must be unique",
                ));
            }
            if option
                .get("description")
                .and_then(Value::as_str)
                .is_none_or(str::is_empty)
            {
                return Err(BusinessError::new(
                    "invalid_arguments",
                    "option description is required",
                ));
            }
        }
    }
    Ok(())
}

pub(in crate::agent) fn validate_question_answers(
    arguments: &Value,
    answers: &[Vec<String>],
) -> Result<(), BusinessError> {
    let questions = arguments
        .get("questions")
        .and_then(Value::as_array)
        .ok_or_else(|| BusinessError::new("invalid_arguments", "questions are missing"))?;
    if answers.len() != questions.len() {
        return Err(BusinessError::new(
            "invalid_arguments",
            "one answer list is required for each question",
        ));
    }
    for (question, values) in questions.iter().zip(answers) {
        let options = question
            .get("options")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                BusinessError::new("invalid_arguments", "question options are missing")
            })?;
        let allowed = options
            .iter()
            .filter_map(|option| option.get("label").and_then(Value::as_str))
            .collect::<std::collections::HashSet<_>>();
        if question.get("multiple").and_then(Value::as_bool) != Some(true) && values.len() > 1 {
            return Err(BusinessError::new(
                "invalid_arguments",
                "this question accepts only one answer",
            ));
        }
        let custom = question
            .get("custom")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        for value in values {
            if value.trim().is_empty() || (!custom && !allowed.contains(value.as_str())) {
                return Err(BusinessError::new(
                    "invalid_arguments",
                    "an answer is not allowed for this question",
                ));
            }
        }
    }
    Ok(())
}

pub(in crate::agent) fn validate_webfetch_arguments(value: &Value) -> Result<(), BusinessError> {
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

pub(in crate::agent) fn timeout_millis(value: &Value) -> Result<u64, BusinessError> {
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

pub(in crate::agent) fn normalize_result(
    name: &str,
    mut value: Value,
    dependency_id: Option<&str>,
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
        if name == "read" {
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

pub(in crate::agent) fn prefix_result_path(value: &mut Value, key: &str, dependency_id: &str) {
    let Some(relative) = value.get(key).and_then(Value::as_str) else {
        return;
    };
    value[key] = json!(dependency_alias(dependency_id, relative));
}
