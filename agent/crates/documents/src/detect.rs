//! Format detection from file content

use crate::model::Format;
use crate::ooxml::Package;
use crate::DocumentError;
use office_oxide::cfb::CfbReader;
use std::io::Cursor;

const PDF_SIGNATURE: &[u8] = b"%PDF-";
const ZIP_SIGNATURE: &[u8] = b"PK\x03\x04";

const COMPOUND_FILE_SIGNATURE: &[u8] = &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

pub(crate) fn detect(bytes: &[u8]) -> Result<Format, DocumentError> {
    // Some producers write a few bytes before the PDF header.
    if bytes.len() >= PDF_SIGNATURE.len()
        && bytes[..bytes.len().min(1024)]
            .windows(PDF_SIGNATURE.len())
            .any(|window| window == PDF_SIGNATURE)
    {
        return Ok(Format::Pdf);
    }
    if bytes.starts_with(COMPOUND_FILE_SIGNATURE) {
        let reader = CfbReader::new(Cursor::new(bytes))
            .map_err(|_| unsupported("the legacy Office container is unreadable"))?;
        if reader.has_stream("EncryptionInfo") && reader.has_stream("EncryptedPackage") {
            return Err(DocumentError::new(
                "encrypted_document",
                "the document is encrypted or password-protected",
            ));
        }
        if reader.has_stream("WordDocument") {
            return Ok(Format::Doc);
        }
        if reader.has_stream("PowerPoint Document") || reader.has_stream("PP97_DUALSTORAGE") {
            return Ok(Format::Ppt);
        }
        if reader.has_stream("Workbook") || reader.has_stream("Book") {
            return Ok(Format::Xls);
        }
        return Err(unsupported(
            "the legacy Office document type is not supported",
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
        if package_mimetype(bytes)?.as_deref()
            == Some("application/vnd.oasis.opendocument.spreadsheet")
        {
            return Ok(Format::Ods);
        }
    }
    Err(unsupported(
        "supported formats are .pdf, .doc, .docx, .ppt, .pptx, .xls, .xlsx, and .ods",
    ))
}

fn package_mimetype(bytes: &[u8]) -> Result<Option<String>, DocumentError> {
    let mut package = Package::open(bytes)?;
    Ok(package
        .part("mimetype")?
        .map(|value| value.trim().to_string()))
}

fn unsupported(message: &str) -> DocumentError {
    DocumentError::new("unsupported_format", message)
}
