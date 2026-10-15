//! Project dependency path aliasing and scoping.

use super::super::*;

pub(in crate::agent) fn scoped_glob(
    scope_root: Option<&Path>,
    path: &str,
    pattern: &str,
) -> String {
    let base = path.trim_matches('/');
    if base.is_empty() || base == "." {
        return pattern.to_string();
    }
    let target_is_file = scope_root
        .map(|root| root.join(base).is_file())
        .unwrap_or(false);
    if target_is_file {
        return base.to_string();
    }
    let pattern = pattern.trim_start_matches('/');
    // The base is a literal directory; brackets in names such as `app/[id]`
    // must not become glob character classes.
    let base = escape_glob(base);
    if pattern == "**/*" || pattern.starts_with("**/") {
        format!("{base}/{pattern}")
    } else {
        format!("{base}/**/{pattern}")
    }
}

/// Escapes globset metacharacters so `value` matches only itself.
fn escape_glob(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        if matches!(character, '*' | '?' | '[' | ']' | '{' | '}') {
            escaped.push('[');
            escaped.push(character);
            escaped.push(']');
        } else {
            escaped.push(character);
        }
    }
    escaped
}

pub(in crate::agent) fn dependency_path(path: &str) -> Option<(&str, &str)> {
    let value = path.strip_prefix("dependency:")?;
    let (dependency_id, relative_path) = value.split_once('/').unwrap_or((value, "."));
    if dependency_id.is_empty() {
        return None;
    }
    Some((dependency_id, relative_path))
}

pub(in crate::agent) fn dependency_tool_allowed(name: &str) -> bool {
    matches!(name, "read" | "glob" | "grep") || lsp::is_lsp_tool(name)
}

pub(in crate::agent) fn dependency_alias(dependency_id: &str, relative: &str) -> String {
    let relative = relative.trim_start_matches('/');
    if relative.is_empty() || relative == "." {
        format!("dependency:{dependency_id}")
    } else {
        format!("dependency:{dependency_id}/{relative}")
    }
}

#[cfg(test)]
mod tests {
    use super::scoped_glob;

    #[test]
    fn scoped_glob_escapes_literal_directory_names() {
        assert_eq!(
            scoped_glob(None, "app/[id]", "*.tsx"),
            "app/[[]id[]]/**/*.tsx"
        );
        assert_eq!(scoped_glob(None, "src", "**/*.rs"), "src/**/*.rs");
    }
}
