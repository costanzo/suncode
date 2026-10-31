use super::*;
use crate::model::Parsed;
use crate::{query, ReadRequest, SearchRequest};
use lopdf::content::{Content, Operation};
use lopdf::{Stream, StringFormat};

fn fixture(encoding: &str, texts: &[&str]) -> Document {
    let mut document = Document::with_version("1.7");
    let pages_id = document.new_object_id();
    let descriptor = document.add_object(dictionary! {
        "Type" => "FontDescriptor", "FontName" => "STSong-Light",
        "Flags" => 6, "FontBBox" => vec![0.into(), 0.into(), 1000.into(), 1000.into()],
        "ItalicAngle" => 0, "Ascent" => 880, "Descent" => -120, "CapHeight" => 880, "StemV" => 80,
    });
    let descendant = document.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "CIDFontType0", "BaseFont" => "STSong-Light",
        "CIDSystemInfo" => dictionary! {
            "Registry" => Object::string_literal("Adobe"),
            "Ordering" => Object::string_literal("GB1"), "Supplement" => 4,
        },
        "FontDescriptor" => descriptor, "DW" => 1000,
    });
    let encoding = document.add_object(Object::Name(encoding.as_bytes().to_vec()));
    let font = document.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "Type0", "BaseFont" => "STSong-Light",
        "Encoding" => encoding,
        "DescendantFonts" => vec![descendant.into()],
    });
    let mut kids = Vec::new();
    for text in texts {
        let bytes = text.encode_utf16().flat_map(u16::to_be_bytes).collect();
        let content = Content {
            operations: vec![
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), 12.into()]),
                Operation::new("Tj", vec![Object::String(bytes, StringFormat::Hexadecimal)]),
                Operation::new("ET", vec![]),
            ],
        };
        let content = document.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
        let page = document.add_object(dictionary! {
            "Type" => "Page", "Parent" => pages_id, "Contents" => content,
            "MediaBox" => vec![0.into(), 0.into(), 596.into(), 397.into()],
        });
        kids.push(page.into());
    }
    // Inherited resources, an indirect font dictionary, and an indirect font
    // exercise the same resolution path as ordinary producer-generated PDFs.
    let fonts = document.add_object(dictionary! { "F1" => font });
    document.objects.insert(
        pages_id,
        dictionary! {
            "Type" => "Pages", "Count" => kids.len() as i64, "Kids" => kids,
            "Resources" => dictionary! { "Font" => fonts },
        }
        .into(),
    );
    let catalog = document.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    document.trailer.set("Root", catalog);
    document
}

fn bytes(mut document: Document) -> Vec<u8> {
    let mut bytes = Vec::new();
    document.save_to(&mut bytes).unwrap();
    bytes
}

fn set_to_unicode(document: &mut Document, cmap: &[u8]) {
    let cmap = document.add_object(Stream::new(dictionary! {}, cmap.to_vec()));
    let font = document
        .objects
        .values_mut()
        .find_map(|object| {
            let dictionary = object.as_dict_mut().ok()?;
            (dictionary.get(b"Subtype").and_then(Object::as_name).ok() == Some(b"Type0"))
                .then_some(dictionary)
        })
        .unwrap();
    font.set("ToUnicode", cmap);
}

const TO_UNICODE: &[u8] = b"/CIDInit /ProcSet findresource begin 12 dict begin begincmap
/CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def
/CMapName /Test def /CMapType 2 def
1 begincodespacerange <0000> <FFFF> endcodespacerange
1 beginbfchar <4E2D> <75324E59> endbfchar
endcmap CMapName currentdict /CMap defineresource pop end end";

#[test]
fn chinese_named_cmaps_preserve_pages_text_and_queries() {
    for encoding in ["UniGB-UCS2-H", "UniGB-UTF16-H"] {
        let document = parse(&bytes(fixture(
            encoding,
            &["中文示例 ABC 123", "第二页内容", ""],
        )))
        .unwrap();
        assert_eq!(document.units.len(), 3);
        assert_eq!(document.units[0].text, "中文示例 ABC 123");
        assert_eq!(document.units[1].text, "第二页内容");
        assert!(document.units[2].text.is_empty());
        let parsed = Parsed::Document(document);
        let summary = query::inspect(&parsed, "test-p3", 1);
        assert_eq!(summary["summary"]["page_count"], 3);
        assert_eq!(summary["summary"]["text_layer"], "partial");
        assert_eq!(
            summary["summary"]["pages_without_text"],
            serde_json::json!([3])
        );
        assert_eq!(summary["warnings"][0]["code"], "no_text_layer");
        let Parsed::Document(document) = &parsed else {
            unreachable!()
        };
        let read = query::read(
            document,
            "test-p3",
            &ReadRequest {
                unit: "page".into(),
                start: 1,
                end: 2,
                offset: 1,
                max_chars: Some(3),
            },
        )
        .unwrap();
        assert_eq!(read["segments"][0]["text"], "文示例");
        assert_eq!(read["next_offset"], 4);
        assert_eq!(read["truncated"], true);
        let search = query::search(
            &parsed,
            "test-p3",
            &SearchRequest {
                query: "第二页".into(),
                offset: 0,
                max_results: None,
            },
        )
        .unwrap();
        assert_eq!(search["matches"][0]["location"]["page"], 2);
        assert_eq!(search["matches"][0]["match"], "第二页");
    }
}

#[test]
fn utf16_named_cmap_decodes_supplementary_characters() {
    let document = parse(&bytes(fixture("UniGB-UTF16-H", &["中文𠮷😀"]))).unwrap();
    assert_eq!(document.units[0].text, "中文𠮷😀");
}

#[test]
fn explicit_to_unicode_takes_precedence_over_named_cmap() {
    let mut document = fixture("UniGB-UCS2-H", &["中"]);
    set_to_unicode(&mut document, TO_UNICODE);
    let parsed = parse(&bytes(document)).unwrap();
    assert_eq!(parsed.units[0].text, "甲乙");
}

#[test]
fn invalid_to_unicode_does_not_silently_fall_back() {
    let mut document = fixture("UniGB-UCS2-H", &["中"]);
    set_to_unicode(&mut document, b"not a CMap");
    let error = parse(&bytes(document)).unwrap_err();
    assert_eq!(error.code, "parse_failed");
    assert!(error.message.contains("ToUnicode"));
}

#[test]
fn unsupported_named_cmap_reports_its_name_without_panicking() {
    let error = parse(&bytes(fixture("GBK-EUC-H", &["中"]))).unwrap_err();
    assert_eq!(error.code, "parse_failed");
    assert!(error
        .message
        .contains("unsupported PDF font encoding: GBK-EUC-H"));
}

#[test]
fn other_pages_keep_the_existing_layout_extractor() {
    let mut document = fixture("UniGB-UCS2-H", &["中文", ""]);
    let page = document.get_pages()[&2];
    let content = document
        .get_dictionary(page)
        .unwrap()
        .get(b"Contents")
        .unwrap()
        .as_reference()
        .unwrap();
    document.objects.insert(
        content,
        Stream::new(
            dictionary! {},
            b"BT /F1 12 Tf 30 100 Td (Hello) Tj 0 -20 Td (world) Tj ET".to_vec(),
        )
        .into(),
    );
    document.get_dictionary_mut(page).unwrap().set("Resources", dictionary! {
        "Font" => dictionary! { "F1" => dictionary! {
            "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica", "Encoding" => "WinAnsiEncoding",
        } },
    });
    let mut expected = String::new();
    pdf_extract::output_doc_page(
        &document,
        &mut pdf_extract::PlainTextOutput::new(&mut expected),
        2,
    )
    .unwrap();
    let parsed = parse(&bytes(document)).unwrap();
    assert_eq!(parsed.units[0].text, "中文");
    assert_eq!(parsed.units[1].text, clean_page(&expected));
    assert!(parsed.units[1].text.contains("Hello"));
    assert!(parsed.units[1].text.contains("world"));
}

#[test]
fn form_xobjects_are_not_silently_omitted_on_named_cmap_pages() {
    let mut document = fixture("UniGB-UCS2-H", &["中文"]);
    let page = document.get_pages()[&1];
    let form = document.add_object(Stream::new(dictionary! {
        "Type" => "XObject", "Subtype" => "Form", "BBox" => vec![0.into(), 0.into(), 10.into(), 10.into()],
    }, b"".to_vec()));
    let fonts = page_fonts(&document, page)
        .unwrap()
        .into_iter()
        .map(|(name, font)| (name, Object::Dictionary(font.clone())))
        .collect::<Dictionary>();
    document.get_dictionary_mut(page).unwrap().set(
        "Resources",
        dictionary! {
            "Font" => fonts, "XObject" => dictionary! { "X1" => form },
        },
    );
    document
        .add_to_page_content(
            page,
            Content {
                operations: vec![Operation::new("Do", vec![Object::Name(b"X1".to_vec())])],
            },
        )
        .unwrap();
    let error = parse(&bytes(document)).unwrap_err();
    assert!(error.message.contains("Form XObjects"));
}

#[test]
fn page_errors_are_not_returned_as_successful_truncated_documents() {
    let mut document = fixture("UniGB-UCS2-H", &["中文", "第二页"]);
    let page = document.get_pages()[&2];
    document
        .get_dictionary_mut(page)
        .unwrap()
        .set("Contents", Object::Reference((9999, 0)));
    assert_eq!(parse(&bytes(document)).unwrap_err().code, "parse_failed");
}

#[test]
fn chinese_search_paginates_and_retains_character_offsets() {
    let parsed = Parsed::Document(
        parse(&bytes(fixture("UniGB-UCS2-H", &["甲中文乙中文", "中文"]))).unwrap(),
    );
    let request = SearchRequest {
        query: "中文".into(),
        offset: 0,
        max_results: Some(1),
    };
    let first = query::search(&parsed, "test-p3", &request).unwrap();
    assert_eq!(first["matches"].as_array().unwrap().len(), 1);
    assert_eq!(first["matches"][0]["char_offset"], 1);
    assert_eq!(first["next_offset"], 1);
    assert_eq!(first["truncated"], true);
    let last = query::search(
        &parsed,
        "test-p3",
        &SearchRequest {
            offset: 2,
            ..request
        },
    )
    .unwrap();
    assert_eq!(last["matches"][0]["location"]["page"], 2);
    assert_eq!(last["truncated"], false);
}

#[test]
fn image_xobjects_do_not_prevent_named_cmap_text_extraction() {
    let mut document = fixture("UniGB-UCS2-H", &["中文"]);
    let page = document.get_pages()[&1];
    let image = document.add_object(Stream::new(
        dictionary! {
            "Type" => "XObject", "Subtype" => "Image", "Width" => 1, "Height" => 1,
            "ColorSpace" => "DeviceGray", "BitsPerComponent" => 8,
        },
        vec![0],
    ));
    let fonts = page_fonts(&document, page)
        .unwrap()
        .into_iter()
        .map(|(name, font)| (name, Object::Dictionary(font.clone())))
        .collect::<Dictionary>();
    document.get_dictionary_mut(page).unwrap().set(
        "Resources",
        dictionary! {
            "Font" => fonts, "XObject" => dictionary! { "X1" => image },
        },
    );
    document
        .add_to_page_content(
            page,
            Content {
                operations: vec![Operation::new("Do", vec![Object::Name(b"X1".to_vec())])],
            },
        )
        .unwrap();
    assert_eq!(parse(&bytes(document)).unwrap().units[0].text, "中文");
}

#[test]
fn named_cmap_pages_preserve_other_fonts_and_require_composite_mappings() {
    let mut document = fixture("UniGB-UCS2-H", &["中文"]);
    let page = document.get_pages()[&1];
    let chinese = page_fonts(&document, page).unwrap()[b"F1".as_slice()].clone();
    let mut identity = chinese.clone();
    identity.set("Encoding", "Identity-H");
    let cmap = document.add_object(Stream::new(dictionary! {}, TO_UNICODE.to_vec()));
    identity.set("ToUnicode", cmap);
    let identity = document.add_object(identity);
    document.get_dictionary_mut(page).unwrap().set("Resources", dictionary! {
        "Font" => dictionary! {
            "F1" => chinese,
            "F2" => dictionary! { "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica", "Encoding" => "WinAnsiEncoding" },
            "F3" => identity,
        },
    });
    document
        .add_page_contents(
            page,
            b"BT /F2 12 Tf (ABC) Tj /F3 12 Tf <4E2D> Tj ET".to_vec(),
        )
        .unwrap();
    let parsed = parse(&bytes(document.clone())).unwrap();
    assert!(parsed.units[0].text.contains("中文"));
    assert!(parsed.units[0].text.contains("ABC"));
    assert!(parsed.units[0].text.contains("甲乙"));
    document
        .get_dictionary_mut(identity)
        .unwrap()
        .remove(b"ToUnicode");
    let error = parse(&bytes(document)).unwrap_err();
    assert!(error.message.contains("requiring a ToUnicode map"));
}
