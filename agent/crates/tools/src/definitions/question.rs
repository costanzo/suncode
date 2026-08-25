use serde_json::{json, Value};

pub fn definition() -> (&'static str, &'static str, Value) {
    (
        "question",
        "Ask the user one or more clarifying questions and wait for their answers before continuing. Use concise option labels and include a custom answer when the user may need to provide text not covered by the options.",
        json!({
            "type": "object",
            "required": ["questions"],
            "properties": {
                "questions": {
                    "type": "array", "minItems": 1, "maxItems": 8,
                    "items": {
                        "type": "object",
                        "required": ["question", "header", "options"],
                        "properties": {
                            "question": {"type": "string", "minLength": 1},
                            "header": {"type": "string", "minLength": 1, "maxLength": 30},
                            "options": {"type": "array", "minItems": 1, "maxItems": 12, "items": {
                                "type": "object", "required": ["label", "description"],
                                "properties": {
                                    "label": {"type": "string", "minLength": 1, "maxLength": 120},
                                    "description": {"type": "string", "minLength": 1, "maxLength": 500}
                                }, "additionalProperties": false
                            }},
                            "multiple": {"type": "boolean"}, "custom": {"type": "boolean"}
                        }, "additionalProperties": false
                    }
                }
            }, "additionalProperties": false
        }),
    )
}
