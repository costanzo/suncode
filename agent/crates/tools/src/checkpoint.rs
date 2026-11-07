//! Content-addressed file versions behind per-turn diffs.
//!
//! `write` stores the bytes a file had before and after each mutation under
//! their SHA-256, gzip-compressed, in one folder per UTC day:
//! `<root>/YYYY-MM-DD/<sha256>.gz`. The digest is of the uncompressed bytes.
//! The data layer keeps, per turn and path, the first-touch baseline and the
//! latest version with the day was stored; diffs are rendered from those
//! two blobs on demand, so repeated writes to one file collapse into one net
//! change. Retention removes whole day folders.

use crate::error::{fatal, retryable};
use crate::support::sha256_hex;
use chrono::{NaiveDate, Utc};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU64;
use std::time::Duration;
use suncode_common::BusinessError;

const DAY_FORMAT: &str = "%Y-%m-%d";

const DIFF_TIMEOUT: Duration = Duration::from_millis(100);
const CONTEXT_LINES: usize = 3;
static  PARTIAL_SEQUENCE: AtomicU64 = AtomicU64::new(1);

/// Net change of one file between its turn baseline and its latest version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDiff {
    pub binary: bool,
    pub additions: u64,
    pub deletions: u64,
    pub hunks: Vec<DiffHunk>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffLine {
    /// `context`, `addition`, or `deletion`
    pub kind: String,
    pub text: String,
    pub old_line: Option<usize>,
    pub new_line: Option<usize>,
}

/// The UTC day new file versions are stored under.
pub(crate) fn today() -> String {
    Utc::now().format(DAY_FORMAT).to_string()
}

fn parse_day(day: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(day, DAY_FORMAT)
        .ok()
        .filter(|date| date.format(DAY_FORMAT).to_string() == day)
}

fn blob_path(checkpoint_root: &Path, day: &str, sha256: &str) -> Result<PathBuf, BusinessError> {
    if parse_day(day).is_none() {
        return Err(fatal("invalid_arguments", "day is invalid"));
    }
    if sha256.len() != 64 || !sha256.bytes().all(|value| value.is_ascii_hexdigit()) {
        return Err(fatal("invalid_arguments", "sha256 is invalid"));
    }
    Ok(checkpoint_root
        .join(day)
        .join(format!("{}.gz", sha256.to_ascii_lowercase())))
}

/// Stores `bytes` compressed, once per day under their digest, and returns the digest.
pub(crate) fn store(
    checkpoint_root: &Path,
    day: &str,
    bytes: &[u8],
) -> Result<String, BusinessError> {
    let sha256 = sha256_hex(bytes);
    let path = blob_path(checkpoint_root, day, &sha256)?;
    if path.is_file() {
        return Ok(sha256);
    }
    let directory = checkpoint_root.join(day);
    fs::create_dir_all(&directory).map_err(|_| {
        retryable(
            "checkpoint_failed",
            "checkpoint directory could not be created",
        )
    })?;
    let partial = directory.join(format!(
        "{sha256}.{}.{}.partial",
        std::process::id(),
        PARTIAL_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    fs::File::create(&partial)
        .and_then(|file| {
            let mut encoder = GzEncoder::new(file, Compression::default());
            encoder.write_all(bytes)?;
            encoder.finish().map(drop)
        })
        .and_then(|()| fs::rename(&partial, &path))
        .map_err(|_| {
            let _ = fs::remove_file(&partial);
            retryable("checkpoint_failed", "checkpoint could not be written")
        })?;
    Ok(sha256)
}

pub(crate) fn load(
    checkpoint_root: &Path,
    day: &str,
    sha256: &str,
) -> Result<Vec<u8>, BusinessError> {
    let file = fs::File::open(blob_path(checkpoint_root, day, sha256)?)
        .map_err(|_| fatal("checkpoint_unavailable", "checkpoint is unavailable"))?;
    let mut bytes = Vec::new();
    GzDecoder::new(file)
        .read_to_end(&mut bytes)
        .map_err(|_| fatal("checkpoint_unavailable", "checkpoint is corrupt"))?;
    Ok(bytes)
}

/// Removes every day folder older than `oldest_kept_day`.
pub(crate) fn retain(checkpoint_root: &Path, oldest_kept_day: &str) {
    let (Some(oldest), Ok(entries)) = (parse_day(oldest_kept_day), fs::read_dir(checkpoint_root))
    else {
        return;
    };
    for entry in entries.flatten() {
        let expired =
            parse_day(&entry.file_name().to_string_lossy().as_ref()).is_some_and(|day| day < oldest);
        if expired && entry.file_type().is_ok_and(|value| value.is_dir()) {
            let _ = fs::remove_dir_all(entry.path());
        }
    }
}

/// Removes individual stored versions, given as `(day, sha256)`.
pub(crate) fn remove<'a>(
    checkpoint_root: &Path,
    versions: impl IntoIterator<Item = (&'a str, &'a str)>,
) {
    for (day, sha256) in versions {
        if let Ok(path) = blob_path(checkpoint_root, day, sha256) {
            let _ = fs::remove_file(path);
        }
    }
}

fn text(bytes: &[u8]) -> Option<&str> {
    std::str::from_utf8(bytes)
        .ok()
        .filter(|value| !value.contains('\0'))
}

/// Line diff from `baseline` (absent for a created file) to `current` (absent
/// for a deleted file).
pub fn diff(baseline: Option<&[u8]>, current: Option<&[u8]>) -> FileDiff {
    let (Some(old), Some(new)) = (
        text(baseline.unwrap_or_default()),
        text(current.unwrap_or_default()),
    ) else {
        return FileDiff {
            binary: true,
            additions: 0,
            deletions: 0,
            hunks: Vec::new(),
        };
    };
    let diff = TextDiff::configure()
        .timeout(DIFF_TIMEOUT)
        .diff_lines(old, new);
    let mut additions = 0;
    let mut deletions = 0;
    let mut hunks = Vec::new();
    for group in diff.grouped_ops(CONTEXT_LINES) {
        let (Some(first), Some(last)) = (group.first(), group.last()) else {
            continue;
        };
        let old_range = first.old_range().start..last.old_range().end;
        let new_range = first.new_range().start..last.new_range().end;
        let mut lines = Vec::new();
        for op in &group {
            for change in diff.iter_changes(op) {
                let kind = match change.tag() {
                    ChangeTag::Equal => "context",
                    ChangeTag::Insert => {
                        additions += 1;
                        "addition"
                    }
                    ChangeTag::Delete => {
                        deletions += 1;
                        "deletion"
                    }
                };
                lines.push(DiffLine {
                    kind: kind.to_string(),
                    text: change
                        .value()
                        .trim_end_matches('\n')
                        .trim_end_matches('\r')
                        .to_string(),
                    old_line: change.old_index().map(|index| index + 1),
                    new_line: change.new_index().map(|index| index + 1),
                });
            }
        }
        hunks.push(DiffHunk {
            old_start: hunk_start(&old_range),
            old_lines: old_range.len(),
            new_start: hunk_start(&new_range),
            new_lines: new_range.len(),
            lines,
        });
    }
    FileDiff {
        binary: false,
        additions,
        deletions,
        hunks,
    }
}

/// Unified-diff start line: one-based, or zero for an empty range.
fn hunk_start(range: &std::ops::Range<usize>) -> usize {
    if range.is_empty() {
        range.start
    } else {
        range.start + 1
    }
}

/// Project-relative path in `/`-separated form, used to key one file per turn.
pub(crate) fn change_path(relative: &Path) -> String {
    relative
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(value) => Some(value.to_string_lossy()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}
