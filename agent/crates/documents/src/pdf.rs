//! PDF text extraction, one unit per page.

use crate::model::{Format, ParsedDocument, Unit, UnitKind, Warning};
use crate::text::clean_page;
use crate::DocumentError;
use lopdf::{dictionary, Dictionary, Document, Object, ObjectId};
use std::collections::{BTreeMap, HashSet};

#[cfg(test)]
mod tests;

pub(crate) fn parse(bytes: &[u8]) -> Result<ParsedDocument, DocumentError> {
    let mut document = Document::load_mem(bytes).map_err(pdf_error)?;
    if document.is_encrypted() {
        document.decrypt("").map_err(|_| {
            DocumentError::new(
                "encrypted_document",
                "the document is encrypted or password-protected",
            )
        })?;
    }
    // Keep the original layout-aware extractor for other pages. The alternate
    // document is created only when a page needs the named Unicode CMap decoder.
    let mut alternate = None;
    let mut units = Vec::new();
    for (number, id) in document.get_pages() {
        // lopdf skips missing content objects; surface them as parse errors
        // instead of misclassifying a damaged page as having no text layer.
        for content in document.get_page_contents(id) {
            document
                .get_object(content)
                .and_then(Object::as_stream)
                .map_err(pdf_error)?;
        }
        let text = if needs_named_cmap_decoder(&document, id)? {
            let alternate = alternate.get_or_insert_with(|| document.clone());
            prepare_named_cmap_page(alternate, id)?;
            alternate.extract_text(&[number]).map_err(pdf_error)?
        } else {
            let mut text = String::new();
            let mut output = pdf_extract::PlainTextOutput::new(&mut text);
            pdf_extract::output_doc_page(&document, &mut output, number).map_err(pdf_error)?;
            text
        };
        units.push(Unit::new(UnitKind::Page, clean_page(&text)));
    }
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

fn pdf_error(error: impl std::fmt::Display) -> DocumentError {
    DocumentError::parse(format!("the PDF could not be read: {error}"))
}

fn is_unicode_cmap(name: &[u8]) -> bool {
    matches!(name, b"UniGB-UCS2-H" | b"UniGB-UTF16-H")
}

fn needs_named_cmap_decoder(document: &Document, page: ObjectId) -> Result<bool, DocumentError> {
    let mut needed = false;
    for font in page_fonts(document, page)?.values() {
        if font.get(b"Subtype").and_then(Object::as_name).ok() != Some(b"Type0") {
            continue;
        }
        if let Object::Name(name) = font.get_deref(b"Encoding", document).map_err(pdf_error)? {
            if is_unicode_cmap(name) {
                needed = true;
            } else if !matches!(name.as_slice(), b"Identity-H" | b"Identity-V") {
                return Err(DocumentError::parse(format!(
                    "unsupported PDF font encoding: {}",
                    String::from_utf8_lossy(name)
                )));
            }
        }
    }
    Ok(needed)
}

fn prepare_named_cmap_page(document: &mut Document, page: ObjectId) -> Result<(), DocumentError> {
    // lopdf's text extractor does not traverse Form XObjects. Refuse these
    // pages rather than silently returning only their top-level text.
    reject_form_xobjects(document, page)?;
    let mut fonts = Dictionary::new();
    for (name, font) in page_fonts(document, page)? {
        let mut font = font.clone();
        if font.has(b"ToUnicode") {
            // lopdf normally checks Encoding first. Prefer the explicit mapping
            // and verify it parsed instead of accepting its standard fallback.
            font.remove(b"Encoding");
            if !matches!(
                font.get_font_encoding(document).map_err(pdf_error)?,
                lopdf::Encoding::UnicodeMapEncoding(_)
            ) {
                return Err(DocumentError::parse(
                    "the PDF font ToUnicode map could not be read",
                ));
            }
        } else if font.get(b"Subtype").and_then(Object::as_name).ok() == Some(b"Type0") {
            let encoding = font.get_deref(b"Encoding", document).map_err(pdf_error)?;
            if !matches!(encoding, Object::Name(name) if is_unicode_cmap(name)) {
                return Err(DocumentError::parse(
                    "the PDF page combines a named Chinese CMap with a font requiring a ToUnicode map",
                ));
            }
        }
        fonts.set(name, font);
    }
    // Flatten the effective fonts on this private page copy, covering direct,
    // indirect, and inherited font dictionaries without changing the input PDF.
    document
        .get_dictionary_mut(page)
        .map_err(pdf_error)?
        .set("Resources", dictionary! { "Font" => fonts });
    Ok(())
}

fn reject_form_xobjects(document: &Document, page: ObjectId) -> Result<(), DocumentError> {
    let content = document
        .get_and_decode_page_content(page)
        .map_err(pdf_error)?;
    let resources = page_resources(document, page)?;
    for operation in content.operations.iter().filter(|op| op.operator == "Do") {
        let name = operation
            .operands
            .first()
            .ok_or_else(|| DocumentError::parse("the PDF XObject invocation has no name"))?
            .as_name()
            .map_err(pdf_error)?;
        let object = resources
            .ok_or_else(|| DocumentError::parse("the PDF page has no resources"))?
            .get_deref(b"XObject", document)
            .and_then(Object::as_dict)
            .and_then(|objects| objects.get_deref(name, document))
            .map_err(pdf_error)?;
        if object
            .as_stream()
            .map_err(pdf_error)?
            .dict
            .get(b"Subtype")
            .and_then(Object::as_name)
            .map_err(pdf_error)?
            == b"Form"
        {
            return Err(DocumentError::parse(
                "PDF pages combining a named Chinese CMap with Form XObjects are not supported",
            ));
        }
    }
    Ok(())
}

fn page_resources(
    document: &Document,
    mut page: ObjectId,
) -> Result<Option<&Dictionary>, DocumentError> {
    let mut visited = HashSet::new();
    loop {
        if !visited.insert(page) {
            return Err(DocumentError::parse(
                "the PDF page resource inheritance is cyclic",
            ));
        }
        let node = document.get_dictionary(page).map_err(pdf_error)?;
        if node.has(b"Resources") {
            return node
                .get_deref(b"Resources", document)
                .and_then(Object::as_dict)
                .map(Some)
                .map_err(pdf_error);
        }
        if !node.has(b"Parent") {
            return Ok(None);
        }
        page = node
            .get(b"Parent")
            .and_then(Object::as_reference)
            .map_err(pdf_error)?;
    }
}

fn page_fonts(
    document: &Document,
    page: ObjectId,
) -> Result<BTreeMap<Vec<u8>, &Dictionary>, DocumentError> {
    let mut fonts = BTreeMap::new();
    if let Some(resources) = page_resources(document, page)? {
        if resources.has(b"Font") {
            let entries = resources
                .get_deref(b"Font", document)
                .and_then(Object::as_dict)
                .map_err(pdf_error)?;
            for (name, font) in entries {
                let (_, font) = document.dereference(font).map_err(pdf_error)?;
                fonts.insert(name.clone(), font.as_dict().map_err(pdf_error)?);
            }
        }
    }
    Ok(fonts)
}
