# Changes

## Source

- Added a bounded `git2` operation module for repository discovery, project-scoped status, and structured per-file diffs.
- Added typed Rust SDK methods and C ABI entry points for Git status and file diffs.
- Added asynchronous Qt Git projections with debounced refresh after project and checkpoint activity.
- Added a left-gutter Git toggle, docked resizable bottom drawer, virtualized changed-file and diff lists, scope filtering, text filtering, patch copy, and complete presentation states.
- Added a colored, clickable footer Git summary while preserving model and cumulative session-token presentation.
- Linked the vendored libgit2 static dependency's native zlib and Apple iconv requirements in the Qt target.

## Contracts and generated artifacts

- Added the Git status and file-diff SDK contract to `contracts/runtime-sdk/README.md`.
- Added shared result vectors to `contracts/vectors/runtime-sdk.json`.

## Configuration and persistence

- No persisted data or schema migration is planned.

## Tests

- Added operation coverage for clean, modified, staged, unstaged, untracked, unborn, nested-project, invalid-path, and non-repository behavior.
- Added typed SDK and C ABI coverage for Git status and file diffs.

## Documentation

- Added this requirement package and updated architecture, Qt/runtime/core feature records, and `ADR-20260815-embedded-git2-review`.
