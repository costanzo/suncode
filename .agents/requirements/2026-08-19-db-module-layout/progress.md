# Progress

- Status: Complete
- Last updated: 2026-08-19

## Completed

- Created the standalone `suncode-db` package with `schema` and `data` module boundaries.
- Split the current schema into 16 table-named SQL files and an ordered manifest.
- Moved store-facing DTO ownership into the database package and kept core compatibility re-exports.
- Removed core's direct `rusqlite` dependency and added the database package to the Cargo workspace.
- Removed schema versions, migration metadata, upgrade functions, and upgrade tests.
- Added exact-table, ordering, idempotency, incompatibility, integrity, foreign-key, index, and snapshot-lifecycle coverage.
- Passed 12 standalone database tests, 18 core tests, all 47 Rust workspace tests, formatting, schema validation, and diff checks.
- Updated the current contracts, architecture, feature record, and decision index.

## In progress

- None.

## Blocked

- None.

## Log

### 2026-08-19

- Replaced the earlier v14 upgrade proposal with a current-schema-only design.
- Completed focused and full verification and closed the requirement.
