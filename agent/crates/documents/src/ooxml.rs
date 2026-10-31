//! Bounded access to ZIP package parts used by format detection.

use crate::DocumentError;
use std::io::{Cursor, Read};
use zip::ZipArchive;

const MAX_PART_BYTES: u64 = 32 * 1024 * 1024;

pub(crate) struct Package<'a> {
    archive: ZipArchive<Cursor<&'a [u8]>>,
}

impl<'a> Package<'a> {
    pub(crate) fn open(bytes: &'a [u8]) -> Result<Self, DocumentError> {
        let archive = ZipArchive::new(Cursor::new(bytes))
            .map_err(|_| DocumentError::parse("the file is not a valid ZIP package"))?;
        Ok(Self { archive })
    }

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
            .map_err(|_| DocumentError::parse(format!("part {name} is unreadable")))?;
        if read as u64 > MAX_PART_BYTES {
            return Err(DocumentError::parse(
                "the package part exceeds the size limit",
            ));
        }
        Ok(Some(text))
    }
}
