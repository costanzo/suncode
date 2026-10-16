//! The model-facing `browser_*` tool contract: definitions and parsing of tool
//! arguments into typed [`BrowserAction`]s.

mod definitions;

use crate::action::{BrowserAction, Locator};
use crate::cdp::error::TOOL_UNKNOWN;
use serde_json::{Map, Value};
use suncode_common::BusinessError;

const MAX_KEY_CHARS: usize = 100;

pub use suncode_common::tool::ToolDefinition;

pub fn definitions() -> Vec<ToolDefinition> {
    definitions::definitions()
        .into_iter()
        .map(|(name, description, parameters)| ToolDefinition {
            name: name.into(),
            description: description.into(),
            parameters,
        })
        .collect()
}

pub fn is_browser_tool(name: &str) -> bool {
    name.starts_with("browser_")
}

/// Validates model-supplied arguments before approval and execution.
pub fn validate_browser_arguments(name: &str, arguments: &Value) -> Result<(), BusinessError> {
    parse_call(name, arguments).map(|_| ())
}

/// Parses one tool call into the action it performs.
pub(crate) fn parse_call(name: &str, arguments: &Value) -> Result<BrowserAction, BusinessError> {
    let arguments = arguments
        .as_object()
        .ok_or_else(|| BusinessError::invalid("Browser tool arguments must be an object"))?;
    match name {
        "browser_open" => match optional_str(arguments, "url")? {
            Some(url) => navigate(url),
            None => Ok(BrowserAction::State),
        },
        "browser_navigate" => navigate(
            optional_str(arguments, "url")?
                .ok_or_else(|| BusinessError::invalid("url is required"))?,
        ),
        "browser_snapshot" => Ok(BrowserAction::Snapshot),
        "browser_tabs" => Ok(BrowserAction::State),
        "browser_screenshot" => Ok(BrowserAction::Screenshot {
            full_page: arguments
                .get("full_page")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        }),
        "browser_close_page" => Ok(BrowserAction::ClosePage),
        "browser_click" => Ok(BrowserAction::Click {
            target: parse_locator(arguments.get("target"))?,
        }),
        "browser_fill" => Ok(BrowserAction::Fill {
            target: parse_locator(arguments.get("target"))?,
            value: optional_str(arguments, "value")?
                .ok_or_else(|| BusinessError::invalid("value is required"))?
                .to_owned(),
        }),
        "browser_press" => {
            let key = optional_str(arguments, "key")?.unwrap_or_default();
            if key.is_empty() || key.chars().count() > MAX_KEY_CHARS {
                return Err(BusinessError::invalid(format!(
                    "key is required and must be at most {MAX_KEY_CHARS} characters"
                )));
            }
            Ok(BrowserAction::Press {
                key: key.to_owned(),
            })
        }
        _ => Err(BusinessError::new(
            TOOL_UNKNOWN,
            "Browser tool is not supported",
        )),
    }
}

fn navigate(url: &str) -> Result<BrowserAction, BusinessError> {
    validate_browser_url(url)?;
    Ok(BrowserAction::Navigate {
        url: url.to_owned(),
    })
}

fn optional_str<'a>(
    arguments: &'a Map<String, Value>,
    key: &str,
) -> Result<Option<&'a str>, BusinessError> {
    match arguments.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value)),
        Some(_) => Err(BusinessError::invalid(format!("{key} must be a string"))),
    }
}

fn parse_locator(target: Option<&Value>) -> Result<Locator, BusinessError> {
    let target = target
        .and_then(Value::as_object)
        .ok_or_else(|| BusinessError::invalid("target is required"))?;
    let text = |key: &str| {
        optional_str(target, key)?
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .ok_or_else(|| BusinessError::invalid(format!("target {key} is required")))
    };
    match optional_str(target, "kind")? {
        Some("label") => Ok(Locator::Label(text("value")?)),
        Some("placeholder") => Ok(Locator::Placeholder(text("value")?)),
        Some("test_id") => Ok(Locator::TestId(text("value")?)),
        Some("text") => Ok(Locator::Text(text("value")?)),
        Some("role") => Ok(Locator::Role {
            role: text("role")?,
            name: text("name")?,
        }),
        Some(_) => Err(BusinessError::invalid("unsupported target kind")),
        None => Err(BusinessError::invalid("target kind is required")),
    }
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
mod tests;
