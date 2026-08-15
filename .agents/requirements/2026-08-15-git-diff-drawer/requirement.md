# Requirement

## Background

The project window shows agent processes, approvals, checkpoints, and session-touched paths, but it cannot inspect the actual Git working tree. Developers need an application-owned Git view that does not depend on an installed Git executable and that keeps project changes visible without replacing the conversation workflow.

## Goals

- Embed local Git status and diff inspection in the Rust runtime through `git2` and vendored libgit2.
- Show a dedicated docked Git drawer controlled by an icon in the left gutter.
- Show a compact, colored Git change summary in the project footer while preserving model and session-token information.
- Cover working-tree, staged, unstaged, untracked, renamed, deleted, binary, and conflicted states.

## Non-goals

- Stage, unstage, discard, commit, branch mutation, fetch, pull, push, or remote credentials.
- A general-purpose Git client or hosted review workflow.
- Filesystem watching or continuous polling.
- Line comments, review threads, or collaboration.

## Requirements

- Rust operations own repository discovery, status calculation, diff generation, path validation, and output bounds.
- Qt must consume named SDK methods and must not read `.git`, project files, or invoke Git directly.
- The runtime must not require a Git executable or a system libgit2 installation.
- The left project gutter must expose a keyboard-focusable Git toggle below the navigation toggle.
- The Git drawer must open from the bottom as a docked, resizable work surface without covering footer status.
- The drawer must provide all, staged, and unstaged scopes, a changed-file list, and a structured unified diff.
- The footer must show repository state, changed-file count, additions, and deletions using semantic colors and text labels.
- Existing footer model and cumulative session token displays must remain available.
- Diff rows must be virtualized and loaded for the selected file rather than rendering the whole repository patch eagerly.

## Edge cases

- The project is not a Git repository or is inside a repository whose root is above the opened project.
- The repository has no commits yet.
- A file is staged and then modified again.
- Untracked directories, renamed files, deleted files, binary files, type changes, submodules, and merge conflicts.
- Invalid UTF-8 file content or paths.
- A file changes while its diff is being loaded.
- Very large files, very large hunks, and repositories with many changed paths.
- Compact 900x620 windows, fullscreen, light mode, and both side panels expanded.

## Acceptance criteria

- `git2` with vendored libgit2 builds as part of the existing Rust static library.
- Status results distinguish index and worktree states and include untracked files.
- File diff results expose structured hunks and lines with old/new line numbers.
- The Qt project window opens and closes the drawer from the left gutter and footer summary.
- The footer presents clean, dirty, conflicted, loading, non-repository, and error states without overlapping existing content.
- Focused Rust tests, the Qt desktop build, QML validation, and `git diff --check` pass.

## Open questions

- Local Git mutations and remote operations require separate authority and credential designs.
