//! Bounded project-root matching for literal markers and catalog-owned glob patterns.

use globset::Glob;
use ignore::WalkBuilder;
use std::path::Path;

const MAX_WALK_ENTRIES: usize = 20_000;
const MAX_WALK_DEPTH: usize = 16;

pub fn validate_root_marker(marker: &str) -> bool {
    !contains_glob(marker) || Glob::new(marker).is_ok()
}

pub fn match_root_marker(project_root: &Path, markers: &[String]) -> Option<String> {
    for marker in markers.iter().filter(|marker| !contains_glob(marker)) {
        if project_root.join(marker).exists() {
            return Some(marker.clone());
        }
    }

    let patterns = markers
        .iter()
        .filter(|marker| contains_glob(marker))
        .map(|marker| Glob::new(marker).map(|glob| glob.compile_matcher()))
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    if patterns.is_empty() {
        return None;
    }
    let mut best = None;
    for entry in WalkBuilder::new(project_root)
        .hidden(false)
        .max_depth(Some(MAX_WALK_DEPTH))
        .build()
        .take(MAX_WALK_ENTRIES)
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_some_and(|kind| kind.is_file()))
    {
        let Some(relative) = entry.path().strip_prefix(project_root).ok() else {
            continue;
        };
        let Some(file_name) = entry.path().file_name() else {
            continue;
        };
        let Some(pattern_index) = patterns
            .iter()
            .position(|pattern| pattern.is_match(relative) || pattern.is_match(file_name))
        else {
            continue;
        };
        if best
            .as_ref()
            .is_none_or(|(best_index, _)| pattern_index < *best_index)
        {
            best = Some((pattern_index, relative.to_string_lossy().replace('\\', "/")));
            if pattern_index == 0 {
                break;
            }
        }
    }
    best.map(|(_, path)| path)
}

fn contains_glob(value: &str) -> bool {
    value
        .chars()
        .any(|character| matches!(character, '*' | '?' | '[' | ']'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_markers_remain_root_relative() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("nested")).unwrap();
        std::fs::write(root.path().join("nested/package.json"), "{}").unwrap();
        assert_eq!(
            match_root_marker(root.path(), &["package.json".into()]),
            None
        );
        std::fs::write(root.path().join("package.json"), "{}").unwrap();
        assert_eq!(
            match_root_marker(root.path(), &["package.json".into()]),
            Some("package.json".into())
        );
    }

    #[test]
    fn glob_markers_find_nested_project_files() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("src/App")).unwrap();
        std::fs::write(root.path().join("src/App/App.csproj"), "<Project />").unwrap();
        assert_eq!(
            match_root_marker(root.path(), &["*.csproj".into()]),
            Some("src/App/App.csproj".into())
        );
    }

    #[test]
    fn glob_markers_respect_configuration_order() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("src/App")).unwrap();
        std::fs::write(root.path().join("src/SunCode.slnx"), "<Solution />").unwrap();
        std::fs::write(root.path().join("src/App/App.csproj"), "<Project />").unwrap();
        assert_eq!(
            match_root_marker(root.path(), &["*.csproj".into(), "*.slnx".into()]),
            Some("src/App/App.csproj".into())
        );
    }

    #[test]
    fn validates_glob_syntax() {
        assert!(validate_root_marker("*.slnx"));
        assert!(!validate_root_marker("[broken"));
    }
}
