//! Filesystem traversal shared by discovery and document loading.
//!
//! Symlinked directories are followed so users can link skills into place,
//! but each real directory is visited once and depth is bounded.

use crate::model::SKILL_FILE_NAME;
use std::{
    collections::HashSet,
    fs,
    ops::ControlFlow,
    path::{Path, PathBuf},
};

const MAX_SCAN_DEPTH: usize = 8;

/// Every `SKILL.md` under `directory`, depth-first in sorted order.
pub(crate) fn skill_files(directory: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    walk_files(directory, &mut |path| {
        if is_skill_file(path) {
            result.push(path.to_path_buf());
        }
        ControlFlow::Continue(())
    });
    result
}

/// Up to `limit` non-`SKILL.md` files under `root` as forward-slash relative
/// paths. Traversal stops once the limit is reached.
pub(crate) fn resource_files(root: &Path, limit: usize) -> Vec<String> {
    let mut result = Vec::new();
    if limit == 0 {
        return result;
    }
    walk_files(root, &mut |path| {
        if is_skill_file(path) {
            return ControlFlow::Continue(());
        }
        if let Ok(relative) = path.strip_prefix(root) {
            result.push(relative.to_string_lossy().replace('\\', "/"));
        }
        if result.len() >= limit {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    });
    result
}

fn walk_files(root: &Path, visit: &mut dyn FnMut(&Path) -> ControlFlow<()>) {
    let mut visited = HashSet::new();
    let _ = walk(root, 0, &mut visited, visit);
}

fn walk(
    directory: &Path,
    depth: usize,
    visited: &mut HashSet<PathBuf>,
    visit: &mut dyn FnMut(&Path) -> ControlFlow<()>,
) -> ControlFlow<()> {
    if depth > MAX_SCAN_DEPTH {
        return ControlFlow::Continue(());
    }
    let Ok(canonical) = fs::canonicalize(directory) else {
        return ControlFlow::Continue(());
    };
    if !visited.insert(canonical) {
        return ControlFlow::Continue(());
    }
    for path in sorted_entries(directory) {
        if path.is_dir() {
            walk(&path, depth + 1, visited, visit)?;
        } else if path.is_file() {
            visit(&path)?;
        }
    }
    ControlFlow::Continue(())
}

/// Directory entries in a stable order; unreadable directories are empty.
fn sorted_entries(directory: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn is_skill_file(path: &Path) -> bool {
    path.file_name().and_then(|name| name.to_str()) == Some(SKILL_FILE_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn resource_collection_stops_at_limit() {
        let root = tempdir().unwrap();
        fs::write(root.path().join(SKILL_FILE_NAME), "skill").unwrap();
        for index in 0..20 {
            fs::write(root.path().join(format!("{index:02}.md")), "x").unwrap();
        }
        let files = resource_files(root.path(), 3);
        assert_eq!(files, vec!["00.md", "01.md", "02.md"]);
    }

    #[test]
    fn traversal_depth_is_bounded() {
        let root = tempdir().unwrap();
        let mut directory = root.path().to_path_buf();
        for index in 0..=MAX_SCAN_DEPTH + 1 {
            directory = directory.join(format!("d{index}"));
        }
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join(SKILL_FILE_NAME), "too deep").unwrap();
        assert!(skill_files(root.path()).is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_cycles_terminate_and_linked_skills_are_found() {
        let root = tempdir().unwrap();
        let skill = root.path().join("release");
        fs::create_dir_all(&skill).unwrap();
        fs::write(skill.join(SKILL_FILE_NAME), "skill").unwrap();
        std::os::unix::fs::symlink(root.path(), skill.join("loop")).unwrap();

        let linked = tempdir().unwrap();
        std::os::unix::fs::symlink(&skill, linked.path().join("release")).unwrap();

        assert_eq!(skill_files(root.path()), vec![skill.join(SKILL_FILE_NAME)]);
        assert_eq!(skill_files(linked.path()).len(), 1);
        assert!(resource_files(&skill, 10).is_empty());
    }
}
