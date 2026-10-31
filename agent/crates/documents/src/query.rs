//! Inspect, range reading, and search over the normalized model.

use crate::model::{Format, Parsed, ParsedDocument, ParsedWorkbook, Unit, UnitKind};
use crate::text::{char_slice, collapse_whitespace, fold_query, Folded};
use crate::xlsx::{cell_reference, display_value};
use crate::{DocumentError, ReadRequest, SearchRequest};
use serde_json::{json, Map, Value};

/// Entries listed by number in a summary before it is marked truncated
const MAX_LISTED: usize = 50;
const DEFAULT_MAX_CHARS: usize = 12_000;
const MAX_CHARS: usize = 50_000;
const DEFAULT_MAX_RESULTS: usize = 20;
const MAX_RESULTS: usize = 100;
/// Characters of context on each side of a match
const CONTEXT_CHARS: usize = 80;
/// How far before a match the suggested read starts.
const READ_LEAD_CHARS: usize = 200;

pub(crate) fn inspect(parsed: &Parsed, fingerprint: &str, bytes: usize) -> Value {
    let (format, warnings, summary) = match parsed {
        Parsed::Document(document) => (
            document.format,
            document
                .warnings
                .iter()
                .map(|warning| json!({"code": warning.code, "message": warning.message}))
                .collect(),
            document_summary(document),
        ),
        Parsed::Workbook(workbook) => (workbook.format, Vec::new(), workbook_summary(workbook)),
    };
    json!({
        "format": format.name(),
        "bytes": bytes,
        "fingerprint": fingerprint,
        "warnings": warnings,
        "summary": summary,
    })
}

fn document_summary(document: &ParsedDocument) -> Value {
    let units = &document.units;
    match document.format {
        Format::Pdf => {
            let empty = numbers(units, |unit| unit.text.is_empty());
            let text_layer = if empty.is_empty() {
                "full"
            } else if empty.len() == units.len() {
                "none"
            } else {
                "partial"
            };
            json!({
                "page_count": units.len(),
                "text_layer": text_layer,
                "pages_without_text": capped(&empty),
                "pages_without_text_truncated": empty.len() > MAX_LISTED,
            })
        }
        Format::Ppt | Format::Pptx => {
            let hidden = numbers(units, |unit| unit.hidden);
            json!({
                "slide_count": units.len(),
                "hidden_slides": capped(&hidden),
                "hidden_slides_truncated": hidden.len() > MAX_LISTED,
                "has_speaker_notes": units.iter().any(|unit| unit.has_notes),
            })
        }
        Format::Doc | Format::Docx => {
            let mut kinds = Map::new();
            for name in ["heading", "paragraph", "list_item", "table"] {
                let count = units
                    .iter()
                    .filter(|unit| unit.kind.block_name() == Some(name))
                    .count();
                kinds.insert(name.into(), json!(count));
            }
            json!({"block_count": units.len(), "blocks_by_kind": kinds})
        }
        Format::Xls | Format::Xlsx | Format::Ods => unreachable!("workbooks use workbook_summary"),
    }
}

fn workbook_summary(workbook: &ParsedWorkbook) -> Value {
    let sheets = workbook
        .sheets
        .iter()
        .take(MAX_LISTED)
        .enumerate()
        .map(|(index, sheet)| {
            let used_range = match (sheet.cells.start(), sheet.cells.end()) {
                (Some(start), Some(end)) if !sheet.cells.is_empty() => Some(format!(
                    "{}:{}",
                    cell_reference(start.0, start.1),
                    cell_reference(end.0, end.1)
                )),
                _ => None,
            };
            json!({
                "name": sheet.name,
                "index": index + 1,
                "visibility": sheet.visibility,
                "used_range": used_range,
                "rows": sheet.cells.height(),
                "columns": sheet.cells.width(),
            })
        })
        .collect::<Vec<_>>();
    json!({
        "sheet_count": workbook.sheets.len(),
        "sheets": sheets,
        "sheets_truncated": workbook.sheets.len() > MAX_LISTED,
    })
}

/// 1-based numbers of units matching `predicate`.
fn numbers(units: &[Unit], predicate: impl Fn(&Unit) -> bool) -> Vec<usize> {
    units
        .iter()
        .enumerate()
        .filter(|(_, unit)| predicate(unit))
        .map(|(index, _)| index + 1)
        .collect()
}

fn capped(values: &[usize]) -> &[usize] {
    &values[..values.len().min(MAX_LISTED)]
}

/// Reads units `start..=end`. `offset` and `max_chars` count characters
/// across the units' texts in order; each segment carries its unit index
/// and its offset within that unit, so successive calls rebuild every unit
/// exactly. Empty units are reported once, as empty segments.
pub(crate) fn read(
    document: &ParsedDocument,
    fingerprint: &str,
    request: &ReadRequest,
) -> Result<Value, DocumentError> {
    let unit_name = document.format.unit();
    if request.unit != unit_name {
        return Err(DocumentError::invalid(format!(
            "unit for {} files is {unit_name}",
            document.format.name()
        )));
    }
    let total = document.units.len();
    if total == 0 {
        return Err(DocumentError::invalid(format!(
            "the document has no {unit_name}s to read"
        )));
    }
    let (start, end) = (request.start as usize, request.end as usize);
    if start == 0 || start > end || end > total {
        return Err(DocumentError::invalid(format!(
            "start and end must satisfy 1 <= start <= end <= {total}"
        )));
    }
    let max_chars = request.max_chars.unwrap_or(DEFAULT_MAX_CHARS);
    if max_chars == 0 || max_chars > MAX_CHARS {
        return Err(DocumentError::invalid(format!(
            "max_chars must be between 1 and {MAX_CHARS}"
        )));
    }
    let selected = &document.units[start - 1..end];
    let lengths = selected
        .iter()
        .map(|unit| unit.text.chars().count())
        .collect::<Vec<_>>();
    let total_chars = lengths.iter().sum::<usize>();
    if request.offset > total_chars {
        return Err(DocumentError::invalid(format!(
            "offset must be at most {total_chars} for this range"
        )));
    }
    let window_end = (request.offset + max_chars).min(total_chars);
    let mut segments = Vec::new();
    let mut position = 0;
    for (index, (unit, length)) in selected.iter().zip(&lengths).enumerate() {
        let (unit_start, unit_end) = (position, position + length);
        position = unit_end;
        let included = if *length == 0 {
            // At a window boundary, an empty unit belongs to the next call.
            request.offset <= unit_start && (unit_start < window_end || window_end == total_chars)
        } else {
            unit_start < window_end && unit_end > request.offset
        };
        if !included {
            continue;
        }
        let from = request.offset.saturating_sub(unit_start);
        let to = window_end.min(unit_end) - unit_start;
        let mut segment = json!({
            "unit": unit_name,
            "index": start + index,
            "offset": from,
            "text": char_slice(&unit.text, from, to),
        });
        describe(&mut segment, unit);
        segments.push(segment);
    }
    let truncated = window_end < total_chars;
    Ok(json!({
        "format": document.format.name(),
        "fingerprint": fingerprint,
        "unit": unit_name,
        "start": start,
        "end": end,
        "offset": request.offset,
        "total_chars": total_chars,
        "segments": segments,
        "truncated": truncated,
        "next_offset": truncated.then_some(window_end),
    }))
}

fn describe(segment: &mut Value, unit: &Unit) {
    match unit.kind {
        UnitKind::Page if unit.text.is_empty() => segment["no_text_layer"] = json!(true),
        UnitKind::Slide if unit.hidden => segment["hidden"] = json!(true),
        UnitKind::Heading(level) => {
            segment["kind"] = json!("heading");
            segment["level"] = json!(level);
        }
        kind => {
            if let Some(name) = kind.block_name() {
                segment["kind"] = json!(name);
            }
        }
    }
}

/// Collects matches after skipping `offset` and stops one past
/// `max_results` to know whether more exist.
struct Matches {
    skip: usize,
    limit: usize,
    found: Vec<Value>,
    more: bool,
}

impl Matches {
    fn full(&self) -> bool {
        !self.more
    }

    fn push(&mut self, matched: impl FnOnce() -> Value) {
        if self.skip > 0 {
            self.skip -= 1;
        } else if self.found.len() < self.limit {
            self.found.push(matched());
        } else {
            self.more = true;
        }
    }
}

pub(crate) fn search(
    parsed: &Parsed,
    fingerprint: &str,
    request: &SearchRequest,
) -> Result<Value, DocumentError> {
    let query = fold_query(&request.query);
    if query.is_empty() {
        return Err(DocumentError::invalid("query must not be empty"));
    }
    let limit = request.max_results.unwrap_or(DEFAULT_MAX_RESULTS);
    if limit == 0 || limit > MAX_RESULTS {
        return Err(DocumentError::invalid(format!(
            "max_results must be between 1 and {MAX_RESULTS}"
        )));
    }
    let mut matches = Matches {
        skip: request.offset,
        limit,
        found: Vec::new(),
        more: false,
    };
    let format = match parsed {
        Parsed::Document(document) => {
            search_document(document, &query, &mut matches);
            document.format
        }
        Parsed::Workbook(workbook) => {
            search_workbook(workbook, &query, &mut matches);
            workbook.format
        }
    };
    let returned = matches.found.len();
    Ok(json!({
        "format": format.name(),
        "fingerprint": fingerprint,
        "query": request.query,
        "matches": matches.found,
        "truncated": matches.more,
        "next_offset": matches.more.then_some(request.offset + returned),
    }))
}

fn search_document(document: &ParsedDocument, query: &str, matches: &mut Matches) {
    let unit_name = document.format.unit();
    for (position, unit) in document.units.iter().enumerate() {
        let index = position + 1;
        let folded = Folded::new(&unit.text);
        for (byte, _) in folded.text.match_indices(query) {
            if matches.full() {
                return;
            }
            matches.push(|| {
                let (start, end) = folded.source_range(byte, byte + query.len());
                let mut location = Map::new();
                location.insert(unit_name.into(), json!(index));
                json!({
                    "location": location,
                    "match": char_slice(&unit.text, start, end),
                    "context": context(&unit.text, start, end),
                    "char_offset": start,
                    "read": {
                        "tool": "document_read",
                        "unit": unit_name,
                        "start": index,
                        "end": index,
                        "offset": start.saturating_sub(READ_LEAD_CHARS),
                    }
                })
            })
        }
    }
}

fn search_workbook(workbook: &ParsedWorkbook, query: &str, matches: &mut Matches) {
    for sheet in &workbook.sheets {
        let Some((first_row, first_column)) = sheet.cells.start() else {
            continue;
        };
        for (row, column, value) in sheet.cells.used_cells() {
            let Some(text) = display_value(value) else {
                continue;
            };
            let folded = Folded::new(&text);
            for (byte, _) in folded.text.match_indices(query) {
                if matches.full() {
                    return;
                }
                let reference =
                    cell_reference(first_row + row as u32, first_column + column as u32);
                matches.push(|| {
                    let (start, end) = folded.source_range(byte, byte + query.len());
                    json!({
                        "location": {"sheet": sheet.name, "cell": reference},
                        "match": char_slice(&text, start, end),
                        "context": context(&text, start, end),
                        "char_offset": start,
                        "read": {
                            "tool": "spreadsheet_read",
                            "sheet": sheet.name,
                            "range": reference,
                        }
                    })
                })
            }
        }
    }
}

fn context(text: &str, start: usize, end: usize) -> String {
    let length = text.chars().count();
    let from = start.saturating_sub(CONTEXT_CHARS);
    let to = (end + CONTEXT_CHARS).min(length);
    let mut context = collapse_whitespace(char_slice(text, from, to));
    if from > 0 {
        context.insert(0, '…');
    }
    if to < length {
        context.push('…');
    }
    context
}
