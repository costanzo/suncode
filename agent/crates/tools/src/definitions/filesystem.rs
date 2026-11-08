use serde_json::{json, Value};

pub(super) fn read() -> (&'static str, &'static str, Value) {
    (
        "read",
        "Read a bounded text file. Use offset (1-indexed) and limit to continue through large files. Reading from the start without a line limit returns the full-file sha256 when the source fits within 16 MiB, even if the preview is truncated. Pass sha256 as expected_sha256 to overwrite or delete the file. Relative paths resolve inside the current project; registered read-only dependencies use dependency:<dependencyId>/<path>.",
        json!({
            "type": "object",
            "required": ["path"],
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File path relative to the project"
                },
                "offset": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "1-indexed line to start reading from"
                },
                "limit": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Maximum number of lines to return"
                },
                "max_bytes": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Maximum number of bytes to return before creating an artifact"
                }
            },
            "additionalProperties": false
        }),
    )
}

pub(super) fn glob() -> (&'static str, &'static str, Value) {
    (
        "glob",
        "List project files matching a glob pattern. Results respect ignore rules and are bounded to 100 entries. Set path to a project subdirectory or dependency:<dependencyId> for a registered read-only dependency.",
        json!({
            "type": "object",
            "required": ["pattern"],
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Glob pattern such as **/*.rs"
                },
                "path": {
                    "type": "string",
                    "description": "Directory to search, relative to the project"
                },
                "limit": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 1000,
                    "description": "Maximum number of paths to return"
                }
            },
            "additionalProperties": false
        }),
    )
}

pub(super) fn grep() -> (&'static str, &'static str, Value) {
    (
        "grep",
        "Fast content search for regular expressions. Use this tool whenever you need to find files or matching lines. Do not use bash to run grep or rg for normal file search; use bash with rg only when you explicitly need raw terminal behavior such as counting matches. Results are bounded to 100 matches. Set path to a project subdirectory or dependency:<dependencyId> for a registered read-only dependency.",
        json!({
            "type": "object",
            "required": ["pattern"],
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Regular expression to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Directory or file to search"
                },
                "include": {
                    "type": "string",
                    "description": "Optional file glob such as **/*.rs"
                },
                "limit": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 500,
                    "description": "Maximum number of matches to return"
                }
            },
            "additionalProperties": false
        }),
    )
}

pub(super) fn write() -> (&'static str, &'static str, Value) {
    (
        "write",
        "Write UTF-8 content to one project file after approval. When updating an existing file, pass the latest sha256 from read, write, or edit as expected_sha256. Omit expected_sha256 for a new file. Returns the written file's sha256.",
        json!({
            "type": "object",
            "required": ["path", "content"],
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File path relative to the project"
                },
                "content": {
                    "type": "string",
                    "description": "Complete file content"
                },
                "expected_sha256": {
                    "type": ["string", "null"],
                    "pattern": "^[0-9a-fA-F]{64}$",
                    "description": "Full-file sha256 from the latest read, write, or edit. Omit for a new file."
                }
            },
            "additionalProperties": false
        }),
    )
}

pub(super) fn edit() -> (&'static str, &'static str, Value) {
    (
        "edit",
        "Replace exact text in one project file after approval. Include enough surrounding text to make each match unique. Set replaceAll only when every occurrence should change. For multiple independent edits, provide edits with oldText and newText entries, each matching once in the original file. Returns the edited file's sha256.",
        json!({
            "type": "object",
            "required": ["path"],
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File path relative to the project"
                },
                "oldString": {
                    "type": "string",
                    "description": "Exact text to replace"
                },
                "newString": {
                    "type": "string",
                    "description": "Replacement text"
                },
                "edits": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["oldText", "newText"],
                        "properties": {
                            "oldText": {
                                "type": "string"
                            },
                            "newText": {
                                "type": "string"
                            }
                        },
                        "additionalProperties": false
                    }
                },
                "replaceAll": {
                    "type": "boolean"
                }
            },
            "additionalProperties": false
        }),
    )
}

pub(super) fn delete() -> (&'static str, &'static str, Value) {
    (
        "delete",
        "Delete one project file after approval. Pass the latest sha256 from read, write, or edit as expected_sha256. Use this tool instead of bash rm so the deletion appears in the turn's changes. Directories cannot be deleted.",
        json!({
            "type": "object",
            "required": ["path", "expected_sha256"],
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File path relative to the project"
                },
                "expected_sha256": {
                    "type": "string",
                    "pattern": "^[0-9a-fA-F]{64}$",
                    "description": "Full-file sha256 from the latest read, write, or edit"
                }
            },
            "additionalProperties": false
        }),
    )
}
