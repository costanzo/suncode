//! DOC/DOCX and PPT/PPTX parsing through office_oxide.

use crate::model::{Format, ParsedDocument, Unit, UnitKind, Warning};
use crate::DocumentError;
use office_oxide::ir::{self, Element, List};
use office_oxide::{Document, DocumentFormat};
use std::io::Cursor;

pub(crate) fn parse(bytes: &[u8], format: Format) -> Result<ParsedDocument, DocumentError> {
    let office_format = match format {
        Format::Doc => DocumentFormat::Doc,
        Format::Docx => DocumentFormat::Docx,
        Format::Ppt => DocumentFormat::Ppt,
        Format::Pptx => DocumentFormat::Pptx,
        _ => {
            return Err(DocumentError::invalid(
                "office parser received a non-Office format",
            ))
        }
    };
    let document =
        Document::from_reader(Cursor::new(bytes.to_vec()), office_format).map_err(|error| {
            DocumentError::parse(format!(
                "the {} file could not be read: {error}",
                format.name()
            ))
        })?;
    let ir = document.to_ir();
    let warnings = warnings(&ir);
    let units = if matches!(format, Format::Ppt | Format::Pptx) {
        presentation_units(&ir)
    } else {
        word_units(&ir)
    };
    Ok(ParsedDocument {
        format,
        units,
        warnings,
    })
}

fn warnings(document: &ir::DocumentIR) -> Vec<Warning> {
    let mut warnings = document
        .metadata
        .warnings
        .iter()
        .cloned()
        .map(|message| Warning {
            code: "parser_warning",
            message,
        })
        .collect::<Vec<_>>();
    if document.metadata.text_truncated {
        warnings.push(Warning {
            code: "text_truncated",
            message: "the Office parser truncated extracted text at its safety limit".to_string(),
        });
    }
    warnings
}

fn word_units(document: &ir::DocumentIR) -> Vec<Unit> {
    let mut units = Vec::new();
    for section in &document.sections {
        append_word_elements(&section.elements, &mut units);
    }
    units
}

fn append_word_elements(elements: &[Element], units: &mut Vec<Unit>) {
    for element in elements {
        match element {
            Element::Heading(heading) => push_unit(
                units,
                UnitKind::Heading(u32::from(heading.clamped_level())),
                ir::inline_to_text(&heading.content),
            ),
            Element::Paragraph(paragraph) => push_unit(
                units,
                UnitKind::Paragraph,
                ir::inline_to_text(&paragraph.content),
            ),
            Element::List(list) => append_list(list, units),
            Element::Table(table) => push_unit(units, UnitKind::Table, table_text(table)),
            Element::TextBox(text_box) => append_word_elements(&text_box.content, units),
            Element::Footnote(note) | Element::Endnote(note) => {
                append_word_elements(&note.content, units)
            }
            Element::CodeBlock(code) => push_unit(units, UnitKind::Paragraph, code.content.clone()),
            _ => {}
        }
    }
}

fn append_list(list: &List, units: &mut Vec<Unit>) {
    for item in &list.items {
        let text = elements_text(&item.content);
        push_unit(units, UnitKind::ListItem, text);
        if let Some(nested) = &item.nested {
            append_list(nested, units);
        }
    }
}

fn presentation_units(document: &ir::DocumentIR) -> Vec<Unit> {
    document
        .sections
        .iter()
        .map(|section| {
            let mut text = elements_text(&section.elements);
            let notes = section
                .speaker_notes
                .as_deref()
                .map(elements_text)
                .unwrap_or_default();
            let has_notes = !notes.trim().is_empty();
            if has_notes {
                if !text.is_empty() {
                    text.push_str("\n\n");
                }
                text.push_str("[Speaker notes]\n");
                text.push_str(&notes);
            }
            let mut unit = Unit::new(UnitKind::Slide, text);
            unit.hidden = section.hidden;
            unit.has_notes = has_notes;
            unit
        })
        .collect()
}

fn push_unit(units: &mut Vec<Unit>, kind: UnitKind, text: String) {
    if !text.trim().is_empty() {
        units.push(Unit::new(kind, text));
    }
}

fn elements_text(elements: &[Element]) -> String {
    elements
        .iter()
        .map(element_text)
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn element_text(element: &Element) -> String {
    match element {
        Element::Heading(heading) => ir::inline_to_text(&heading.content),
        Element::Paragraph(paragraph) => ir::inline_to_text(&paragraph.content),
        Element::Table(table) => table_text(table),
        Element::List(list) => list_text(list),
        Element::TextBox(text_box) => elements_text(&text_box.content),
        Element::Footnote(note) | Element::Endnote(note) => elements_text(&note.content),
        Element::CodeBlock(code) => code.content.clone(),
        Element::Image(image) => image.alt_text.clone().unwrap_or_default(),
        _ => String::new(),
    }
}

fn list_text(list: &List) -> String {
    list.items
        .iter()
        .map(|item| {
            let mut text = elements_text(&item.content);
            if let Some(nested) = &item.nested {
                let nested_text = list_text(nested);
                if !nested_text.is_empty() {
                    if !text.is_empty() {
                        text.push('\n');
                    }
                    text.push_str(&nested_text);
                }
            }
            text
        })
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn table_text(table: &ir::Table) -> String {
    table
        .rows
        .iter()
        .map(|row| {
            row.cells
                .iter()
                .map(|cell| elements_text(&cell.content))
                .map(|text| crate::text::collapse_whitespace(&text))
                .collect::<Vec<_>>()
                .join(" | ")
        })
        .filter(|row| !row.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
