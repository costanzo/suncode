use serde_json::{json, Value};

const PATH: &str = "File path relative to the project, a dependency:<dependencyId>/<path> alias, or an absolute path inside the project or a registered dependency";

pub fn inspect() -> (&'static str, &'static str, Value) {
    (
        "document_inspect",
        "Summarize a PDF, Word (.docx), PowerPoint (.pptx), or Excel (.xlsx) file before reading it: format, fingerprint, warnings, and available locators (page count and text layer, slide count and hidden slides, block counts, or sheets with used ranges). Use document_read, document_search, or spreadsheet_read next.",
        json!({"type": "object", "required": ["path"], "properties": {"path": {"type": "string", "description": PATH}}, "additionalProperties": false}),
    )
}

pub fn read() -> (&'static str, &'static str, Value) {
    (
        "document_read",
        "Read text from a PDF (unit page), PowerPoint (unit slide, including speaker notes), or Word file (unit block: each heading, paragraph, list item, or table). Numbers start at 1 and ranges are inclusive. offset and max_chars count characters across the units in order; each segment gives its unit index and its offset within that unit. When truncated is true, call again with offset set to next_offset. For Excel files use spreadsheet_read.",
        json!(
            {
                "type": "object",
                "required": ["path", "unit", "start", "end"],
                "properties":
                    {
                        "path": {"type": "string", "description": PATH},
                        "unit": {"type": "string", "enum": ["page", "slide", "block"],"description": "page for PDF, slide for PowerPoint, block for Word"},
                        "start": {"type": "integer", "minimum": 1, "description": "the first unit to read (inclusive)"},
                        "end": {"type": "integer", "minimum": 1, "description": "the last unit to read (inclusive)"},
                        "offset": {"type": "integer", "minimum": 0, "description": "Characters to skip across the range; use next_offset from the previous call"},
                        "max_chars": {"type": "integer", "minimum": 1, "maximum": 50000, "description": "Maximum characters to return (default 12000)"}
                    },
                "additionalProperties": false
            }
        )
    )
}

pub fn search() -> (&'static str, &'static str, Value) {
    (
        "document_search",
        "Find plain text in a PDF, Word, PowerPoint, or Excel file. Matching ignores case and treats any whitespace run as one space; matches do not span pages, slides, blocks, or cells. Excel search covers cell values, not formulas. Each match gives its location and a read hint for document_read or spreadsheet_read.",
        json!(
            {
                "type": "object",
                "required": ["path", "query"],
                "properties":
                    {
                        "path": {"type": "string", "description": PATH},
                        "query": {"type": "string", "description": "Text to find"},
                        "offset": {"type": "integer", "minimum": 0, "description": "Matches to skip; use next_offset from the previous call"},
                        "max_results": {"type": "integer", "minimum": 1, "maximum": 100, "description": "Maximum matches to return (default 20)"}
                    },
                "additionalProperties": false
            }
        )
    )
}

pub fn spreadsheet() -> (&'static str, &'static str, Value) {
    (
        "spreadsheet_read",
        "Read cells from one sheet of an Excel (.xlsx) file. range is a cell (A1) or a rectangle (A1:F50) of at most 3000 cells, whole rows or columns are not accepted. Returns non-empty cells with their cached values typed as string, number, boolean, datetime (ISO 8601 plus serial), duration, or error. Formulas are not recalculated; set include_formulas to also return formula text.",
        json!(
            {
                "type": "object",
                "required": ["path", "sheet", "range"],
                "properties":
                    {
                        "path": {"type": "string", "description": PATH},
                        "sheet": {"type": "string", "description": "Sheet name as listed by document_inspect"},
                        "range": {"type": "string", "description": "A1 or A1:F50; $ is allowed"},
                        "include_formulas": {"type": "boolean", "description": "Also return formula text (default false)"}
                    },
                "additionalProperties": false
            }
        )
    )
}