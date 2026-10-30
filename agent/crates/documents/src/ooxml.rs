//! Bounded reading of OOXML package parts and relationships

use crate::DocumentError;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use std::io::{Cursor, Read};
use zip::ZipArchive;

/// Largest single part read, uncompressed.
const MAX_PART_BYTES: u64 = 32 * 1024 * 1024;
/// Largest total of parts read from one package, uncompressed.
const MAX_TOTAL_BYTES: u64 = 256 * 1024 * 1024;

pub(crate) struct Package<'a> {
    archive: ZipArchive<Cursor<&'a [u8]>>,
    read_bytes: u64,
}

pub(crate) struct Relationship {
    pub(crate) id: String,
    pub(crate) kind: String,
    ///Package part name without leading slash, e.g. "word/document.xml"
    pub(crate) target: String,
}

impl<'a> Package<'a> {
    pub(crate) fn open(bytes: &'a [u8]) -> Result<Self, DocumentError> {
        let archive = ZipArchive::new(Cursor::new(bytes))
            .map_err(|_| DocumentError::parse("ths file is not a valid OOXML package"))?;
        Ok(Self {
            archive,
            read_bytes: 0,
        })
    }

    /// The UTF-8 text of part `name`, or `None` when the part is absent.
    pub(crate) fn part(&mut self, name: &str) -> Result<Option<String>, DocumentError> {
        let mut file = match self.archive.by_name(name) {
            Ok(file) => file,
            Err(zip::result::ZipError::FileNotFound) => return Ok(None),
            Err(_) => return Err(DocumentError::parse(format!("part {name} is unreadable"))),
        };
        let mut text = String::new();
        let read = (&mut file)
            .take(MAX_PART_BYTES + 1)
            .read_to_string(&mut text)
            .map_err(|_| DocumentError::parse(format!("part {name} is unreadable")))?
            as u64;
        self.read_bytes += read;
        if read > MAX_PART_BYTES || self.read_bytes > MAX_TOTAL_BYTES {
            return Err(DocumentError::parse(
                "the document expands beyond the size limit",
            ))
        }
        Ok(Some(text))
    }

    /// Internal relationships of part `source`, with resolved targets.
    pub(crate) fn relationships(
        &mut self,
        source: &str,
    ) -> Result<Vec<Relationship>, DocumentError> {
        let (directory, file) = source.rsplit_once('/').unwrap_or(("", source));
        let rels_name = if directory.is_empty() {
            format!("_rels/{file}.rels")
        } else {
            format!("{directory}/_rels/{file}.rels")
        };
        let Some(xml) = self.part(&rels_name)? else {
            return Ok(Vec::new());
        };
        let mut relationships = Vec::new();
        let mut reader = Reader::from_str(&xml);
        loop {
            match reader.read_event().map_err(xml_error)? {
                Event::Start(element) | Event::Empty(element)
                    if element.local_name().as_ref() == "Relationship" =>
                    {
                        if attribute(&element, "TargetMode").as_deref() == Some("External") {
                            continue;
                        }
                        let (Some(id), Some(kind), Some(target)) = (
                            attribute(&element, "Id"),
                            attribute(&element, "Type"),
                            attribute(&element, "Target"),
                        ) else {
                            continue;
                        };
                        relationships.push(Relationship {
                            id,
                            kind,
                            target: resolve(directory, &target),
                        });
                    }
                Event::Eof => break,
                _ => {}
            }
        }
        Ok(relationships)
    }
}

/// The value of attribute `name` (matched by local name), unescaped.
pub(crate) fn attribute(element: &BytesStart, name: &str) -> Option<String> {
    element
        .attributes()
        .flatten()
        .find(|attribute| attribute.key.local_name().as_ref() == name)
        .and_then(|attribute| {
            quick_xml::escape::unescape(&attribute.value)
                .ok()
                .map(|value| value.into_owned())
        })
}

pub(crate) fn xml_error(_: quick_xml::Error) -> DocumentError {
    DocumentError::parse("the document contains malformed XML")
}

/// Resolves a relationship target against the source part's directory.
fn resolve(directory: &str, target: &str) -> String {
    let mut parts: Vec<&str> = if target.starts_with('/') {
        Vec::new()
    } else {
        directory
            .split('/')
            .filter(|part| !part.is_empty())
            .collect()
    };
    for part in target.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            part => parts.push(part),
        }
    }
    parts.join("/")
}