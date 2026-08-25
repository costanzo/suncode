use super::arguments::{EditArguments, WriteArguments};
use super::{existing_file, require_project, write, BusinessError};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::path::Path;

fn verify_expected(current: &[u8], args: &EditArguments) -> Result<(), BusinessError> {
    let expected = args.expected_base64.as_str();
    let bytes = STANDARD.decode(expected).map_err(|_| {
        BusinessError::new("invalid_arguments", "expected_base64 is invalid").with_retryable(false)
    })?;
    if current != bytes {
        return Err(
            BusinessError::new("conflict", "file changed since it was read").with_retryable(false),
        );
    }
    Ok(())
}

pub(super) fn edit(
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
    args: &EditArguments,
) -> Result<Value, BusinessError> {
    let root = require_project(project_root)?;
    let path = args.path.as_str();
    let (_, current) = existing_file(root, path)?;
    verify_expected(&current, args)?;
    let raw_text = String::from_utf8(current.clone()).map_err(|_| {
        BusinessError::new("encoding_unsupported", "edit requires UTF-8 text").with_retryable(false)
    })?;
    let has_bom = raw_text.starts_with('\u{feff}');
    let text_without_bom = raw_text.strip_prefix('\u{feff}').unwrap_or(&raw_text);
    let line_ending = if text_without_bom.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let normalized = text_without_bom.replace("\r\n", "\n").replace('\r', "\n");
    let replacements = &args.replacements;
    if replacements.len() > 200 {
        return Err(
            BusinessError::new("resource_limit", "too many replacements").with_retryable(false),
        );
    }
    let mut requested = Vec::with_capacity(replacements.len());
    for replacement in replacements {
        let old = replacement.old.as_str();
        let new = replacement.new.as_str();
        if old.is_empty() {
            return Err(BusinessError::new(
                "invalid_arguments",
                "replacement old text cannot be empty",
            )
            .with_retryable(false));
        }
        let replace_all = replacement.replace_all;
        requested.push((
            old.replace("\r\n", "\n").replace('\r', "\n"),
            new.replace("\r\n", "\n").replace('\r', "\n"),
            replace_all,
        ));
    }
    let mut ranges = Vec::<(usize, usize, String)>::new();
    for (old, new, replace_all) in requested {
        let mut search_from = 0;
        let mut found = 0;
        while let Some(relative) = normalized[search_from..].find(&old) {
            let start = search_from + relative;
            let end = start + old.len();
            ranges.push((start, end, new.clone()));
            found += 1;
            search_from = end;
            if !replace_all {
                break;
            }
        }
        if found == 0 {
            return Err(
                BusinessError::new("edit_conflict", "replacement text was not found")
                    .with_retryable(false),
            );
        }
    }
    ranges.sort_by_key(|(start, _, _)| *start);
    for pair in ranges.windows(2) {
        if pair[0].1 > pair[1].0 {
            return Err(
                BusinessError::new("edit_conflict", "replacement ranges overlap")
                    .with_retryable(false),
            );
        }
    }
    let mut text = normalized;
    for (start, end, new) in ranges.into_iter().rev() {
        text.replace_range(start..end, &new);
    }
    let mut final_text = String::new();
    if has_bom {
        final_text.push('\u{feff}');
    }
    if line_ending == "\r\n" {
        final_text.push_str(&text.replace('\n', "\r\n"));
    } else {
        final_text.push_str(&text);
    }
    let write_args = WriteArguments {
        path: path.to_string(),
        content_base64: STANDARD.encode(final_text.as_bytes()),
        expected_base64: Some(STANDARD.encode(current)),
        idempotency_key: args.idempotency_key.clone(),
        operation_id: args.operation_id.clone(),
    };
    let mut result = write::write(project_root, checkpoint_root, &write_args)?;
    result["operation"] = json!("edit");
    Ok(result)
}
