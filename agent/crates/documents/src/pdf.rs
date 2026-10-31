///! PDF text extraction, one unit per page.

use crate::model::{Format, ParsedDocument, Unit, UnitKind, Warning};
use crate::text::clean_page;
use crate::DocumentError;

pub(crate) fn parse(bytes: &[u8]) -> Result<ParsedDocument, DocumentError> {
    let pages = pdf_extract::extract_text_from_mem_by_pages(bytes).map_err(|error| {
        if is_encrypted(bytes) {
            DocumentError::new(
                "encrypted_document",
                "the document is encrypted or password-protected",
            )
        } else {
            DocumentError::parse(format!("the PDF could not be read: {error}"))
        }
    })?;
    let units = pages
        .iter()
        .map(|page| Unit::new(UnitKind::Page, clean_page(page)))
        .collect::<Vec<_>>();
    let empty = units.iter().filter(|unit| unit.text.is_empty()).count();
    let mut warnings = Vec::new();
    if empty > 0 {
        warnings.push(Warning {
            code: "no_text_layer",
            message: format!(
                "{empty} of {} pages have no text layer: OCR is not available",
                units.len()
            ),
        });
    }
    Ok(ParsedDocument {
        format: Format::Pdf,
        units,
        warnings,
    })
}

fn is_encrypted(bytes: &[u8]) -> bool {
    bytes
        .windows(b"/Encrypt".len())
        .any(|window| window == b"/Encrypt")
}
