


use calamine::{Data, Range};

#[derive(Clone, Copy, Debug,PartialEq, Eq, Hash)]
pub enum Format {
    Pdf,
    Docx,
    Pptx,
    Xlsx,
}

impl Format {
    pub fn name(self) -> &'static str {
        match self {
            Format::Pdf => "pdf",
            Format::Docx => "docx",
            Format::Pptx => "pptx",
            Format::Xlsx => "xlsx",
        }
    }


    pub(crate) fn unit(self) -> &'static str {
        match self {
            Format::Pdf => "page",
            Format::Docx => "block",
            Format::Pptx => "slide",
            Format::Xlsx => "cell",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UnitKind {
    Page,
    Slide,
    Heading(u32),
    Paragraph,
    ListItem,
    Table,
}

impl UnitKind {
    pub(crate) fn block_name(self) -> Option<&'static str> {
        match self {
            Self::Heading(_) => Some("heading"),
            Self::Paragraph => Some("paragraph"),
            Self::ListItem => Some("list_item"),
            Self::Table => Some("table"),
            Self::Page | Self::Slide => None,
        }
    }
}


#[derive(Debug)]
pub(crate) struct Unit {
    pub(crate) kind: UnitKind,
    pub(crate) text: String,
    pub(crate) hidden: bool,
    pub(crate) has_notes: bool,
}

impl Unit {
    pub(crate) fn new(kind: UnitKind, text: String) -> Self {
        Self {
            kind,
            text,
            hidden: false,
            has_notes: false,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Warning {
    pub(crate) code: &'static str,
    pub(crate) message: String,
}

#[derive(Debug)]
pub(crate) struct ParsedDocument {
    pub(crate) format: Format,
    pub(crate) units: Vec<Unit>,
    pub(crate) warnings: Vec<Warning>,
}

pub(crate) struct Sheet {
    pub(crate) name: String,
    pub(crate) visibility: &'static str,
    pub(crate) cells: Range<Data>,
}

pub(crate) struct ParsedWorkbook {
    pub(crate) sheets: Vec<Sheet>,
}

pub(crate) enum Parsed {
    Document(ParsedDocument),
    Workbook(ParsedWorkbook),
}