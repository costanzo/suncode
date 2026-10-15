//! Filesystem traversal shared by discovery and document loading.

use crate::model::SKILL_FILE_NAME;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Directory entries in a stable order; unreadable directories are empty.
pub(crate) fn sorted_entries(directory: &Path) -> Vec<PathBuf> {
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

pub(crate) fn is_skill_file(path: &Path) -> bool {
    path.file_name().and_then(|name| name.to_str()) == Some(SKILL_FILE_NAME)
}

/// Every `SKILL.md` under `directory`, depth-first in sorted order.
pub(crate) fn skill_files(directory: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    collect_skill_files(directory, &mut result);
    result
}

fn collect_skill_files(directory: &Path, result: &mut Vec<PathBuf>) {
    for path in sorted_entries(directory) {
        if path.is_dir() {
            collect_skill_files(&path, result);
        } else if is_skill_file(&path) {
            result.push(path);
        }
    }
}

/// Non-`SKILL.md` files under `root` as forward-slash relative paths.
pub(crate) fn resource_files(root: &Path) -> Vec<String> {
    let mut result = Vec::new();
    collect_resources(root, root, &mut result);
    result
}

fn collect_resources(root: &Path, directory: &Path, result: &mut Vec<String>) {
    for path in sorted_entries(directory) {
        if is_skill_file(&path) {
            continue;
        }
        if path.is_dir() {
            collect_resources(root, &path, result);
        } else if path.is_file() {
            if let Ok(relative) = path.strip_prefix(root) {
                result.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
}
