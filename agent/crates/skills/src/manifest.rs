//! `SKILL.md` parsing and metadata validation.

use crate::{
    diagnostic::SkillDiagnostic,
    model::{SkillInfo, SkillSource},
};
use serde::Deserialize;
use std::{fs, path::Path};

pub const MAX_SKILL_BYTES: u64 = 64 * 1024;
pub const MAX_DESCRIPTION_CHARS: usize = 1024;
pub const MAX_NAME_CHARS: usize = 64;

#[derive(Debug, Deserialize, Default)]
struct Frontmatter {
    name: Option<String>,
    description: Option<String>,
    #[serde(rename = "disable-model-invocation")]
    disable_model_invocation: Option<bool>,
}

/// Reads and validates the metadata of the `SKILL.md` at `path`.
pub(crate) fn read_info(path: &Path, source: SkillSource) -> Result<SkillInfo, SkillDiagnostic> {
    let warn = |message: String| SkillDiagnostic::warning(message, Some(path.to_path_buf()));
    let metadata = fs::metadata(path).map_err(|error| warn(error.to_string()))?;
    if metadata.len() > MAX_SKILL_BYTES {
        return Err(warn(format!("skill exceeds {MAX_SKILL_BYTES} bytes")));
    }
    let raw = fs::read_to_string(path).map_err(|error| warn(error.to_string()))?;
    let (frontmatter, _) = split_frontmatter(&raw).map_err(warn)?;
    let parsed: Frontmatter =
        serde_yaml::from_str(frontmatter).map_err(|error| warn(error.to_string()))?;
    let name = parsed.name.unwrap_or_else(|| {
        path.parent()
            .and_then(Path::file_name)
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_string()
    });
    validate_name(&name).map_err(warn)?;
    let description = parsed.description.unwrap_or_default().trim().to_string();
    if description.is_empty() || description.chars().count() > MAX_DESCRIPTION_CHARS {
        return Err(warn(format!(
            "skill description must contain 1 through {MAX_DESCRIPTION_CHARS} characters"
        )));
    }
    let base_directory = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    Ok(SkillInfo {
        name,
        description,
        location: path.to_path_buf(),
        base_directory,
        source,
        explicit_only: parsed.disable_model_invocation.unwrap_or(false),
    })
}

/// Returns the trimmed body of the `SKILL.md` at `path`.
pub(crate) fn read_body(path: &Path) -> Result<String, String> {
    let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let (_, content) = split_frontmatter(&raw)?;
    Ok(content.trim().to_string())
}

/// Splits `raw` into frontmatter and body. Delimiters are lines containing
/// exactly `---`, ignoring trailing whitespace.
fn split_frontmatter(raw: &str) -> Result<(&str, &str), String> {
    let mut lines = raw.split_inclusive('\n');
    let opening = lines.next().unwrap_or_default();
    if opening.trim_end() != "---" {
        return Err("SKILL.md must start with YAML frontmatter".into());
    }
    let start = opening.len();
    let mut offset = start;
    for line in lines {
        if line.trim_end() == "---" {
            return Ok((&raw[start..offset], &raw[offset + line.len()..]));
        }
        offset += line.len();
    }
    Err("frontmatter closing delimiter is missing".into())
}

fn validate_name(name: &str) -> Result<(), String> {
    let valid = !name.is_empty()
        && name.chars().count() <= MAX_NAME_CHARS
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !name.contains("--");
    valid.then_some(()).ok_or_else(|| {
        format!(
            "skill name must match ^[a-z0-9]+(-[a-z0-9]+)*$ and be at most {MAX_NAME_CHARS} characters"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_frontmatter_on_exact_delimiter_lines() {
        let raw = "---\nname: a\ndescription: ---- not a delimiter\n---\nbody\n---- rule\n";
        let (frontmatter, body) = split_frontmatter(raw).unwrap();
        assert_eq!(frontmatter, "name: a\ndescription: ---- not a delimiter\n");
        assert_eq!(body, "body\n---- rule\n");
    }

    #[test]
    fn splits_empty_and_crlf_frontmatter() {
        assert_eq!(split_frontmatter("---\n---\nbody").unwrap(), ("", "body"));
        assert_eq!(
            split_frontmatter("---\r\nname: a\r\n---\r\nbody").unwrap(),
            ("name: a\r\n", "body")
        );
    }

    #[test]
    fn rejects_missing_frontmatter_delimiters() {
        assert!(split_frontmatter("body").is_err());
        assert!(split_frontmatter("---\nname: a\n----\nbody").is_err());
    }
}
