//! DOCX body blocks: headings, paragraphs, list items, and tables, in
//! document order. Empty paragraphs produce no block.

use crate::model::{Format, ParsedDocument, Unit,UnitKind};
use crate::DocumentError;
use docx_rs::{
    DocumentChild, InsertChild, Paragraph, ParagraphChild, RunChild, StructuredDataTagChild, Style,
    Table, TableCellContent, TableChild, TableRowChild,
};
use std::collections::HashMap;

/// Longest style inheritance chain followed when looking for a heading level.
const MAX_STYLE_DEPTH: usize = 8;

pub(crate) fn parse(bytes: &[u8]) -> Result<ParsedDocument, DocumentError> {
    let mut options = docx_rs::ReadDocxOptions::default();
    options.generate_image_previews = false;
    let docx = docx_rs::read_docx_with_options(bytes, options)
        .map_err(|error| DocumentError::parse(format!("the DOCX could not be read: {error}")))?;
    let styles = docx
        .styles
        .styles
        .iter()
        .map(|style| (style.style_id.as_str(), style))
        .collect::<HashMap<_, _>>();
    let mut blocks = Blocks {
        styles,
        units: Vec::new(),
    };
    for child in &docx.document.children {
        match child {
            DocumentChild::Paragraph(paragraph) => blocks.paragraph(paragraph),
            DocumentChild::Table(table) => blocks.table(table),
            DocumentChild::StructuredDataTag(tag) => blocks.tagged(&tag.children),
            _ => {}
        }
    }
    Ok(ParsedDocument {
        format: Format::Docx,
        units: blocks.units,
        warnings: Vec::new(),
    })
}

struct Blocks<'a> {
    styles: HashMap<&'a str, &'a Style>,
    units: Vec<Unit>,
}

impl Blocks<'_> {
    fn paragraph(&mut self, paragraph: &Paragraph) {
        let mut text = String::new();
        paragraph_text(&paragraph.children, &mut text);
        if text.trim().is_empty() {
            return;
        }
        let kind = if let Some(level) = self.heading_level(paragraph) {
            UnitKind::Heading(level)
        } else if paragraph
            .property
            .numbering_property
            .as_ref()
            .is_some_and(|numbering| numbering.id.as_ref().is_some_and(|id| id.id != 0))
        {
            UnitKind::ListItem
        } else {
            UnitKind::Paragraph
        };
        self.units.push(Unit::new(kind, text));
    }

    fn table(&mut self, table: &Table) {
        let text = table_text(table);
        if !text.is_empty() {
            self.units.push(Unit::new(UnitKind::Table, text));
        }
    }

    fn tagged(&mut self, children: &[StructuredDataTagChild]) {
        let mut runs = String::new();
        for child in children {
            match child {
                StructuredDataTagChild::Run(run) => run_text(&run.children, &mut runs),
                StructuredDataTagChild::Paragraph(paragraph) => self.paragraph(paragraph),
                StructuredDataTagChild::Table(table) => self.table(table),
                StructuredDataTagChild::StructuredDataTag(tag) => self.tagged(&tag.children),
                _ => {}
            }
        }
        if !runs.trim().is_empty() {
            self.units.push(Unit::new(UnitKind::Paragraph, runs));
        }
    }

    fn heading_level(&self, paragraph: &Paragraph) -> Option<u32> {
        if let Some(level) = &paragraph.property.outline_lvl {
            return heading_from_outline(level.v);
        }
        let mut style_id = paragraph.property.style.as_ref()?.val.clone();
        for _ in 0..MAX_STYLE_DEPTH {
            if let Some(style) = heading_from_style_id(&style_id) {
                return Some(style);
            }
            let style = self.styles.get(style_id.as_str())?;
            if let Some(level) = &style.paragraph_property.outline_lvl {
                return heading_from_outline(level.v);
            }
            // `BasedOn` keeps its value private; it serializes as the style ID.
            style_id = serde_json::to_value(style.based_on.as_ref()?)
                .ok()?
                .as_str()?
                .to_string();
        }
        None
    }
}

/// Outline levels 0-8 are headings 1-9; level 9 is body text
fn heading_from_outline(level: usize) -> Option<u32> {
    (level < 9).then_some(level as u32 + 1)
}

fn heading_from_style_id(style_id: &str) -> Option<u32> {
    if style_id.eq_ignore_ascii_case("title") {
        return Some(1);
    }
    let digits = style_id
        .get(..7)
        .filter(|prefix| prefix.eq_ignore_ascii_case("heading"))
        .map(|_| &style_id[..7])?;
    digits.parse().ok().filter(|level| (1..=9).contains(level))
}

fn paragraph_text(children: &[ParagraphChild], text: &mut String) {
    for child in children {
        match child {
            ParagraphChild::Run(run) => run_text(&run.children, text),
            ParagraphChild::Hyperlink(link) => paragraph_text(&link.children, text),
            ParagraphChild::Insert(insert) => {
                for child in &insert.children {
                    if let InsertChild::Run(run) = child {
                        run_text(&run.children, text);
                    }
                }
            }
            ParagraphChild::StructuredDataTag(tag) => {
                for child in &tag.children {
                    match child {
                        StructuredDataTagChild::Run(run) => run_text(&run.children, text),
                        StructuredDataTagChild::Paragraph(paragraph) => {
                            paragraph_text(&paragraph.children, text);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

fn run_text(children: &[RunChild], text: &mut String) {
    for child in children {
        match child {
            RunChild::Text(value) => text.push_str(&value.text),
            RunChild::Tab(_) | RunChild::PTab(_) => text.push('\t'),
            RunChild::Break(_) | RunChild::CarriageReturn(_) => text.push('\n'),
            _ => {}
        }
    }
}

/// Rows on separate lines, cells separated by ` | `; empty rows are dropped.
fn table_text(table: &Table) -> String {
    let mut rows = Vec::new();
    for TableChild::TableRow(row) in &table.rows {
        let cells = row
            .cells
            .iter()
                    .map(|TableRowChild::TableCell(cell)| {
                        let mut parts = Vec::new();
                        for content in &cell.children {
                            match content {
                                TableCellContent::Paragraph(paragraph) => {
                                    let mut text = String::new();
                                    paragraph_text(&paragraph.children, &mut text);
                                    parts.push(text);
                                }
                                TableCellContent::Table(table) => parts.push(table_text(table)),
                                _ => {}
                            }
                        }
                        crate::text::collapse_whitespace(&parts.join(" "))
                    }
            ).collect::<Vec<_>>();
        if cells.iter().any(|cell| !cell.is_empty()) {
            rows.push(cells.join(" | "));
        }
    }
    rows.join("\n")
}
