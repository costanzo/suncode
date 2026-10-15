//! The model-facing `browser_*` tool contract: definitions, argument
//! validation, and mapping to CDP session requests.

mod definitions;

use serde_json::Value;
use suncode_common::BusinessError;

#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: Value,
}

pub fn definitions() -> Vec<ToolDefinition> {
    definitions::definitions()
        .into_iter()
        .map(|(name, description, parameters)| ToolDefinition {
            name,
            description,
            parameters,
        })
        .collect()
}

/// CDP session method implementing a `browser_*` tool.
pub(crate) fn session_method(name: &str) -> Result<&'static str, BusinessError> {
    match name {
        "browser_open" => Ok("open"),
        "browser_snapshot" => Ok("snapshot"),
        "browser_navigate" => Ok("navigate"),
        "browser_click" => Ok("click"),
        "browser_fill" => Ok("fill"),
        "browser_press" => Ok("press"),
        "browser_tabs" => Ok("state"),
        "browser_screenshot" => Ok("screenshot"),
        "browser_close_page" => Ok("close_page"),
        _ => Err(BusinessError::new(
            "browser_tool_unknown",
            "Browser tool is not supported",
        )),
    }
}

pub fn is_browser_tool(name: &str) -> bool {
    name.starts_with("browser_")
}

/// Validates model-supplied arguments before approval and execution.
pub fn validate_browser_arguments(name: &str, arguments: &Value) -> Result<(), BusinessError> {
    let object = arguments
        .as_object()
        .ok_or_else(|| BusinessError::invalid("Browser tool arguments must be an object"))?;
    if matches!(name, "browser_click" | "browser_fill" | "browser_press")
        && !object.get("target").is_some_and(Value::is_object)
    {
        return Err(BusinessError::invalid("target is required"));
    }
    if name == "browser_fill" && !object.get("value").is_some_and(Value::is_string) {
        return Err(BusinessError::invalid("value is required"));
    }
    if name == "browser_press"
        && object
            .get("key")
            .and_then(Value::as_str)
            .is_none_or(|value| value.is_empty() || value.chars().count() > 100)
    {
        return Err(BusinessError::invalid(
            "key is required and must be at most 100 characters",
        ));
    }
    session_params(name, arguments).map(|_| ())
}

/// Maps tool arguments to CDP session request parameters.
pub(crate) fn session_params(name: &str, arguments: &Value) -> Result<Value, BusinessError> {
    let object = arguments
        .as_object()
        .ok_or_else(|| BusinessError::invalid("Browser tool arguments must be an object"))?;
    let mut params = serde_json::Map::new();
    if let Some(page_id) = object.get("page_id") {
        params.insert("pageId".into(), page_id.clone());
    }
    if let Some(target) = object.get("target") {
        params.insert("target".into(), target.clone());
    }
    if let Some(value) = object.get("value") {
        params.insert("value".into(), value.clone());
    }
    if let Some(key) = object.get("key") {
        params.insert("key".into(), key.clone());
    }
    if let Some(timeout) = object.get("timeout_ms") {
        params.insert("timeoutMs".into(), timeout.clone());
    }
    if let Some(full_page) = object.get("full_page") {
        params.insert("fullPage".into(), full_page.clone());
    }
    if let Some(url) = object.get("url").and_then(Value::as_str) {
        validate_browser_url(url)?;
        params.insert("url".into(), Value::String(url.into()));
    } else if name == "browser_navigate" {
        return Err(BusinessError::invalid("url is required"));
    }
    Ok(Value::Object(params))
}

fn validate_browser_url(value: &str) -> Result<(), BusinessError> {
    let url = url::Url::parse(value)
        .map_err(|_| BusinessError::invalid("Browser URL must be an absolute HTTP or HTTPS URL"))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(BusinessError::invalid(
            "Browser URL must be an absolute HTTP or HTTPS URL",
        ));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(BusinessError::invalid(
            "Browser URL must not contain embedded credentials",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn definitions_cover_every_session_method() {
        let names = definitions()
            .into_iter()
            .map(|definition| definition.name)
            .collect::<BTreeSet<_>>();
        assert_eq!(
            names,
            BTreeSet::from([
                "browser_click",
                "browser_close_page",
                "browser_fill",
                "browser_navigate",
                "browser_open",
                "browser_press",
                "browser_screenshot",
                "browser_snapshot",
                "browser_tabs",
            ])
        );
        assert!(names.iter().all(|name| session_method(name).is_ok()));
    }

    #[test]
    fn browser_urls_reject_credentials_and_non_http_schemes() {
        assert!(validate_browser_url("https://example.test/path").is_ok());
        assert!(validate_browser_url("http://127.0.0.1:5173").is_ok());
        assert!(validate_browser_url("file:///etc/passwd").is_err());
        assert!(validate_browser_url("https://user:secret@example.test").is_err());
    }
}
