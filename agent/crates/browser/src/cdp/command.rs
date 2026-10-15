//! Typed browser commands parsed from the `(method, params)` wire request.

use crate::cdp::error::TOOL_UNKNOWN;
use crate::cdp::locator::Locator;
use serde_json::Value;
use suncode_common::BusinessError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BrowserCommand {
    /// `start`, `state`, `take_control`, and `return_control` all report state;
    /// CEF control handoff is owned by the desktop host.
    State,
    Probe,
    Navigate {
        url: String,
    },
    Snapshot,
    Screenshot {
        full_page: bool,
    },
    Click {
        target: Locator,
    },
    Fill {
        target: Locator,
        value: String,
    },
    Press {
        key: String,
    },
    ClosePage,
}

impl BrowserCommand {
    pub(crate) fn parse(method: &str, params: &Value) -> Result<Self, BusinessError> {
        match method {
            "start" | "state" | "take_control" | "return_control" => Ok(Self::State),
            "probe" => Ok(Self::Probe),
            "open" | "navigate" => Ok(Self::Navigate {
                url: required_str(params, "url")?,
            }),
            "snapshot" => Ok(Self::Snapshot),
            "screenshot" => Ok(Self::Screenshot {
                full_page: params
                    .get("fullPage")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            }),
            "click" => Ok(Self::Click {
                target: Locator::parse(params.get("target"))?,
            }),
            "fill" => Ok(Self::Fill {
                target: Locator::parse(params.get("target"))?,
                value: required_str(params, "value")?,
            }),
            "press" => Ok(Self::Press {
                key: required_str(params, "key")?,
            }),
            "close_page" => Ok(Self::ClosePage),
            _ => Err(BusinessError::new(
                TOOL_UNKNOWN,
                "Browser CDP command is not supported",
            )),
        }
    }
}

fn required_str(params: &Value, key: &str) -> Result<String, BusinessError> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| BusinessError::invalid(format!("{key} is required")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn state_aliases_share_one_command() {
        for method in ["start", "state", "take_control", "return_control"] {
            assert_eq!(
                BrowserCommand::parse(method, &json!({})).unwrap(),
                BrowserCommand::State
            );
        }
    }

    #[test]
    fn navigate_requires_url() {
        let error = BrowserCommand::parse("navigate", &json!({})).unwrap_err();
        assert_eq!(error.message, "url is required");
        assert_eq!(
            BrowserCommand::parse("open", &json!({"url": "https://example.com"})).unwrap(),
            BrowserCommand::Navigate {
                url: "https://example.com".into()
            }
        );
    }

    #[test]
    fn fill_requires_target_and_value() {
        let target = json!({"kind": "label", "value": "Email"});
        assert!(BrowserCommand::parse("fill", &json!({"target": target})).is_err());
        assert!(BrowserCommand::parse("fill", &json!({"value": "x"})).is_err());
        assert!(BrowserCommand::parse("fill", &json!({"target": target, "value": "x"})).is_ok());
    }

    #[test]
    fn screenshot_defaults_to_viewport() {
        assert_eq!(
            BrowserCommand::parse("screenshot", &json!({})).unwrap(),
            BrowserCommand::Screenshot { full_page: false }
        );
    }

    #[test]
    fn unknown_methods_are_rejected() {
        let error = BrowserCommand::parse("evaluate", &json!({})).unwrap_err();
        assert_eq!(error.code, TOOL_UNKNOWN);
    }
}
