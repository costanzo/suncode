use super::*;
use serde_json::json;
use std::collections::BTreeSet;

fn schema(name: &str) -> Value {
    definitions()
        .into_iter()
        .find(|definition| definition.name == name)
        .unwrap()
        .parameters
}

fn schema_accepts(name: &str, arguments: &Value) -> bool {
    jsonschema::validator_for(&schema(name))
        .unwrap()
        .is_valid(arguments)
}

#[test]
fn definitions_are_valid_and_parseable() {
    let names = definitions()
        .into_iter()
        .map(|definition| {
            assert!(
                jsonschema::validator_for(&definition.parameters).is_ok(),
                "invalid schema for {}",
                definition.name
            );
            definition.name
        })
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
    for name in [
        "browser_open",
        "browser_snapshot",
        "browser_tabs",
        "browser_close_page",
    ] {
        assert!(
            parse_call(name, &json!({})).is_ok(),
            "{name} rejects no arguments"
        );
    }
}

/// Every example the schema accepts must parse, and vice versa, so the model
/// is never offered arguments the runtime refuses.
#[test]
fn schema_and_parser_agree() {
    let cases = [
        ("browser_open", json!({}), true),
        ("browser_open", json!({"url": "https://example.test"}), true),
        ("browser_navigate", json!({}), false),
        (
            "browser_navigate",
            json!({"url": "https://example.test", "page_id": "p"}),
            false,
        ),
        ("browser_screenshot", json!({"full_page": true}), true),
        (
            "browser_click",
            json!({"target": {"kind": "label", "value": "Email"}}),
            true,
        ),
        (
            "browser_click",
            json!({"target": {"kind": "role", "role": "button", "name": "Save"}}),
            true,
        ),
        (
            "browser_click",
            json!({"target": {"kind": "role", "name": "Save"}}),
            false,
        ),
        (
            "browser_click",
            json!({"target": {"kind": "ref", "ref": "e1", "revision": 1}}),
            false,
        ),
        (
            "browser_click",
            json!({"target": {"kind": "css", "value": "body"}}),
            false,
        ),
        (
            "browser_fill",
            json!({"target": {"kind": "placeholder", "value": "Search"}}),
            false,
        ),
        (
            "browser_fill",
            json!({"target": {"kind": "test_id", "value": "q"}, "value": "x"}),
            true,
        ),
        ("browser_press", json!({"key": "Enter"}), true),
        ("browser_press", json!({"key": ""}), false),
    ];
    for (name, arguments, valid) in cases {
        assert_eq!(
            schema_accepts(name, &arguments),
            valid,
            "schema {name} {arguments}"
        );
        // The parser ignores unknown properties; the schema rejects them.
        if arguments.get("page_id").is_none() {
            assert_eq!(
                parse_call(name, &arguments).is_ok(),
                valid,
                "parser {name} {arguments}"
            );
        }
    }
}

#[test]
fn parses_typed_actions() {
    assert_eq!(
        parse_call("browser_open", &json!({"url": "https://example.test/a"})).unwrap(),
        BrowserAction::Navigate {
            url: "https://example.test/a".into()
        }
    );
    assert_eq!(
        parse_call("browser_open", &json!({})).unwrap(),
        BrowserAction::State
    );
    assert_eq!(
        parse_call(
            "browser_fill",
            &json!({"target": {"kind": "role", "role": "textbox", "name": "Email"}, "value": "a@b"})
        )
        .unwrap(),
        BrowserAction::Fill {
            target: Locator::Role {
                role: "textbox".into(),
                name: "Email".into()
            },
            value: "a@b".into()
        }
    );
    assert_eq!(
        parse_call("browser_screenshot", &json!({})).unwrap(),
        BrowserAction::Screenshot { full_page: false }
    );
}

#[test]
fn rejects_unknown_tools_and_non_object_arguments() {
    assert_eq!(
        parse_call("browser_evaluate", &json!({})).unwrap_err().code,
        TOOL_UNKNOWN
    );
    assert!(parse_call("browser_tabs", &json!([])).is_err());
    assert!(parse_call("browser_navigate", &json!({"url": 1})).is_err());
}

#[test]
fn browser_urls_reject_credentials_and_non_http_schemes() {
    assert!(validate_browser_url("https://example.test/path").is_ok());
    assert!(validate_browser_url("http://127.0.0.1:5173").is_ok());
    assert!(validate_browser_url("file:///etc/passwd").is_err());
    assert!(validate_browser_url("https://user:secret@example.test").is_err());
}
