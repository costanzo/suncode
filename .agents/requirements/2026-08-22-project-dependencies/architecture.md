# Architecture

## Current state

One project owns one canonical root. Avalonia receives project/session DTOs through the C ABI and cannot inspect files directly. Rust operations enforce the project boundary.

## Proposed design

Add `project_dependency` as a project-owned registry of canonical roots. Add named SDK/C ABI methods to list, add, remove, and shallow-list project/dependency directories. Present those methods in an Avalonia Explorer that shares the current navigation pane.

## Boundaries and dependencies

- SQLite and canonical roots remain Rust-owned.
- Avalonia owns tree presentation, expansion state, folder picking, and sidebar switching.
- Operations owns canonical path validation and bounded directory reads.
- Agent routing resolves opaque dependency IDs immediately before an allowed read/search operation.

## Data and control flow

1. Avalonia asks Rust to add the selected folder.
2. Rust canonicalizes it, checks project/dependency overlap, and persists the registration.
3. Explorer requests one directory level with project ID, optional dependency ID, and relative path.
4. Rust resolves the root, validates containment, omits symlinks, and returns sorted bounded entries.
5. Agent context lists dependency display names and IDs. `dependency:<id>/...` calls resolve to the stored root; results are rewritten back to the alias.

## Security and failure handling

Dependencies add read authority only. The router accepts their aliases for `read`, `glob`, and `grep`; every other tool fails with `scope_denied`. Absolute roots never enter client/model DTOs or logs. Missing IDs and unavailable paths fail closed.

## Compatibility and migration

There is no migration runner or schema version. Because schema scripts use `CREATE TABLE IF NOT EXISTS` in one transaction before exact-table validation, an otherwise-current 13-table database receives the empty dependency table safely. Unknown tables or incompatible structures still roll back and fail.

## Risks and rollback

- Large trees are bounded and lazy to avoid UI stalls and large payloads.
- Removing a registration is reversible by re-adding the folder and never mutates its files.
- Rolling back the feature after users add dependencies requires an explicit schema decision because the current table manifest would otherwise reject the extra table.

## Open questions

- None.
