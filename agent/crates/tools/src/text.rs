//! Strict round-trip text encodings shared by read, edit, and retained diffs.
use crate::error::fatal;
use suncode_common::BusinessError;

pub(crate) const MAX_EDIT_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Copy)]
pub(crate) enum Encoding {
    Utf8,
    Utf8Bom,
    Utf16Le,
}

pub(crate) fn decode(bytes: &[u8]) -> Result<(String, Encoding), BusinessError> {
    let invalid = || {
        fatal("encoding_unsupported", "expected valid UTF-8 or BOM-marked UTF-16LE; preserve the encoding or explicitly convert the file before editing")
    };
    if let Some(bytes) = bytes.strip_prefix(&[0xff, 0xfe]) {
        if bytes.len() % 2 != 0 {
            return Err(invalid());
        }
        let units: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        return String::from_utf16(&units)
            .map(|text| (text, Encoding::Utf16Le))
            .map_err(|_| invalid());
    }
    let (bytes, encoding) = match bytes.strip_prefix(&[0xef, 0xbb, 0xbf]) {
        Some(bytes) => (bytes, Encoding::Utf8Bom),
        None => (bytes, Encoding::Utf8),
    };
    std::str::from_utf8(bytes)
        .map(|text| (text.to_owned(), encoding))
        .map_err(|_| invalid())
}

impl Encoding {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Utf8 => "utf-8",
            Self::Utf8Bom => "utf-8-bom",
            Self::Utf16Le => "utf-16le",
        }
    }
    pub(crate) fn encoded_len(self, text: &str) -> usize {
        match self {
            Self::Utf8 => text.len(),
            Self::Utf8Bom => 3 + text.len(),
            Self::Utf16Le => 2 + text.encode_utf16().count() * 2,
        }
    }
    pub(crate) fn encode(self, text: &str) -> Vec<u8> {
        match self {
            Self::Utf8 => text.as_bytes().to_vec(),
            Self::Utf8Bom => [b"\xef\xbb\xbf".as_slice(), text.as_bytes()].concat(),
            Self::Utf16Le => {
                let mut bytes = vec![0xff, 0xfe];
                bytes.extend(text.encode_utf16().flat_map(u16::to_le_bytes));
                bytes
            }
        }
    }
}
