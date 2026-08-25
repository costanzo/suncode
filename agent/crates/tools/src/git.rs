use super::arguments::GitDiffArguments;
use super::{require_project, safe_relative_path, CoreFailure};
use git2::{
    Delta, Diff, DiffFindOptions, DiffLineType, DiffOptions, ErrorCode, Patch, Repository, Status,
    StatusOptions, StatusShow, Tree,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const MAX_STATUS_FILES: usize = 2_000;
const MAX_DIFF_FILE_BYTES: i64 = 2 * 1024 * 1024;
const MAX_DIFF_LINES: usize = 12_000;
const MAX_DIFF_HUNKS: usize = 2_000;

struct RepositoryContext {
    repository: Repository,
    project_prefix: PathBuf,
}

#[derive(Clone, Default)]
struct FileStats {
    additions: usize,
    deletions: usize,
    binary: bool,
}

pub(super) fn status(project_root: Option<&Path>) -> Result<Value, CoreFailure> {
    let root = require_project(project_root)?;
    let context = open_repository(root)?;
    let mut options = StatusOptions::new();
    options
        .show(StatusShow::IndexAndWorkdir)
        .include_untracked(true)
        .recurse_untracked_dirs(true)
        .renames_head_to_index(true)
        .renames_index_to_workdir(true)
        .renames_from_rewrites(true);
    let statuses = context
        .repository
        .statuses(Some(&mut options))
        .map_err(git_read_failure)?;

    let mut combined = build_diff(&context.repository, "all")?;
    detect_renames(&mut combined)?;
    let stats = collect_file_stats(&combined, &context.project_prefix)?;
    let mut files = Vec::new();
    let mut unsupported_paths = 0usize;

    for entry in statuses.iter() {
        let flags = entry.status();
        let (repo_path, old_repo_path) = status_paths(&entry);
        let Some(repo_path) = repo_path else {
            unsupported_paths += 1;
            continue;
        };
        let Some(path) = project_relative(&repo_path, &context.project_prefix) else {
            continue;
        };
        let old_path = (flags.contains(Status::INDEX_RENAMED)
            || flags.contains(Status::WT_RENAMED))
        .then(|| {
            old_repo_path
                .as_deref()
                .and_then(|value| project_relative(value, &context.project_prefix))
        })
        .flatten();
        let file_stats = stats.get(&path).cloned().unwrap_or_default();
        files.push(json!({
            "path": path,
            "old_path": old_path,
            "status": overall_status(flags),
            "index_status": index_status(flags),
            "worktree_status": worktree_status(flags),
            "staged": has_index_change(flags),
            "unstaged": has_worktree_change(flags),
            "conflicted": flags.contains(Status::CONFLICTED),
            "binary": file_stats.binary,
            "additions": file_stats.additions,
            "deletions": file_stats.deletions
        }));
    }

    files.sort_by(|left, right| {
        let left_conflicted = left["conflicted"].as_bool().unwrap_or(false);
        let right_conflicted = right["conflicted"].as_bool().unwrap_or(false);
        right_conflicted.cmp(&left_conflicted).then_with(|| {
            left["path"]
                .as_str()
                .unwrap_or_default()
                .cmp(right["path"].as_str().unwrap_or_default())
        })
    });
    let changed_files = files.len();
    let additions = files
        .iter()
        .filter_map(|file| file["additions"].as_u64())
        .sum::<u64>();
    let deletions = files
        .iter()
        .filter_map(|file| file["deletions"].as_u64())
        .sum::<u64>();
    let conflicts = files
        .iter()
        .filter(|file| file["conflicted"].as_bool().unwrap_or(false))
        .count();
    let truncated = files.len() > MAX_STATUS_FILES;
    files.truncate(MAX_STATUS_FILES);

    let (branch, detached, head_oid) = head_summary(&context.repository);
    Ok(json!({
        "repository": true,
        "branch": branch,
        "detached": detached,
        "head_oid": head_oid,
        "changed_files": changed_files,
        "additions": additions,
        "deletions": deletions,
        "conflicts": conflicts,
        "files": files,
        "truncated": truncated,
        "unsupported_paths": unsupported_paths
    }))
}

pub(super) fn diff_file(
    project_root: Option<&Path>,
    args: &GitDiffArguments,
) -> Result<Value, CoreFailure> {
    let root = require_project(project_root)?;
    let requested = args.path.as_str();
    let requested = safe_relative_path(requested)?;
    let requested = path_string(&requested).ok_or(CoreFailure {
        code: "unsupported_path_encoding",
        message: "path is not valid UTF-8",
        retryable: false,
    })?;
    let scope = args.scope.as_str();
    if !matches!(scope, "all" | "staged" | "unstaged") {
        return Err(CoreFailure {
            code: "invalid_arguments",
            message: "scope must be all, staged, or unstaged",
            retryable: false,
        });
    }

    let context = open_repository(root)?;
    let repo_path = context.project_prefix.join(&requested);
    let mut diff = build_diff(&context.repository, scope)?;
    detect_renames(&mut diff)?;
    let delta_index = diff
        .deltas()
        .enumerate()
        .find_map(|(index, delta)| {
            let old_matches = delta.old_file().path() == Some(repo_path.as_path());
            let new_matches = delta.new_file().path() == Some(repo_path.as_path());
            (old_matches || new_matches).then_some(index)
        })
        .ok_or(CoreFailure {
            code: "git_diff_not_found",
            message: "the selected file has no diff in this scope",
            retryable: false,
        })?;
    let delta = diff.get_delta(delta_index).ok_or(CoreFailure {
        code: "git_diff_not_found",
        message: "the selected file has no diff in this scope",
        retryable: false,
    })?;
    let path = delta
        .new_file()
        .path()
        .or_else(|| delta.old_file().path())
        .and_then(|value| project_relative(value, &context.project_prefix))
        .unwrap_or_else(|| requested.clone());
    let old_path = delta
        .old_file()
        .path()
        .and_then(|value| project_relative(value, &context.project_prefix));
    let binary = delta.old_file().is_binary() || delta.new_file().is_binary();
    let mut additions = 0usize;
    let mut deletions = 0usize;
    let mut hunks = Vec::new();
    let mut patch_text = String::new();
    let mut truncated = false;
    let mut line_count = 0usize;

    if let Some(mut patch) = Patch::from_diff(&diff, delta_index).map_err(git_read_failure)? {
        let (_, patch_additions, patch_deletions) = patch.line_stats().map_err(git_read_failure)?;
        additions = patch_additions;
        deletions = patch_deletions;
        for hunk_index in 0..patch.num_hunks() {
            if hunks.len() >= MAX_DIFF_HUNKS || line_count >= MAX_DIFF_LINES {
                truncated = true;
                break;
            }
            let (hunk, hunk_lines) = patch.hunk(hunk_index).map_err(git_read_failure)?;
            let mut lines = Vec::new();
            for line_index in 0..hunk_lines {
                if line_count >= MAX_DIFF_LINES {
                    truncated = true;
                    break;
                }
                let line = patch
                    .line_in_hunk(hunk_index, line_index)
                    .map_err(git_read_failure)?;
                let kind = match line.origin_value() {
                    DiffLineType::Addition | DiffLineType::AddEOFNL => "addition",
                    DiffLineType::Deletion | DiffLineType::DeleteEOFNL => "deletion",
                    DiffLineType::Context | DiffLineType::ContextEOFNL => "context",
                    _ => "meta",
                };
                let content = String::from_utf8_lossy(line.content())
                    .trim_end_matches(['\r', '\n'])
                    .to_string();
                lines.push(json!({
                    "kind": kind,
                    "old_line": line.old_lineno(),
                    "new_line": line.new_lineno(),
                    "text": content
                }));
                line_count += 1;
            }
            hunks.push(json!({
                "header": String::from_utf8_lossy(hunk.header()).trim().to_string(),
                "old_start": hunk.old_start(),
                "old_lines": hunk.old_lines(),
                "new_start": hunk.new_start(),
                "new_lines": hunk.new_lines(),
                "lines": lines
            }));
        }
        let patch_buffer = patch.to_buf().map_err(git_read_failure)?;
        patch_text = String::from_utf8_lossy(patch_buffer.as_ref()).to_string();
        if patch_text.len() > MAX_DIFF_FILE_BYTES as usize {
            patch_text.truncate(MAX_DIFF_FILE_BYTES as usize);
            truncated = true;
        }
    }

    Ok(json!({
        "scope": scope,
        "path": path,
        "old_path": if old_path.as_deref() == Some(path.as_str()) { None::<String> } else { old_path },
        "status": delta_status(delta.status()),
        "binary": binary,
        "additions": additions,
        "deletions": deletions,
        "hunks": hunks,
        "patch": patch_text,
        "truncated": truncated
    }))
}

fn open_repository(project_root: &Path) -> Result<RepositoryContext, CoreFailure> {
    let repository = Repository::discover(project_root).map_err(|error| {
        if matches!(error.code(), ErrorCode::NotFound) {
            CoreFailure {
                code: "not_git_repository",
                message: "project is not inside a Git repository",
                retryable: false,
            }
        } else {
            git_read_failure(error)
        }
    })?;
    let workdir = repository.workdir().ok_or(CoreFailure {
        code: "unsupported_git_repository",
        message: "bare Git repositories are not supported",
        retryable: false,
    })?;
    let workdir = workdir.canonicalize().map_err(|_| CoreFailure {
        code: "git_unavailable",
        message: "Git working directory is unavailable",
        retryable: true,
    })?;
    let canonical_project = project_root.canonicalize().map_err(|_| CoreFailure {
        code: "project_unavailable",
        message: "project root is unavailable",
        retryable: false,
    })?;
    let project_prefix = canonical_project
        .strip_prefix(&workdir)
        .map_err(|_| CoreFailure {
            code: "scope_denied",
            message: "Git working directory does not contain the project",
            retryable: false,
        })?
        .to_path_buf();
    Ok(RepositoryContext {
        repository,
        project_prefix,
    })
}

fn build_diff<'repo>(
    repository: &'repo Repository,
    scope: &str,
) -> Result<Diff<'repo>, CoreFailure> {
    let mut options = DiffOptions::new();
    options
        .include_typechange(true)
        .context_lines(3)
        .interhunk_lines(0)
        .max_size(MAX_DIFF_FILE_BYTES);
    if scope != "staged" {
        options
            .include_untracked(true)
            .recurse_untracked_dirs(true)
            .show_untracked_content(true);
    }
    let head_tree = head_tree(repository)?;
    match scope {
        "all" => repository.diff_tree_to_workdir_with_index(head_tree.as_ref(), Some(&mut options)),
        "staged" => repository.diff_tree_to_index(head_tree.as_ref(), None, Some(&mut options)),
        "unstaged" => repository.diff_index_to_workdir(None, Some(&mut options)),
        _ => unreachable!("scope is validated before build_diff"),
    }
    .map_err(git_read_failure)
}

fn head_tree(repository: &Repository) -> Result<Option<Tree<'_>>, CoreFailure> {
    match repository.head().and_then(|head| head.peel_to_tree()) {
        Ok(tree) => Ok(Some(tree)),
        Err(error) if matches!(error.code(), ErrorCode::UnbornBranch | ErrorCode::NotFound) => {
            Ok(None)
        }
        Err(error) => Err(git_read_failure(error)),
    }
}

fn detect_renames(diff: &mut Diff<'_>) -> Result<(), CoreFailure> {
    let mut options = DiffFindOptions::new();
    options.renames(true).renames_from_rewrites(true);
    diff.find_similar(Some(&mut options))
        .map_err(git_read_failure)
}

fn collect_file_stats(
    diff: &Diff<'_>,
    project_prefix: &Path,
) -> Result<BTreeMap<String, FileStats>, CoreFailure> {
    let mut result = BTreeMap::new();
    for (index, delta) in diff.deltas().enumerate() {
        let Some(repo_path) = delta.new_file().path().or_else(|| delta.old_file().path()) else {
            continue;
        };
        let Some(path) = project_relative(repo_path, project_prefix) else {
            continue;
        };
        let mut stats = FileStats {
            binary: delta.old_file().is_binary() || delta.new_file().is_binary(),
            ..FileStats::default()
        };
        if let Some(patch) = Patch::from_diff(diff, index).map_err(git_read_failure)? {
            let (_, additions, deletions) = patch.line_stats().map_err(git_read_failure)?;
            stats.additions = additions;
            stats.deletions = deletions;
        } else if delta.old_file().size() > 0 || delta.new_file().size() > 0 {
            stats.binary = true;
        }
        result.insert(path, stats);
    }
    Ok(result)
}

fn status_paths(entry: &git2::StatusEntry<'_>) -> (Option<PathBuf>, Option<PathBuf>) {
    let delta = entry.index_to_workdir().or_else(|| entry.head_to_index());
    if let Some(delta) = delta {
        let new_path = delta.new_file().path().map(Path::to_path_buf);
        let old_path = delta.old_file().path().map(Path::to_path_buf);
        return (new_path.or_else(|| old_path.clone()), old_path);
    }
    let path = std::str::from_utf8(entry.path_bytes())
        .ok()
        .map(PathBuf::from);
    (path, None)
}

fn project_relative(repo_path: &Path, project_prefix: &Path) -> Option<String> {
    let relative = if project_prefix.as_os_str().is_empty() {
        repo_path
    } else {
        repo_path.strip_prefix(project_prefix).ok()?
    };
    if relative.as_os_str().is_empty() {
        return None;
    }
    path_string(relative)
}

fn path_string(path: &Path) -> Option<String> {
    path.to_str()
        .map(|value| value.replace(std::path::MAIN_SEPARATOR, "/"))
}

fn has_index_change(status: Status) -> bool {
    status.intersects(
        Status::INDEX_NEW
            | Status::INDEX_MODIFIED
            | Status::INDEX_DELETED
            | Status::INDEX_RENAMED
            | Status::INDEX_TYPECHANGE,
    )
}

fn has_worktree_change(status: Status) -> bool {
    status.intersects(
        Status::WT_NEW
            | Status::WT_MODIFIED
            | Status::WT_DELETED
            | Status::WT_RENAMED
            | Status::WT_TYPECHANGE,
    )
}

fn index_status(status: Status) -> Option<&'static str> {
    if status.contains(Status::INDEX_NEW) {
        Some("added")
    } else if status.contains(Status::INDEX_DELETED) {
        Some("deleted")
    } else if status.contains(Status::INDEX_RENAMED) {
        Some("renamed")
    } else if status.contains(Status::INDEX_TYPECHANGE) {
        Some("typechange")
    } else if status.contains(Status::INDEX_MODIFIED) {
        Some("modified")
    } else {
        None
    }
}

fn worktree_status(status: Status) -> Option<&'static str> {
    if status.contains(Status::WT_NEW) {
        Some("untracked")
    } else if status.contains(Status::WT_DELETED) {
        Some("deleted")
    } else if status.contains(Status::WT_RENAMED) {
        Some("renamed")
    } else if status.contains(Status::WT_TYPECHANGE) {
        Some("typechange")
    } else if status.contains(Status::WT_MODIFIED) {
        Some("modified")
    } else {
        None
    }
}

fn overall_status(status: Status) -> &'static str {
    if status.contains(Status::CONFLICTED) {
        "conflicted"
    } else if status.intersects(Status::INDEX_DELETED | Status::WT_DELETED) {
        "deleted"
    } else if status.intersects(Status::INDEX_RENAMED | Status::WT_RENAMED) {
        "renamed"
    } else if status.contains(Status::WT_NEW) {
        "untracked"
    } else if status.contains(Status::INDEX_NEW) {
        "added"
    } else if status.intersects(Status::INDEX_TYPECHANGE | Status::WT_TYPECHANGE) {
        "typechange"
    } else {
        "modified"
    }
}

fn delta_status(status: Delta) -> &'static str {
    match status {
        Delta::Added => "added",
        Delta::Deleted => "deleted",
        Delta::Renamed => "renamed",
        Delta::Copied => "copied",
        Delta::Untracked => "untracked",
        Delta::Typechange => "typechange",
        Delta::Conflicted => "conflicted",
        Delta::Unreadable => "unreadable",
        Delta::Ignored => "ignored",
        Delta::Modified | Delta::Unmodified => "modified",
    }
}

fn head_summary(repository: &Repository) -> (Option<String>, bool, Option<String>) {
    let detached = repository.head_detached().unwrap_or(false);
    match repository.head() {
        Ok(head) => (
            head.shorthand().ok().map(str::to_string),
            detached,
            head.target().map(|oid| oid.to_string()),
        ),
        Err(_) => {
            let branch = repository
                .find_reference("HEAD")
                .ok()
                .and_then(|head| head.symbolic_target().ok().flatten().map(str::to_string))
                .and_then(|target| target.strip_prefix("refs/heads/").map(str::to_string));
            (branch, false, None)
        }
    }
}

fn git_read_failure(error: git2::Error) -> CoreFailure {
    let retryable = matches!(error.code(), ErrorCode::Locked | ErrorCode::Modified);
    CoreFailure {
        code: "git_read_failed",
        message: "Git repository could not be read",
        retryable,
    }
}

#[cfg(test)]
mod tests {
    use super::{diff_file, status};
    use crate::arguments::GitDiffArguments;
    use git2::{Repository, Signature};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn reports_clean_modified_staged_and_untracked_files() {
        let root = repository("status");
        fs::write(root.join("tracked.txt"), "changed\n").unwrap();
        fs::write(root.join("untracked.txt"), "new\n").unwrap();
        let repository = Repository::open(&root).unwrap();
        let mut index = repository.index().unwrap();
        index.add_path(Path::new("tracked.txt")).unwrap();
        index.write().unwrap();
        fs::write(root.join("tracked.txt"), "changed again\n").unwrap();

        let result = status(Some(&root)).unwrap();
        assert_eq!(result["changed_files"], 2);
        assert_eq!(result["additions"], 2);
        let tracked = result["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|file| file["path"] == "tracked.txt")
            .unwrap();
        assert_eq!(tracked["staged"], true);
        assert_eq!(tracked["unstaged"], true);
        assert_eq!(tracked["index_status"], "modified");
        assert_eq!(tracked["worktree_status"], "modified");
        assert!(tracked["old_path"].is_null());
        let untracked = result["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|file| file["path"] == "untracked.txt")
            .unwrap();
        assert_eq!(untracked["status"], "untracked");
        cleanup(root);
    }

    #[test]
    fn returns_structured_diff_for_each_scope() {
        let root = repository("diff");
        let repository = Repository::open(&root).unwrap();
        fs::write(root.join("tracked.txt"), "staged\nline\n").unwrap();
        let mut index = repository.index().unwrap();
        index.add_path(Path::new("tracked.txt")).unwrap();
        index.write().unwrap();
        fs::write(root.join("tracked.txt"), "staged\nworktree\n").unwrap();

        let staged = diff_file(
            Some(&root),
            &GitDiffArguments {
                path: "tracked.txt".into(),
                scope: "staged".into(),
            },
        )
        .unwrap();
        assert_eq!(staged["scope"], "staged");
        assert!(staged["hunks"].as_array().unwrap().len() >= 1);
        let unstaged = diff_file(
            Some(&root),
            &GitDiffArguments {
                path: "tracked.txt".into(),
                scope: "unstaged".into(),
            },
        )
        .unwrap();
        assert_eq!(unstaged["scope"], "unstaged");
        assert_eq!(unstaged["additions"], 1);
        assert_eq!(unstaged["deletions"], 1);
        cleanup(root);
    }

    #[test]
    fn supports_unborn_repositories_and_nested_projects() {
        let root = temporary_path("unborn");
        Repository::init(&root).unwrap();
        fs::create_dir_all(root.join("nested")).unwrap();
        fs::write(root.join("nested/new.txt"), "hello\n").unwrap();
        fs::write(root.join("outside.txt"), "outside\n").unwrap();
        let result = status(Some(&root.join("nested"))).unwrap();
        assert_eq!(result["branch"], "master");
        assert_eq!(result["changed_files"], 1);
        assert_eq!(result["files"][0]["path"], "new.txt");
        cleanup(root);
    }

    #[test]
    fn rejects_non_repositories_and_parent_paths() {
        let root = temporary_path("plain");
        fs::create_dir_all(&root).unwrap();
        assert_eq!(status(Some(&root)).unwrap_err().code, "not_git_repository");
        assert_eq!(
            diff_file(
                Some(&root),
                &GitDiffArguments {
                    path: "../outside".into(),
                    scope: "all".into(),
                },
            )
            .unwrap_err()
            .code,
            "scope_denied"
        );
        cleanup(root);
    }

    fn repository(name: &str) -> PathBuf {
        let root = temporary_path(name);
        let repository = Repository::init(&root).unwrap();
        fs::write(root.join("tracked.txt"), "original\nline\n").unwrap();
        let mut index = repository.index().unwrap();
        index.add_path(Path::new("tracked.txt")).unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repository.find_tree(tree_id).unwrap();
        let signature = Signature::now("SunCode Test", "test@suncode.local").unwrap();
        repository
            .commit(Some("HEAD"), &signature, &signature, "initial", &tree, &[])
            .unwrap();
        drop(tree);
        drop(repository);
        root.canonicalize().unwrap()
    }

    fn temporary_path(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("suncode-git-{name}-{nonce}"))
    }

    fn cleanup(root: PathBuf) {
        fs::remove_dir_all(root).unwrap();
    }
}
