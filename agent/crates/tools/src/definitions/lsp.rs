use serde_json::{json, Value};

fn path_properties() -> Value {
    json!({
        "path": {"type":"string","description":"Project-relative file path or dependency:<dependencyId>/<path>"},
        "languageId": {"type":"string","description":"Optional LSP language id override"}
    })
}

fn position_properties() -> Value {
    let mut properties = path_properties().as_object().cloned().unwrap_or_default();
    properties.insert(
        "line".into(),
        json!({"type":"integer","minimum":1,"description":"One-based source line"}),
    );
    properties.insert(
        "column".into(),
        json!({"type":"integer","minimum":1,"description":"One-based Unicode character column"}),
    );
    Value::Object(properties)
}

pub(super) fn diagnostics() -> (&'static str, &'static str, Value) {
    (
        "lsp_diagnostics",
        "Read language-server diagnostics for one source file. Use this after changing code or when compiler-like errors need precise locations.",
        json!({"type":"object","required":["path"],"properties":path_properties(),"additionalProperties":false}),
    )
}

pub(super) fn definition() -> (&'static str, &'static str, Value) {
    (
        "lsp_definition",
        "Find the definition of the symbol at a one-based source position.",
        json!({"type":"object","required":["path","line","column"],"properties":position_properties(),"additionalProperties":false}),
    )
}

pub(super) fn references() -> (&'static str, &'static str, Value) {
    (
        "lsp_references",
        "Find bounded references to the symbol at a one-based source position.",
        json!({"type":"object","required":["path","line","column"],"properties":position_properties(),"additionalProperties":false}),
    )
}

pub(super) fn hover() -> (&'static str, &'static str, Value) {
    (
        "lsp_hover",
        "Read type and documentation information for the symbol at a one-based source position.",
        json!({"type":"object","required":["path","line","column"],"properties":position_properties(),"additionalProperties":false}),
    )
}

pub(super) fn symbols() -> (&'static str, &'static str, Value) {
    (
        "lsp_symbols",
        "List bounded language-server symbols declared in one source file.",
        json!({"type":"object","required":["path"],"properties":path_properties(),"additionalProperties":false}),
    )
}
