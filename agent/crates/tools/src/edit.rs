use crate::arguments::EditArguments;
use crate::error::fatal;
use crate::scope::{existing_file_path, require_project};
use crate::support::sha256_hex;
use crate::text::{decode, MAX_EDIT_BYTES};
use crate::write;
use serde_json::{json, Value};
use std::io::Read;
use std::path::Path;
use suncode_common::BusinessError;

const MAX_MATCHES: usize = 10_000;
const MAX_CANDIDATES: usize = 8;
const MAX_CHANGES: usize = 50;
const PREVIEW_BYTES: usize = 8 * 1024;

/// Map normalized boundaries back to original decoded bytes. Only CRLF needs
/// an offset adjustment; unchanged bytes never pass through normalization.
struct Source {
    original: String,
    normalized: String,
    removed_cr: Vec<usize>,
    line_starts: Vec<usize>,
}

impl Source {
    fn new(original: String) -> Self {
        let mut normalized = String::with_capacity(original.len());
        let mut removed_cr = Vec::new();
        let mut line_starts = vec![0];
        let mut chars = original.chars().peekable();
        while let Some(character) = chars.next() {
            if character == '\r' {
                normalized.push('\n');
                if chars.peek() == Some(&'\n') {
                    chars.next();
                    removed_cr.push(normalized.len());
                }
                line_starts.push(normalized.len());
            } else {
                normalized.push(character);
                if character == '\n' {
                    line_starts.push(normalized.len());
                }
            }
        }
        Self {
            original,
            normalized,
            removed_cr,
            line_starts,
        }
    }
    fn raw_offset(&self, offset: usize) -> usize {
        offset
            + self
                .removed_cr
                .partition_point(|removed| *removed <= offset)
    }
    fn line(&self, offset: usize) -> usize {
        self.line_starts.partition_point(|start| *start <= offset)
    }
    fn replacement(&self, start: usize, end: usize, new: &str) -> String {
        let original = &self.original[self.raw_offset(start)..self.raw_offset(end)];
        let endings = line_endings(original);
        // Corresponding newlines retain their original style; added lines use
        // the first matched ending, then the nearest preceding/following line.
        let preceding = self
            .line_starts
            .partition_point(|line| *line <= start)
            .saturating_sub(1);
        let fallback = endings.first().copied().unwrap_or_else(|| {
            if preceding > 0 {
                self.ending_at(self.line_starts[preceding] - 1)
            } else if self.line_starts.len() > 1 {
                self.ending_at(self.line_starts[1] - 1)
            } else {
                "\n"
            }
        });
        let normalized = normalize(new);
        let mut output = String::with_capacity(normalized.len());
        for (index, part) in normalized.split('\n').enumerate() {
            if index > 0 {
                output.push_str(endings.get(index - 1).copied().unwrap_or(fallback));
            }
            output.push_str(part);
        }
        output
    }
    fn ending_at(&self, offset: usize) -> &str {
        &self.original[self.raw_offset(offset)..self.raw_offset(offset + 1)]
    }
}

fn normalize(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

fn line_endings(text: &str) -> Vec<&str> {
    let mut endings = Vec::new();
    let bytes = text.as_bytes();
    let mut offset = 0;
    while offset < bytes.len() {
        let start = offset;
        match bytes[offset] {
            b'\r' => {
                offset += 1;
                if bytes.get(offset) == Some(&b'\n') {
                    offset += 1;
                }
                endings.push(&text[start..offset]);
            }
            b'\n' => {
                offset += 1;
                endings.push(&text[start..offset]);
            }
            _ => offset += 1,
        }
    }
    endings
}

struct Replacement {
    start: usize,
    end: usize,
    index: usize,
    new: String,
}

pub(crate) fn edit(
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
    args: &EditArguments,
) -> Result<Value, BusinessError> {
    if args.replacements.is_empty() || args.replacements.len() > 200 {
        return Err(fatal(
            "invalid_arguments",
            "replacements must contain between 1 and 200 entries",
        ));
    }
    let root = require_project(project_root)?;
    let canonical = existing_file_path(root, &args.path)?;
    let mut current = Vec::new();
    std::fs::File::open(canonical)
        .and_then(|file| {
            file.take(MAX_EDIT_BYTES as u64 + 1)
                .read_to_end(&mut current)
        })
        .map_err(|_| fatal("read_failed", "file could not be read"))?;
    if current.len() > MAX_EDIT_BYTES {
        return Err(fatal("resource_limit", "edit source exceeds 16 MiB"));
    }
    let (original, encoding) = decode(&current)?;
    if original.contains('\0') {
        return Err(fatal(
            "encoding_unsupported",
            "edit does not accept text containing NUL",
        ));
    }
    let source = Source::new(original);
    let mut ranges = Vec::<Replacement>::new();
    let mut projected_bytes = encoding.encoded_len(&source.original) as i64;
    let mut staged_text_bytes = 0usize;
    for (index, replacement) in args.replacements.iter().enumerate() {
        if replacement.old.len() > MAX_EDIT_BYTES * 2 || replacement.new.len() > MAX_EDIT_BYTES * 2
        {
            return Err(fatal(
                "resource_limit",
                "replacement text exceeds the edit limit",
            ));
        }
        let old = normalize(&replacement.old);
        if old.is_empty() || replacement.new.contains('\0') {
            return Err(fatal(
                "invalid_arguments",
                "old text must be non-empty and new text must not contain NUL",
            )
            .with_details(json!({"edit_index": index})));
        }
        let mut matches = Vec::new();
        let mut count = 0usize;
        let mut search_from = 0;
        let mut candidates = Vec::new();
        while let Some(relative) = source.normalized[search_from..].find(&old) {
            let start = search_from + relative;
            count += 1;
            if candidates.len() < MAX_CANDIDATES {
                candidates.push(source.line(start));
            }
            if count > MAX_MATCHES {
                return Err(
                    fatal("resource_limit", "too many matches; narrow the target text")
                        .with_details(json!({"edit_index": index, "match_count_at_least": count})),
                );
            }
            matches.push(start);
            // Count overlapping matches for uniqueness, non-overlapping for replaceAll.
            search_from = start
                + if replacement.replace_all {
                    old.len()
                } else {
                    old.chars().next().unwrap().len_utf8()
                };
        }
        if count == 0 || (!replacement.replace_all && count > 1) {
            let reason = if count == 0 { "not_found" } else { "ambiguous" };
            let hint = if count == 0 {
                "Read the current file and copy the exact old text, including indentation."
            } else {
                "Include surrounding context to select one match, or use replaceAll in a single replacement call."
            };
            return Err(fatal("edit_conflict", format!("edit {index}: {reason}. {hint}"))
                .with_details(json!({"reason": reason, "edit_index": index, "match_count": count, "candidate_lines": candidates,
                    "candidates_truncated": count > MAX_CANDIDATES, "suggestion": hint})));
        }
        for start in matches {
            if ranges.len() >= MAX_MATCHES {
                return Err(fatal("resource_limit", "too many replacement ranges"));
            }
            let end = start + old.len();
            let new = source.replacement(start, end, &replacement.new);
            staged_text_bytes += new.len();
            if staged_text_bytes > MAX_EDIT_BYTES * 2 {
                return Err(fatal(
                    "resource_limit",
                    "combined replacement text exceeds 32 MiB",
                ));
            }
            let old_raw = &source.original[source.raw_offset(start)..source.raw_offset(end)];
            projected_bytes +=
                encoding.encoded_len(&new) as i64 - encoding.encoded_len(old_raw) as i64;
            ranges.push(Replacement {
                start,
                end,
                index,
                new,
            });
        }
    }
    ranges.sort_by_key(|range| range.start);
    for pair in ranges.windows(2) {
        if pair[0].end > pair[1].start {
            return Err(fatal("edit_conflict", "replacement ranges overlap; combine them into one edit")
                .with_details(json!({"reason": "overlap", "edit_indexes": [pair[0].index, pair[1].index],
                    "candidate_lines": [source.line(pair[0].start), source.line(pair[1].start)],
                    "suggestion": "Combine overlapping changes into one old/new replacement against the original file."})));
        }
    }
    if projected_bytes > MAX_EDIT_BYTES as i64 {
        return Err(fatal("output_limit", "edited file exceeds 16 MiB"));
    }
    let mut output = String::new();
    let mut cursor = 0;
    let mut changes = Vec::new();
    let mut budget = PREVIEW_BYTES;
    let mut line_delta = 0isize;
    let mut changed_count = 0;
    for range in &ranges {
        let start = source.raw_offset(range.start);
        let end = source.raw_offset(range.end);
        output.push_str(&source.original[cursor..start]);
        output.push_str(&range.new);
        let old = &source.original[start..end];
        let old_breaks = source.line(range.end) - source.line(range.start);
        let new_breaks = line_endings(&range.new).len();
        if old != range.new {
            changed_count += 1;
            if changes.len() < MAX_CHANGES {
                let old_preview = preview(old, &mut budget);
                let new_preview = preview(&range.new, &mut budget);
                let new_start = source.line(range.start).saturating_add_signed(line_delta);
                changes.push(json!({"edit_index": range.index,
                    "old_start_line": source.line(range.start), "old_end_line": source.line(range.end),
                    "new_start_line": new_start, "new_end_line": new_start + new_breaks,
                    "old_text": old_preview, "new_text": new_preview,
                    "preview_truncated": old_preview.len() < old.len() || new_preview.len() < range.new.len()}));
            }
        }
        line_delta += new_breaks as isize - old_breaks as isize;
        cursor = end;
    }
    output.push_str(&source.original[cursor..]);
    if encoding.encoded_len(&output) > MAX_EDIT_BYTES {
        return Err(fatal("output_limit", "edited file exceeds 16 MiB"));
    }
    let baseline = sha256_hex(&current);
    let mut result = write::write_bytes(
        project_root,
        checkpoint_root,
        &args.path,
        &encoding.encode(&output),
        Some(&baseline),
        MAX_EDIT_BYTES,
    )?;
    result["operation"] = json!("edit");
    result["source_sha256"] = json!(baseline);
    result["encoding"] = json!(encoding.name());
    result["replacement_count"] = json!(ranges.len());
    result["changed_count"] = json!(changed_count);
    result["changes_truncated"] = json!(changes.len() < changed_count);
    result["changes"] = json!(changes);
    Ok(result)
}

fn preview<'a>(text: &'a str, budget: &mut usize) -> &'a str {
    let mut end = text.len().min(*budget).min(1024);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    *budget -= end;
    &text[..end]
}

#[cfg(test)]
mod tests;
