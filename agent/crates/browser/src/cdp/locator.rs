//! DOM lookup expressions for semantic [`Locator`]s.

use crate::action::Locator;

impl Locator {
    /// JavaScript expression evaluating to the matched element or a nullish value.
    pub(crate) fn expression(&self) -> String {
        match self {
            Self::Label(value) => format!(
                "[...document.querySelectorAll('label')].find(e=>e.innerText.trim()==={})?.control || document.querySelector({})",
                js_string(value),
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

    #[test]
    fn label_matches_label_text_then_aria_label() {
        let expression = Locator::Label("Email".into()).expression();
        assert!(expression.contains(r#"e.innerText.trim()==="Email")?.control"#));
        assert!(expression.contains(r#"[aria-label=\"Email\"]"#));
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
