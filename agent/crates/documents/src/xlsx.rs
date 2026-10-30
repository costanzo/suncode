//! XLSX workbooks through calamine: cached cell values per worksheet,
//! typed cell output, and A1 range reading.

use crate::model::{ParsedWorkbook, Sheet};
use crate::{DocumentError, SpreadsheetRequest};
use calamine::{open_workbook_from_rs, Data, ExcelDateTime, Reader, SheetType, SheetVisible, Xlsx};
use serde_json::{json, Value};
use std::io::Cursor;

/// Largest readable rectangle, in cells
pub(crate) const MAX_RANGE_CELLS: u64 = 3000;
const MAX_ROWS: u32 = 1_048_576;
const MAX_COLUMNS: u32 = 16_384;
/// Cells listed by name in one warning.
const MAX_WARNING_CELLS: usize = 20;

pub(crate) fn parse(bytes: &[u8]) -> Result<ParsedWorkbook, DocumentError> {
    let mut workbook = open(bytes)?;
    let metadata = workbook.sheets_metadata().to_vec();
    let mut sheets = Vec::new();
    for sheet in metadata {
        if !matches!(sheet.typ, SheetType::WorkSheet) {
            continue;
        }
        let cells = workbook.worksheet_range(&sheet.name).map_err(|error| {
            DocumentError::parse(format!("sheet {} could not be read: {error}", sheet.name))
        })?;
        sheets.push(Sheet {
            name: sheet.name,
            visibility: match sheet.visible {
                SheetVisible::Visible => "visible",
                SheetVisible::Hidden => "hidden",
                SheetVisible::VeryHidden => "very_hidden",
            },
            cells,
        });
    }
    Ok(ParsedWorkbook { sheets })
}

fn open(bytes: &[u8]) -> Result<Xlsx<Cursor<&[u8]>>, DocumentError> {
    open_workbook_from_rs(Cursor::new(bytes))
        .map_err(|error| { DocumentError::parse(format!("workbook could not be opened: {error}")) })
}

pub(crate) fn read_range(
    workbook: &ParsedWorkbook,
    bytes: &[u8],
    fingerprint: &str,
    request: &SpreadsheetRequest,
) -> Result<Value, DocumentError> {
    let sheet = workbook
        .sheets
        .iter()
        .find(|sheet| sheet.name == request.sheet)
        .ok_or_else(|| {
            let names = workbook
                .sheets
                .iter()
                .take(20)
                .map(|sheet| sheet.name.as_str())
                .collect::<Vec<_>>();
            DocumentError::invalid(format!(
                "sheet {:?} was not found; sheets: {}",
                request.sheet,
                names.join(", ")
            ))
        })?;
    let ((first_row, first_column), (last_row, last_column)) = parse_range(&request.range)?;
    let area = u64::from(last_row - first_row + 1) * u64::from(last_column - first_column + 1);
    if area > MAX_RANGE_CELLS {
        return Err(DocumentError::invalid(format!(
            "range {} covers {area} cells; maximum is {MAX_RANGE_CELLS}",
            request.range
        )));
    }
    let formulas = if request.include_formulas {
        Some(
            open(bytes)?
                .worksheet_formula(&sheet.name)
                .map_err(|error| {
                    DocumentError::parse(format!("formulas could not be read: {error}"))
                })?
        )
    } else {
        None
    };
    let mut cells = Vec::new();
    let mut uncached = Vec::new();
    for row in first_row..=last_row {
        for column in first_column..=last_column {
            let value = sheet
                .cells
                .get_value((row, column))
                .filter(|value| !matches!(value, Data::Empty));
            let formula = formulas
                .as_ref()
                .and_then(|formulas| formulas.get_value((row, column)))
                .filter(|formula| !formula.is_empty());
            if value.is_none() && formula.is_none() {
                continue;
            }
            let reference = cell_reference(row, column);
            let mut cell = match value {
                Some(value) => typed_value(value),
                None => {
                    uncached.push(reference.clone());
                    json!({"type": "empty", "value": null})
                }
            };
            cell["cell"] = json!(reference);
            if let Some(formula) = formula {
                cell["formula"] = json!(format!("={formula}"));
            }
            cells.push(cell);
        }
    }
    let mut warnings = Vec::new();
    if !uncached.is_empty() {
        let listed = uncached
            .iter()
            .take(MAX_WARNING_CELLS)
            .cloned()
            .collect::<Vec<_>>();
        warnings.push(json!({
            "code": "formula_without_cached value",
            "message": format!(
                "{} formula cells have no cached value: {}{}",
                uncached.len(),
                listed.join(", "),
                if uncached.len() > MAX_WARNING_CELLS { ", ..." } else { "" }
            ),
        }));
    }
    Ok(json!({
        "format": "xlsx",
        "fingerprint": fingerprint,
        "sheet": sheet.name,
        "range": format!(
            "{}:{}",
            cell_reference(first_row, first_column),
            cell_reference(last_row, last_column)
        ),
        "cells": cells,
        "warnings": warnings,
    }))
}

/// `{"type", "value"}` for a non-empty cached value.
pub(crate) fn typed_value(value: &Data) -> Value {
    match value {
        Data::Int(number) => json!({"type": "number", "value": number}),
        Data::Float(number) => json!({"type": "number", "value": number}),
        Data::String(text) => json!({"type": "string", "value": text}),
        Data::Bool(flag) => json!({"type": "bool", "value": flag}),
        Data::DateTime(datetime) if datetime.is_duration() => json!({
            "type": "duration",
            "value": iso_duration(datetime.as_f64()),
            "serial": datetime.as_f64(),
        }),
        Data::DateTime(datetime) => json!({
            "type": "datetime",
            "value": ios_datetime(datetime),
            "serial": datetime.as_f64(),
        }),
        Data::DateTimeIso(text) => json!({"type": "datetime", "value": text}),
        Data::DurationIso(text) => json!({"type": "duration", "value": text}),
        Data::Error(error) => json!({"type": "error", "value": error.to_string()}),
        Data::Empty => json!({"type": "empty", "value": null}),
    }
}

/// The searchable text of a cached value; `None` for empty cells
pub(crate) fn display_value(value: &Data) -> Option<String> {
    Some(match value {
        Data::Int(number) => number.to_string(),
        Data::Float(number) => number.to_string(),
        Data::String(text) => text.clone(),
        Data::Bool(flag) => if *flag { "TRUE" } else { "FALSE" }.to_string(),
        Data::DateTime(datetime) if datetime.is_duration() => iso_duration(datetime.as_f64()),
        Data::DateTime(datetime) => ios_datetime(datetime),
        Data::DateTimeIso(text) | Data::DurationIso(text) => text.clone(),
        Data::Error(error) => error.to_string(),
        Data::Empty => return None,
    })
}

fn ios_datetime(datetime: &ExcelDateTime) -> String {
    let (year, month, day, hour, minute, second, millisecond) = datetime.to_ymd_hms_milli();
    let mut text = format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}");
    if millisecond > 0 {
        text.push_str(&format!(".{millisecond:03}"));
    }
    text
}

fn iso_duration(days: f64) -> String {
    let total = (days.abs() * 86_400_000.0).round() as u64;
    let (hours, rest) = (total / 3_600_000, total % 3_600_000);
    let (minutes, rest) = (rest / 60_000, rest % 60_000);
    let (seconds, milliseconds) = (rest / 1_000, rest % 1_000);
    let sign = if days < 0.0 { "-" } else { "" };
    let mut text = format!("{sign}PT{hours}H{minutes}M{seconds}");
    if milliseconds > 0 {
        text.push_str(&format!(".{milliseconds:03}"));
    }
    text.push('S');
    text
}

/// `A1` notation for zero-based `(row, column)`.
pub(crate) fn cell_reference(row: u32, column: u32) -> String {
    let mut letters = Vec::new();
    let mut remaining = column + 1;
    while remaining > 0 {
        let digit = (remaining - 1) % 26;
        letters.push(char::from(b'A' + digit as u8));
        remaining = (remaining - 1) / 26;
    }
    letters.iter().rev().collect::<String>() + &(row + 1).to_string()
}

/// Zero-based `(row, column)`
pub(crate) type CellPosition = (u32, u32);

/// Zero-based inclusive corners of `A1` or `A1:F50`, ordered top-left first.
pub(crate) fn parse_range(range: &str) -> Result<(CellPosition, CellPosition), DocumentError> {
    let invalid = || {
        DocumentError::invalid(format!(
            "range {range:?} must be a cell such as A1 or a rectangle such as A1:F50; whole rows and columns are not supported"
        ))
    };
    let (first, last) = match range.trim().split_once(':') {
        Some((first, last)) => (first, last),
        None => (range.trim(), range.trim()),
    };
    let first = parse_cell(first).ok_or_else(invalid)?;
    let last = parse_cell(last).ok_or_else(invalid)?;
    Ok((
        (first.0.min(last.0),first.1.min(last.1)),
        (first.0.max(last.0),first.1.max(last.1)),
    ))
}

fn parse_cell(cell: &str) -> Option<(u32, u32)> {
    let cell = cell.trim().replace('$', "");
    let split = cell.find(|c: char| c.is_ascii_digit())?;
    let (letters, digits) = cell.split_at(split);
    if letters.is_empty()
        || !letters
            .chars()
            .all(|c| c.is_ascii_alphabetic())
    {
        return None;
    }
    let mut column = 0_u32;
    for letter in letters.chars() {
        column = column
            .checked_mul(26)?
            .checked_add(u32::from(letter.to_ascii_uppercase() as u8 - b'A') + 1)?
    }
    let row = digits.parse::<u32>().ok()?;
    if row == 0 || row > MAX_ROWS || column > MAX_COLUMNS {
        return None;
    }
    Some((row - 1, column - 1))
}
