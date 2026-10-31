//! Read-only parsing of PDF, Word, PowerPoint, and spreadsheet files into a normalized
//! model, with bounded range reading, plain-text search, spreadsheet ranges,
//! and an in-memory parse cache shared across ccalls.
//!
//! - `detect`: format detection from file content.
//! - `pdf`, `office`, `xlsx`: parser adapters for the supported formats.
//! - `ooxml`: bounded package part and relationship reading for PPTX.
//! - `query`: inspect, read, and search over the normzalized model.
//! - `text`: text cleanup and search folding.
//!
mod cache;
mod detect;
mod model;
mod office;
mod ooxml;
mod pdf;
mod query;
mod text;
mod xlsx;

#[cfg(test)]
mod tests;

pub use model::Format;

use model::Parsed;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fmt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Files larger than this are rejected before parsing
pub const MAX_DOCUMENT_BYTES: u64 = 64 * 1024 * 1024;
/// Part of every fingerprint; bump it when parser output changes
const PARSER_VERSION: u64 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentError {
    pub code: &'static str,
    pub message: String,
}

impl DocumentError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub(crate) fn invalid(message: impl Into<String>) -> Self {
        Self::new("invalid_arguments", message)
    }

    pub(crate) fn parse(message: impl Into<String>) -> Self {
        Self::new("parse_failed", message)
    }
}

impl fmt::Display for DocumentError {
    fn fmt(&self, formater: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formater, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for DocumentError {}

pub struct ReadRequest {
    pub unit: String,
    pub start: u32,
    pub end: u32,
    pub offset: usize,
    pub max_chars: Option<usize>,
}

pub struct SearchRequest {
    pub query: String,
    pub offset: usize,
    pub max_results: Option<usize>,
}

pub struct SpreadsheetRequest {
    pub sheet: String,
    pub range: String,
    pub include_formulas: bool,
}

#[derive(Default)]
pub struct Documents {
    cache: cache::Cache,
}

struct Loaded {
    parsed: Arc<Parsed>,
    fingerprint: String,
    bytes: Vec<u8>,
}

impl Documents {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn inspect(
        &self,
        path: &Path,
        cancellation: Option<&AtomicBool>,
    ) -> Result<Value, DocumentError> {
        let loaded = self.load(path, cancellation)?;
        Ok(query::inspect(
            &loaded.parsed,
            &loaded.fingerprint,
            loaded.bytes.len(),
        ))
    }

    pub fn read(
        &self,
        path: &Path,
        request: &ReadRequest,
        cancellation: Option<&AtomicBool>,
    ) -> Result<Value, DocumentError> {
        let loaded = self.load(path, cancellation)?;
        match loaded.parsed.as_ref() {
            Parsed::Document(document) => query::read(document, &loaded.fingerprint, request),
            Parsed::Workbook(workbook) => Err(DocumentError::invalid(format!(
                "document_read does not support {} files; use spreadsheet_read",
                workbook.format.name()
            ))),
        }
    }

    pub fn search(
        &self,
        path: &Path,
        request: &SearchRequest,
        cancellation: Option<&AtomicBool>,
    ) -> Result<Value, DocumentError> {
        let loaded = self.load(path, cancellation)?;
        query::search(&loaded.parsed, &loaded.fingerprint, request)
    }

    pub fn spreadsheet(
        &self,
        path: &Path,
        request: &SpreadsheetRequest,
        cancellation: Option<&AtomicBool>,
    ) -> Result<Value, DocumentError> {
        let loaded = self.load(path, cancellation)?;
        match loaded.parsed.as_ref() {
            Parsed::Workbook(workbook) => {
                xlsx::read_range(workbook, &loaded.bytes, &loaded.fingerprint, request)
            }
            Parsed::Document(document) => Err(DocumentError::invalid(format!(
                "spreadsheet_read supports only spreadsheet files; this is a {} file",
                document.format.name()
            ))),
        }
    }

    fn load(
        &self,
        path: &Path,
        cancellation: Option<&AtomicBool>,
    ) -> Result<Loaded, DocumentError> {
        let too_large = || {
            DocumentError::new(
                "file_too_large",
                format!("file exceeds maximum size of {} bytes", MAX_DOCUMENT_BYTES),
            )
        };
        let metadata = std::fs::metadata(path)
            .map_err(|_| DocumentError::new("path_unavailable", "path is not available"))?;
        if metadata.len() > MAX_DOCUMENT_BYTES {
            return Err(too_large());
        }
        let bytes = std::fs::read(path)
            .map_err(|_| DocumentError::new("read_failed", "file could not be read"))?;
        if bytes.len() as u64 > MAX_DOCUMENT_BYTES {
            return Err(too_large());
        }
        let fingerprint = fingerprint(&bytes);
        if let Some(parsed) = self.cache.get(path, &fingerprint) {
            return Ok(Loaded {
                parsed,
                fingerprint,
                bytes,
            });
        }
        check_cancelled(cancellation)?;
        let format = detect::detect(&bytes)?;
        let parsed =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| parse(format, &bytes)))
                .unwrap_or_else(|_| Err(DocumentError::parse("the parser stopped unexpectedly")))?;
        check_cancelled(cancellation)?;
        let parsed = Arc::new(parsed);
        self.cache
            .insert(path, &fingerprint, parsed.clone(), bytes.len());
        Ok(Loaded {
            parsed,
            fingerprint,
            bytes,
        })
    }
}

fn parse(format: Format, bytes: &[u8]) -> Result<Parsed, DocumentError> {
    Ok(match format {
        Format::Pdf => Parsed::Document(pdf::parse(bytes)?),
        Format::Doc | Format::Docx | Format::Ppt | Format::Pptx => {
            Parsed::Document(office::parse(bytes, format)?)
        }
        Format::Xls | Format::Xlsx | Format::Ods => Parsed::Workbook(xlsx::parse(bytes, format)?),
    })
}

fn fingerprint(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let hex = digest
        .iter()
        .take(16)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("{hex}-p{PARSER_VERSION}")
}

fn check_cancelled(cancellation: Option<&AtomicBool>) -> Result<(), DocumentError> {
    if cancellation.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
        return Err(DocumentError::new(
            "cancelled",
            "document parsing was cancelled",
        ));
    }
    Ok(())
}
