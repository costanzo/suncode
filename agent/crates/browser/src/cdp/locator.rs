//! Element locators and their DOM lookup expressions.
//!
//! Locators are deliberately semantic (label, placeholder, test id, text, role).
//! Arbitrary CSS selectors are rejected so the agent targets what users see.

use crate::cdp::error::REFERENCE_STALE;
use serde_json::Value;
use suncode_common::BusinessError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Locator {
    Label(String),
    Placeholder(String),
    TestId(String),
    Text(String),
    Role { role: String, name: String },
}

impl Locator {
    /// Parses the `target` tool parameter: `{"kind": ..., "value"|"name": ...}`.
    pub(crate) fn parse(target: Option<&Value>) -> Result<Self, BusinessError> {
        let target = target
            .and_then(Value::as_object)
            .ok_or_else(|| BusinessError::invalid("target is required"))?;
        let kind = target
            .get("kind")
            .and_then(Value::as_str)
            .ok_or_else(|| BusinessError::invalid("target kind is required"))?;
        if kind == "ref" {
            return Err(BusinessError::new(
                REFERENCE_STALE,
                "CDP element references must be refreshed from the current snapshot",
            ));
        }
        let value = target
            .get("value")
            .and_then(Value::as_str)
            .or_else(|| target.get("name").and_then(Value::as_str))
            .ok_or_else(|| BusinessError::invalid("target value is required"))?
            .to_owned();
        match kind {
            "label" => Ok(Self::Label(value)),
            "placeholder" => Ok(Self::Placeholder(value)),
            "test_id" => Ok(Self::TestId(value)),
            "text" => Ok(Self::Text(value)),
            "role" => Ok(Self::Role {
                role: target
                    .get("role")
                    .and_then(Value::as_str)
                    .unwrap_or("button")
                    .to_owned(),
                name: value,
            }),
            _ => Err(BusinessError::invalid("unsupported target kind")),
        }
    }

    /// JavaScript expression evaluating to the matched element or a nullish value.
    pub(crate) fn expression(&self) -> String {
        match self {
            Self::Label(value) => format!(
                "document.querySelector({})?.control || document.querySelector({})",
                attribute_selector("label", "for", value),
                attribute_selector("", "aria-label", value),
            ),
            Self::Placeholder(value) => format!(
                "document.querySelector({})",
                attribute_selector("", "placeholder", value)
            ),
            Self::TestId(value) => format!(
                "document.querySelector({})",
                attribute_selector("", "data-testid", value)
            ),
            Self::Text(value) => format!(
                "[...document.querySelectorAll('button,a,input,textarea,select')].find(e=>e.innerText.trim()==={})",
                js_string(value)
            ),
            Self::Role { role, name } => format!(
                "[...document.querySelectorAll('[role]')].find(e=>e.getAttribute('role')==={} && e.innerText.trim()==={})",
                js_string(role),
                js_string(name)
            ),
        }
    }
}

/// JSON string literal, which is also a valid JavaScript string literal.
pub(crate) fn js_string(value: &str) -> String {
    serde_json::to_string(value).expect("string serialization is infallible")
}

/// JS literal for `element[attribute="value"]` with the value CSS-escaped.
fn attribute_selector(element: &str, attribute: &str, value: &str) -> String {
    js_string(&format!("{element}[{attribute}=\"{}\"]", css_string(value)))
}

/// Escapes text for a double-quoted CSS string (CSS Syntax §4.3.5).
fn css_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' | '\\' => {
                escaped.push('\\');
                escaped.push(character);
            }
            '\0' => escaped.push('\u{FFFD}'),
            control if control.is_control() => {
                escaped.push_str(&format!("\\{:x} ", control as u32));
            }
            other => escaped.push(other),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rejects_unbounded_selectors() {
        assert!(Locator::parse(Some(&json!({"css": "body"}))).is_err());
        assert!(Locator::parse(Some(&json!({"kind": "css", "value": "body"}))).is_err());
    }

    #[test]
    fn rejects_stale_references() {
        let error = Locator::parse(Some(&json!({"kind": "ref", "value": "e1"}))).unwrap_err();
        assert_eq!(error.code, REFERENCE_STALE);
    }

    #[test]
    fn role_defaults_to_button_and_accepts_name() {
        let locator = Locator::parse(Some(&json!({"kind": "role", "name": "Save"}))).unwrap();
        assert_eq!(
            locator,
            Locator::Role {
                role: "button".into(),
                name: "Save".into()
            }
        );
    }

    #[test]
    fn attribute_values_are_quoted_and_escaped() {
        let expression = Locator::Placeholder(r#"Search "docs" \ here"#.into()).expression();
        assert_eq!(
            expression,
            r#"document.querySelector("[placeholder=\"Search \\\"docs\\\" \\\\ here\"]")"#
        );
    }

    #[test]
    fn css_string_escapes_control_characters() {
        assert_eq!(css_string("a\nb"), "a\\a b");
    }
}
