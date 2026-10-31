use super::detect;
use super::model::{Format, Parsed};
use super::office;
use super::query;
use super::xlsx;
use office_oxide::create::create_from_markdown_to_writer;
use office_oxide::DocumentFormat;
use std::io::{Cursor, Write};

fn office_fixture(format: DocumentFormat) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    create_from_markdown_to_writer(
        "# Heading\n\nA paragraph.\n\n- one\n- two\n\n| A | B |\n|---|---|\n| 1 | 2 |",
        format,
        &mut output,
    )
    .unwrap();
    output.into_inner()
}

fn ods_fixture() -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(&mut output);
    let options = zip::write::SimpleFileOptions::default();
    zip.start_file("mimetype", options).unwrap();
    zip.write_all(b"application/vnd.oasis.opendocument.spreadsheet")
        .unwrap();
    zip.start_file("content.xml", options).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" office:version="1.2">
  <office:body><office:spreadsheet><table:table table:name="Sheet1">
    <table:table-row><table:table-cell office:value-type="string"><text:p>Hello</text:p></table:table-cell></table:table-row>
  </table:table></office:spreadsheet></office:body>
</office:document-content>"#,
    )
    .unwrap();
    zip.start_file("META-INF/manifest.xml", options).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.2">
  <manifest:file-entry manifest:full-path="/" manifest:media-type="application/vnd.oasis.opendocument.spreadsheet"/>
  <manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>
</manifest:manifest>"#,
    )
    .unwrap();
    zip.finish().unwrap();
    output.into_inner()
}

#[test]
fn office_formats_are_detected_and_normalized() {
    for (office_format, format) in [
        (DocumentFormat::Docx, Format::Docx),
        (DocumentFormat::Pptx, Format::Pptx),
        (DocumentFormat::Xlsx, Format::Xlsx),
    ] {
        let bytes = office_fixture(office_format);
        assert_eq!(detect::detect(&bytes).unwrap(), format);
        match format {
            Format::Docx | Format::Pptx => {
                let parsed = office::parse(&bytes, format).unwrap();
                assert_eq!(parsed.format, format);
                assert!(!parsed.units.is_empty());
            }
            Format::Xlsx => {
                let workbook = xlsx::parse(&bytes, format).unwrap();
                assert_eq!(workbook.format, format);
                let summary = query::inspect(&Parsed::Workbook(workbook), "test-p1", bytes.len());
                assert_eq!(summary["format"], "xlsx");
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn ods_is_detected_and_read_as_a_workbook() {
    let bytes = ods_fixture();
    assert_eq!(detect::detect(&bytes).unwrap(), Format::Ods);
    let workbook = xlsx::parse(&bytes, Format::Ods).unwrap();
    assert_eq!(workbook.format, Format::Ods);
    assert_eq!(workbook.sheets.len(), 1);
    let summary = query::inspect(&Parsed::Workbook(workbook), "test-p1", bytes.len());
    assert_eq!(summary["format"], "ods");
}
