//! The read-only document tools over `suncode_documents`.

use crate::arguments::{
    DocumentReadArguments, DocumentInspectArguments, DocumentSearchArguments,
    SpreadsheetReadArguments,
};
use crate::error::{fatal, retryable};
use crate::scope::{existing_file_path, require_project};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use suncode_common::BusinessError;
use suncode_documents::{DocumentError, Documents, ReadRequest, SearchRequest, SpreadsheetRequest};

pub(crate) fn inspect(
    project_root: Option<&Path>,
    documents: &Documents,
    args: &DocumentInspectArguments,
    cancellation: Option<&AtomicBool>,
) -> Result<Value, BusinessError> {
    let path = document_path(project_root, &args.path)?;
    with_path(documents.inspect(&path, cancellation), &args.path)
}

pub(crate) fn read(
    project_root: Option<&Path>,
    documents: &Documents,
    args: &DocumentReadArguments,
    cancellation: Option<&AtomicBool>,
) -> Result<Value, BusinessError> {
    let path = document_path(project_root, &args.path)?;
    let request = ReadRequest {
        unit: args.unit.clone(),
        start: args.start,
        end: args.end,
        offset: args.offset.unwrap_or(0),
        max_chars: args.max_chars,
    };
    with_path(documents.read(&path, &request, cancellation), &args.path)
}

pub(crate) fn search(
    project_root: Option<&Path>,
    documents: &Documents,
    args: &DocumentSearchArguments,
    cancellation: Option<&AtomicBool>,
) -> Result<Value, BusinessError> {
    let path = document_path(project_root, &args.path)?;
    let request = SearchRequest {
        query: args.query.clone(),
        offset: args.offset.unwrap_or(0),
        max_results: args.max_results,
    };
    with_path(documents.search(&path, &request, cancellation), &args.path)
}

pub(crate) fn spreadsheet(
    project_root: Option<&Path>,
    documents: &Documents,
    args: &SpreadsheetReadArguments,
    cancellation: Option<&AtomicBool>,
) -> Result<Value, BusinessError> {
    let path = document_path(project_root, &args.path)?;
    let request = SpreadsheetRequest {
        sheet: args.sheet.clone(),
        range: args.range.clone(),
        include_formulas: args.include_formulas.unwrap_or(false),
    };
    with_path(
        documents.spreadsheet(&path, &request, cancellation),
        &args.path
    )
}

fn document_path(project_root: Option<&Path>, path: &str) -> Result<PathBuf, BusinessError> {
    existing_file_path(require_project(project_root)?, path)
}

fn with_path(result: Result<Value, DocumentError>, path: &str) -> Result<Value, BusinessError> {
    let mut value = result.map_err(|error| match error.code {
        "read_failed" => retryable(error.code, &error.message),
        code => fatal(code, error.message),
    })?;
    value["path"] = json!(path);
    Ok(value)
}
