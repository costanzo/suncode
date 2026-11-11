use super::*;
use crate::arguments::{ReadArguments, ReplacementArguments};
use crate::text::Encoding;
use std::fs;

fn run(
    bytes: &[u8],
    replacements: Vec<ReplacementArguments>,
) -> (Result<Value, BusinessError>, Vec<u8>) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let checkpoints = root.join("versions");
    fs::write(root.join("file.txt"), bytes).unwrap();
    let result = edit(
        Some(&root),
        Some(&checkpoints),
        &EditArguments {
            path: "file.txt".into(),
            replacements,
        },
    );
    (result, fs::read(root.join("file.txt")).unwrap())
}
fn replacement(old: &str, new: &str) -> ReplacementArguments {
    ReplacementArguments {
        old: old.into(),
        new: new.into(),
        replace_all: false,
    }
}

#[test]
fn preserves_untouched_mixed_endings_and_local_inserted_endings() {
    let original = b"\xef\xbb\xbfalpha\r\nbeta\ngamma\rdelta";
    let (result, bytes) = run(original, vec![replacement("beta\ngamma", "B\nG\nextra")]);
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(bytes, b"\xef\xbb\xbfalpha\r\nB\nG\nextra\rdelta");
    let (_, bytes) = run(
        b"first\rsecond\rthird",
        vec![replacement("second", "two\nmore")],
    );
    assert_eq!(bytes, b"first\rtwo\rmore\rthird");
    let (_, bytes) = run(b"a\r\nb\nc\r", vec![replacement("a\nb\nc\n", "A\nB\nC\n")]);
    assert_eq!(bytes, b"A\r\nB\nC\r");
}

#[test]
fn reports_unicode_positions_and_uses_original_batch_coordinates() {
    let (result, bytes) = run(
        "é\r\n目标\nend".as_bytes(),
        vec![replacement("é", "é\nadded"), replacement("目标", "替换")],
    );
    assert_eq!(String::from_utf8(bytes).unwrap(), "é\r\nadded\r\n替换\nend");
    let result = result.unwrap();
    assert_eq!(result["replacement_count"], 2);
    assert_eq!(result["changes"][1]["old_start_line"], 2);
    assert_eq!(result["changes"][1]["new_start_line"], 3);
    let (result, bytes) = run(b"A", vec![replacement("A", "B"), replacement("B", "C")]);
    assert_eq!(result.unwrap_err().details["edit_index"], 1);
    assert_eq!(bytes, b"A");
}

#[test]
fn conflict_details_identify_missing_ambiguous_and_overlapping_edits() {
    let (result, _) = run(b"a\na\na\na\na\na\na\na\na\na", vec![replacement("a", "b")]);
    let error = result.unwrap_err();
    assert_eq!(error.code, "edit_conflict");
    assert_eq!(error.details["match_count"], 10);
    assert_eq!(
        error.details["candidate_lines"].as_array().unwrap().len(),
        8
    );
    assert_eq!(error.details["candidates_truncated"], true);
    let (result, _) = run("ééé".as_bytes(), vec![replacement("éé", "b")]);
    assert_eq!(result.unwrap_err().details["match_count"], 2);
    let (result, bytes) = run(b"abc", vec![replacement("ab", "X"), replacement("bc", "Y")]);
    assert_eq!(result.unwrap_err().details["edit_indexes"], json!([0, 1]));
    assert_eq!(bytes, b"abc");
    let (result, _) = run(b"abc", vec![replacement("missing", "Y")]);
    assert_eq!(result.unwrap_err().details["reason"], "not_found");
}

#[test]
fn bounded_previews_and_noop_results_are_explicit() {
    let mut change = replacement("x", &"中".repeat(600));
    change.replace_all = true;
    let (result, _) = run("x\n".repeat(70).as_bytes(), vec![change]);
    let result = result.unwrap();
    assert_eq!(result["replacement_count"], 70);
    assert_eq!(result["changes"].as_array().unwrap().len(), 50);
    assert_eq!(result["changes_truncated"], true);
    let bytes: usize = result["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|change| {
            change["old_text"].as_str().unwrap().len() + change["new_text"].as_str().unwrap().len()
        })
        .sum();
    assert!(bytes <= PREVIEW_BYTES);
    let (result, bytes) = run(b"a\r\nb\nc", vec![replacement("a\nb", "a\nb")]);
    assert_eq!(result.unwrap()["changed_count"], 0);
    assert_eq!(bytes, b"a\r\nb\nc");
}

#[test]
fn utf16le_round_trips_and_read_hashes_stay_raw() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let original = Encoding::Utf16Le.encode("开始\r\n😀 tail\n");
    fs::write(root.join("file.txt"), &original).unwrap();
    let read = crate::read::read(
        Some(&root),
        None,
        &ReadArguments {
            path: "file.txt".into(),
            offset: Some(2),
            limit: Some(1),
            max_bytes: None,
        },
    )
    .unwrap();
    use base64::Engine;
    assert_eq!(
        base64::engine::general_purpose::STANDARD
            .decode(read["data_base64"].as_str().unwrap())
            .unwrap(),
        "😀 tail\n".as_bytes()
    );
    assert_eq!(read["source_sha256"], sha256_hex(&original));
    assert!(read.get("sha256").is_none());
    let (result, bytes) = run(&original, vec![replacement("😀", "中文")]);
    assert_eq!(result.unwrap()["encoding"], "utf-16le");
    assert_eq!(bytes, Encoding::Utf16Le.encode("开始\r\n中文 tail\n"));
    let diff = crate::checkpoint::diff(Some(&original), Some(&bytes));
    assert!(!diff.binary);
    assert_eq!(diff.additions, 1);
    for invalid in [
        vec![0xff, 0xfe, 0x00],
        vec![0xff, 0xfe, 0x00, 0xd8],
        vec![0xff],
        vec![0xfe, 0xff, 0, 65],
        b"a\0b".to_vec(),
    ] {
        let (result, bytes) = run(&invalid, vec![replacement("a", "b")]);
        assert_eq!(result.unwrap_err().code, "encoding_unsupported");
        assert_eq!(bytes, invalid);
    }
}

#[test]
fn larger_files_are_editable_but_sources_and_results_are_bounded() {
    let original = format!("start{}end", "x".repeat(1024 * 1024));
    let (result, bytes) = run(original.as_bytes(), vec![replacement("start", "START")]);
    assert!(result.is_ok(), "{result:?}");
    assert!(bytes.starts_with(b"START"));
    let original = vec![b'x'; MAX_EDIT_BYTES + 1];
    let (result, bytes) = run(&original, vec![replacement("x", "y")]);
    assert_eq!(result.unwrap_err().code, "resource_limit");
    assert_eq!(bytes, original);
    let (result, bytes) = run(
        b"x",
        vec![replacement("x", &"y".repeat(MAX_EDIT_BYTES + 1))],
    );
    assert_eq!(result.unwrap_err().code, "output_limit");
    assert_eq!(bytes, b"x");
    let mut change = replacement("x", "y");
    change.replace_all = true;
    let (result, _) = run(&vec![b'x'; MAX_MATCHES + 1], vec![change]);
    assert_eq!(result.unwrap_err().code, "resource_limit");
}

#[test]
fn large_utf8_line_reads_remain_bounded_and_do_not_claim_a_digest() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let mut bytes = vec![b'x'; MAX_EDIT_BYTES + 1];
    bytes.extend_from_slice(b"\nlast\n");
    fs::write(root.join("file.txt"), bytes).unwrap();
    let arguments = ReadArguments {
        path: "file.txt".into(),
        offset: Some(2),
        limit: Some(1),
        max_bytes: None,
    };
    let read = crate::read::read(Some(&root), None, &arguments).unwrap();
    use base64::Engine;
    assert_eq!(
        base64::engine::general_purpose::STANDARD
            .decode(read["data_base64"].as_str().unwrap())
            .unwrap(),
        b"last\n"
    );
    assert!(read.get("source_sha256").is_none());
    assert!(read.get("sha256").is_none());
    assert_eq!(read["truncated"], false);
    let arguments = ReadArguments {
        offset: Some(1),
        ..arguments
    };
    assert_eq!(
        crate::read::read(Some(&root), None, &arguments)
            .unwrap_err()
            .code,
        "resource_limit"
    );
}
