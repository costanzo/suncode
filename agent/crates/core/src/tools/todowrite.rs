use serde_json::{json, Value};

pub fn definition() -> (&'static str, &'static str, Value) {
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
                            "content": {"type": "string", "minLength": 1, "maxLength": 500, "description": "Brief actionable task description"},
                            "status": {"type": "string", "enum": ["pending", "in_progress", "completed", "cancelled"]},
                            "priority": {"type": "string", "enum": ["high", "medium", "low"]}
                        },
                        "additionalProperties": false
                    }
                }
            },
            "additionalProperties": false
        }),
    )
}
