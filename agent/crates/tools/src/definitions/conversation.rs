use serde_json::{json, Value};

pub(super) fn question() -> (&'static str, &'static str, Value) {
    (
        "question",
        "Ask the user one or more clarifying questions and wait for their answers before continuing. Use concise option labels and include a custom answer when the user may need to provide text not covered by the options.",
        json!({
            "type": "object",
            "required": ["questions"],
            "properties": {
                "questions": {
                    "type": "array",
                    "minItems": 1,
                    "maxItems": 8,
                    "items": {
                        "type": "object",
                        "required": ["question", "header", "options"],
                        "properties": {
                            "question": {
                                "type": "string",
                                "minLength": 1
                            },
                            "header": {
                                "type": "string",
                                "minLength": 1,
                                "maxLength": 30
                            },
                            "options": {
                                "type": "array",
                                "minItems": 1,
                                "maxItems": 12,
                                "items": {
                                    "type": "object",
                                    "required": ["label", "description"],
                                    "properties": {
                                        "label": {
                                            "type": "string",
                                            "minLength": 1,
                                            "maxLength": 120
                                        },
                                        "description": {
                                            "type": "string",
                                            "minLength": 1,
                                            "maxLength": 500
                                        }
                                    },
                                    "additionalProperties": false
                                }
                            },
                            "multiple": {
                                "type": "boolean"
                            },
                            "custom": {
                                "type": "boolean"
                            }
                        },
                        "additionalProperties": false
                    }
                }
            },
            "additionalProperties": false
        }),
    )
}

pub(super) fn todowrite() -> (&'static str, &'static str, Value) {
    (
        "todowrite",
        "Create and maintain a structured task list for the current turn. Use it to track multi-step work and keep todo statuses current.",
        json!({
            "type": "object",
            "required": ["todos"],
            "properties": {
                "todos": {
                    "type": "array",
                    "maxItems": 100,
                    "description": "The complete replacement todo list for the current turn",
                    "items": {
                        "type": "object",
                        "required": ["content", "status", "priority"],
                        "properties": {
                            "content": {
                                "type": "string",
                                "minLength": 1,
                                "maxLength": 500,
                                "description": "Brief actionable task description"
                            },
                            "status": {
                                "type": "string",
                                "enum": ["pending", "in_progress", "completed", "cancelled"]
                            },
                            "priority": {
                                "type": "string",
                                "enum": ["high", "medium", "low"]
                            }
                        },
                        "additionalProperties": false
                    }
                }
            },
            "additionalProperties": false
        }),
    )
}
