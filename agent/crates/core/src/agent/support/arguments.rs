//! Validation for the core's turn-level tools.

use super::super::*;

pub(in crate::agent) fn tool_signature(call: &ToolCall) -> String {
    format!(
        "{}:{}",
        call.name,
        serde_json::to_string(&call.arguments).unwrap_or_default()
    )
}

pub(in crate::agent) fn validate_before_policy(
    name: &str,
    value: &Value,
) -> Result<(), BusinessError> {
    if name == "question" {
        return validate_question_arguments(value);
    }
    if name == "todowrite" {
        return validate_todowrite_arguments(value);
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
