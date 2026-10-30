//! Format detection from file content

use crate::model::Format;
use crate::ooxml::Package;
use crate::DocumentError;

const PDF_SIGNATURE: &[u8] = b"%PDF-";
const ZIP_SIGNATURE: &[u8] = b"PK\x03\x04";

const COMPOUND_FILE_SIGNATURE: &[u8] = &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

pub(crate) fn detect(bytes: &[u8]) -> Result<Format, DocumentError> {
    // Some producers write a few bytes before the PDF header.
    if bytes.len() >= PDF_SIGNATURE.len()
        && bytes[.. bytes.len().min(1024)]
            .windows(PDF_SIGNATURE.len())
            .any(|window| window == PDF_SIGNATURE)
    {
        return Ok(Format::Pdf);
    }
    if bytes.starts_with(COMPOUND_FILE_SIGNATURE) {
        if contains_utf16(bytes, "EncryptionInfo") {
            return Err(DocumentError::new(
                "encrypted_document",
                "the document is encrypted or password-protected",
            ));
        }
        return Err(unsupported(
            "legacy binary Office formats (.doc, .xls, .ppt) are not supported",
        ));
    }
    if bytes.starts_with(ZIP_SIGNATURE) {
        let content_types = Package::open(bytes)?
            .part("[Content_Types].xml")?
            .unwrap_or_default();
        let has = |kinds: &[&str]| kinds.iter().any(|kind| content_types.contains(kind));
        if has(&[
            "wordprocessingml.document.main+xml",
            "ms-word.document.macroEnabled.main+xml",
        ]) {
            return Ok(Format::Docx);
        }
        if has(&[
            "presentationml.presentation.main+xml",
            "ms-powerpoint.presentation.macroEnabled.main+xml",
        ]) {
            return Ok(Format::Pptx);
        }
        if has(&[
            "spreadsheetml.sheet.main+xml",
            "ms-excel.sheet.macroEnabled.main+xml",
        ]) {
            return Ok(Format::Xlsx);
        }
    }
    Err(unsupported(
        "supported formats are .pdf, .docx, .pptx, and .xlsx",
    ))
}

fn unsupported(message: &str) -> DocumentError {
    DocumentError::new("unsupported_format", message)
}

fn contains_utf16(bytes: &[u8], text: &str) -> bool {
    let needle = text
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect::<Vec<_>>();
    bytes.windows(needle.len()).any(|window| window == needle)
}