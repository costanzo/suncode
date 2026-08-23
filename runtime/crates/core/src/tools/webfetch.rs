use serde_json::{json, Value};

pub fn definition() -> (&'static str, &'static str, Value) {
    (
        "webfetch",
        "Fetch content from an HTTP or HTTPS URL after approval and return it as text, markdown, or HTML. Markdown is the default.",
        json!({
            "type":"object",
            "required":["url"],
            "properties":{
                "url":{"type":"string","description":"The HTTP or HTTPS URL to fetch content from"},
                "format":{
                    "type":"string",
                    "enum":["text","markdown","html"],
                    "default":"markdown",
                    "description":"The format to return the content in. Defaults to markdown."
                },
                "timeout":{
                    "type":"number",
                    "exclusiveMinimum":0,
                    "maximum":120,
                    "description":"Optional timeout in seconds (maximum 120)"
                }
            },
            "additionalProperties":false
        }),
    )
}
