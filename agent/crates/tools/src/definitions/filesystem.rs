use serde_json::{json, Value};

pub(super) fn read() -> (&'static str, &'static str, Value) {
    (
        "read",
        "Read a bounded text file, including BOM-marked UTF-16LE up to 16 MiB. Use offset (1-indexed) and limit to continue through large UTF-8 files. Reading from the start without a line limit returns the raw full-file sha256 when the source fits within 16 MiB, even if the preview is truncated. Pass sha256 as expected_sha256 to overwrite or delete the file. source_sha256 also accompanies bounded partial reads for change detection; it does not mean all content was shown. Relative paths resolve inside the main project. Use real absolute paths for files in registered dependencies; legacy dependency:<dependencyId>/<path> aliases are also accepted.",
        json!({
            "type": "object",
            "required": ["path"],
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File path relative to the main project, or an absolute path inside the main project or a registered dependency"
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
        "List project files matching a glob pattern. Results respect ignore rules and are bounded to 100 entries. Set path to a project subdirectory or the real absolute path of a registered dependency directory. Legacy dependency aliases are accepted.",
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
                    "description": "Directory relative to the main project, or an absolute directory inside a registered dependency"
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
        "Fast content search for regular expressions. Use this tool whenever you need to find files or matching lines. Do not use bash to run grep or rg for normal file search; use bash with rg only when you explicitly need raw terminal behavior such as counting matches. Results are bounded to 100 matches. Set path to a project subdirectory or the real absolute path of a registered dependency directory. Legacy dependency aliases are accepted.",
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
                    "description": "File path relative to the main project, or an absolute path inside the main project or a registered dependency"
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
        "Replace exact text in one project file after approval. Inspect relevant content and repository instructions first. Use either oldString/newString with optional replaceAll, or edits containing oldString/newString objects (never both). Each batch entry matches once in the original file, not earlier edits' output. Include context to make matches unique. LF/CRLF/CR match equivalently; untouched bytes and UTF-8 or BOM-marked UTF-16LE encoding are preserved. Source and result are limited to 16 MiB. Returns sha256, replacement counts, changed ranges, and a bounded before/after preview. Conflict details identify zero-based edit indexes and candidate lines; re-read and correct the call rather than retrying unchanged.",
        json!({
            "type": "object",
            "required": ["path"],
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File path relative to the main project, or an absolute path inside the main project or a registered dependency"
                },
                "oldString": {
                    "type": "string",
                    "minLength": 1,
                    "description": "Exact text to replace"
                },
                "newString": {
                    "type": "string",
                    "description": "Replacement text"
                },
                "edits": {
                    "type": "array",
                    "minItems": 1,
                    "maxItems": 200,
                    "items": {
                        "type": "object",
                        "required": ["oldString", "newString"],
                        "properties": {
                            "oldString": {
                                "type": "string",
                                "minLength": 1
                            },
                            "newString": {
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
                    "description": "File path relative to the main project, or an absolute path inside the main project or a registered dependency"
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
