//! PPTX slides in presentation order, each with its speaker nots.

use crate::model::{Format, ParsedDocument, Unit, UnitKind};
use crate::ooxml::{attribute, xml_error, Package};
use crate::DocumentError;
use quick_xml::events::Event;
use quick_xml::Reader;

/// Marks the start of the speaker notes inside a slide's text.
pub(crate) const NOTES_HEADING: &str = "[Speaker notes]";
const OFFICE_DOCUMENT: &str = "/officeDocument";
const NOTES_SLIDE: &str = "/notesSlide";
/// Placeholders whose text is boilerplate rather than content.
const SLIDE_SKIPPED_PLACEHOLDERS: &[&str] = &["sldNum"];
const NOTES_SKIPPED_PLACEHOLDERS: &[&str] = &["dt", "sldNum", "hdr", "ftr"];

pub(crate) fn parse(bytes: &[u8]) -> Result<ParsedDocument, DocumentError> {
    let mut package = Package::open(bytes)?;
    let main = package
        .relationships("")?
        .into_iter()
        .find(|relationship| relationship.kind.ends_with(OFFICE_DOCUMENT))
        .map(|relationship| relationship.target)
        .ok_or_else(|| DocumentError::parse("the presentation part is missing"))?;
    let presentation = package
        .part(&main)?
        .ok_or_else(|| DocumentError::parse("the presentation part is missing"))?;
    let relationships = package.relationships(&main)?;
    let mut units = Vec::new();
    for slide_id in slide_ids(&presentation)? {
        let Some(slide_part) = relationships
            .iter()
            .find(|relationship| relationship.id == slide_id)
            .map(|relationship| relationship.target.clone())
        else {
            continue;
        };
        let Some(slide_xml) = package.part(&slide_part)? else {
            continue;
        };
        let slide = extract(&slide_xml, SLIDE_SKIPPED_PLACEHOLDERS)?;
        let notes_part = package
            .relationships(&slide_part)?
            .into_iter()
            .find(|relationship| relationship.kind.ends_with(NOTES_SLIDE))
            .map(|relationship| relationship.target);
        let notes = match notes_part {
            Some(part) => match package.part(&part)? {
                Some(xml) => extract(&xml, NOTES_SKIPPED_PLACEHOLDERS)?.text,
                None => String::new(),
            },
            None => String::new(),
        };
        let mut text = slide.text;
        let has_notes = !notes.is_empty();
        if has_notes {
            if !text.is_empty() {
                text.push_str("\n\n");
            }
            text.push_str(NOTES_HEADING);
            text.push('\n');
            text.push_str(&notes);
        }
        let mut unit = Unit::new(UnitKind::Slide, text);
        unit.hidden = slide.hidden;
        unit.has_notes = has_notes;
        units.push(unit);
    }
    Ok(ParsedDocument {
        format: Format::Pptx,
        units,
        warnings: Vec::new(),
    })
}

/// Relationship IDs of `p:sldIdLst` entries, in presentation order.
fn slide_ids(presentation: &str) -> Result<Vec<String>, DocumentError> {
    let mut reader = Reader::from_str(presentation);
    let mut ids = Vec::new();
    loop {
        match reader.read_event().map_err(xml_error)? {
            Event::Start(element) | Event::Empty(element)
                if element.local_name().as_ref() == "sldId" =>
            {
                // The relationship ID is the namespaced `r:id`, not the numeric `id`.
                let id = element
                    .attributes()
                    .flatten()
                    .find(|attribute| {
                        attribute.key.local_name().as_ref() == "id"
                        && attribute.key.prefix().is_some()
                    })
                    .and_then(|attribute| {
                        quick_xml::escape::unescape(&attribute.value)
                            .ok()
                            .map(|value| value.into_owned())
                    });
                if let Some(id) = id {
                    ids.push(id);
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(ids)
}

struct Extracted {
    text: String,
    hidden: bool,
}

#[derive(Default)]
struct Shape {
    skipped: bool,
    lines: Vec<String>,
}

#[derive(Default)]
struct TableState {
    rows: Vec<String>,
    cells: Vec<String>,
    cell: Option<Vec<String>>,
}

/// Text of a slide or notes part: one line per paragraph, table rows as
/// ` | `-separated cells, skipping shapes that are placeholders of the
/// given types.
fn extract(xml: &str, skipped_placeholders: &[&str]) -> Result<Extracted, DocumentError> {
    let mut reader = Reader::from_str(xml);
    let mut hidden = false;
    let mut lines = Vec::new();
    let mut shapes: Vec<Shape> = Vec::new();
    let mut tables: Vec<TableState> = Vec::new();
    let mut paragraph: Option<String> = None;
    let mut in_text = false;
    loop {
        let event = reader.read_event().map_err(xml_error)?;
        match event {
            Event::Start(element) => match element.local_name().as_ref() {
                "sld" => {
                    hidden = matches!(attribute(&element, "show").as_deref(), Some("0" | "false"))
                }
                "sp" => shapes.push(Shape::default()),
                "ph" => skip_placeholder(&element, &mut shapes, skipped_placeholders),
                "p" => paragraph = Some(String::new()),
                "t" => in_text = true,
                "tbl" => tables.push(TableState::default()),
                "tc" => {
                    if let Some(table) = tables.last_mut() {
                        table.cell = Some(Vec::new());
                    }
                }
                _ => {}
            }
            Event::Empty(element) => match element.local_name().as_ref() {
                "sld" => {
                    hidden = matches!(attribute(&element, "show").as_deref(), Some("0" | "false"))
                }
                "ph" => skip_placeholder(&element, &mut shapes, skipped_placeholders),
                "br" => {
                    if let Some(paragraph) = paragraph.as_mut() {
                        paragraph.push('\n');
                    }
                }
                _ => {}
            },
            Event::Text(text) if in_text => {
                if let Some(paragraph) = paragraph.as_mut() {
                    paragraph.push_str(&text.xml10_content());
                }
            }
            Event::GeneralRef(reference) if in_text => {
                if let Some(paragraph) = paragraph.as_mut() {
                    let raw = format!("&{};", reference.into_inner());
                    if let Ok(value) = quick_xml::escape::unescape(&raw) {
                        paragraph.push_str(&value);
                    }
                }
            }
            Event::End(element) => match element.local_name().as_ref() {
                "t" => in_text = false,
                "p" => {
                    if let Some(text) = paragraph.take() {
                        let text = text.trim_end().to_string();
                        if !text.is_empty() {
                            sink(&mut lines, &mut shapes, &mut tables).push(text);
                        }
                    }
                },
                "sp" => {
                    if let Some(shape) = shapes.pop() {
                        if !shape.skipped {
                            sink(&mut lines, &mut shapes, &mut tables).extend(shape.lines);
                        }
                    }
                },
                "tc" => {
                    if let Some(table) = tables.last_mut() {
                        let cell = table.cell.take().unwrap_or_default();
                        table
                            .cells
                            .push(crate::text::collapse_whitespace(&cell.join(" ")));
                    }
                },
                "tr" => {
                    if let Some(table) = tables.last_mut() {
                        let cells = std::mem::take(&mut table.cells);
                        if cells.iter().any(|cell| !cell.is_empty()) {
                            table.rows.push(cells.join(" | "));
                        }
                    }
                },
                "tbl" => {
                    if let Some(table) = tables.pop() {
                        if !table.rows.is_empty() {
                            sink(&mut lines, &mut shapes, &mut tables).push(table.rows.join("\n"));
                        }
                    }
                },
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(Extracted {
        text: lines.join("\n"),
        hidden,
    })
}

fn skip_placeholder(
    element: &quick_xml::events::BytesStart<'_>,
    shapes: &mut [Shape],
    skipped_placeholders: &[&str],
) {
    let kind = attribute(element, "type");
    if kind.is_some_and(|kind| skipped_placeholders.contains(&kind.as_str())) {
        if let Some(shape) = shapes.last_mut() {
            shape.skipped = true;
        }
    }
}

/// Where a finished paragraph or table goes: the open table cell, else the
/// open shape, else the top level.
fn sink<'a>(
    lines: &'a mut Vec<String>,
    shapes: &'a mut [Shape],
    table: &'a mut [TableState],
) -> &'a mut Vec<String> {
    if let Some(cell) = table.last_mut().and_then(|t| t.cell.as_mut()) {
        return cell;
    }
    if let Some(shape) = shapes.last_mut() {
        return &mut shape.lines;
    }
    lines
}