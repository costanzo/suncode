# Architecture

## Current state

The Qt client receives project, session, checkpoint, and activity DTOs from the embedded Rust SDK. Its `changedPaths` projection describes paths touched by session events and is not a Git working-tree snapshot. The runtime has no VCS-aware operation or SDK method.

## Proposed design

Add a read-only `git` module to `agent/crates/operations` backed by `git2` with vendored libgit2 and no remote transport features. The module returns bounded structured status and per-file diff DTOs. The Rust SDK resolves the project record, calls the operation with its canonical root, and exposes named C ABI methods. The Qt adapter retains only current presentation projections.

Restructure the project-window body as fixed gutters around a vertical work area. The existing navigation/conversation/process row remains the upper work area. A resizable Git drawer occupies the lower work area when open. The footer remains outside the drawer and exposes a compact Git summary.

## Boundaries and dependencies

- `git2` belongs only to the audited operations crate.
- Core owns project lookup and the public SDK DTO boundary.
- Qt owns drawer visibility, selected scope, selected file, filter text, and drawer height.
- The first delivery is read-only and does not modify the worktree, index, refs, configuration, or remotes.

## Data and control flow

1. Project selection or an explicit refresh calls `git_status(project_id)`.
2. Core resolves the canonical project root and calls `git/status` in operations.
3. Qt renders the footer summary and changed-file list.
4. File selection calls `git_diff_file(project_id, scope, path)`.
5. Operations validates the project-relative path and returns structured hunks and lines.
6. Agent checkpoint events, drawer opening, and window activation trigger debounced status refreshes.

## Security and failure handling

Repository discovery may find a Git root above the opened project, but returned paths are filtered to the opened project and remain project-relative. Git metadata is never returned as an absolute path. Diff content is transported as plain text and never interpreted as rich text. Results are bounded by file count, file size, hunk count, line count, and serialized content size.

Read-only Git calls require no approval under the interactive inspection policy. Future index, worktree, ref, or remote mutations must pass through policy, audit, and checkpoint requirements before implementation.

## Compatibility and migration

The C ABI adds methods without changing the ABI major version. No SQLite migration is required. Vendored libgit2 removes the runtime dependency on a system Git installation.

## Risks and rollback

Vendored libgit2 increases build time and binary size. Repository semantics can differ from Git CLI in obscure attribute, filter, and submodule cases, so fixtures cover the supported local review contract rather than promising byte-for-byte CLI parity. The feature can be removed without durable data migration.

## Open questions

- None for the read-only delivery.
