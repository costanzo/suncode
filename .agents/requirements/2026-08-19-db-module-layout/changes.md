# Changes

## Source

- Added the standalone `suncode-db` package under `runtime/crates/db` as the database ownership boundary.
- Moved `persistence.rs` to `runtime/crates/db/src/store.rs` and updated core imports.
- Moved persistence DTO ownership into the database package and retained core re-exports for SDK compatibility.
- Removed core's direct `rusqlite` dependency.
- Added ordered `schema` and `data` manifests with table-named schema files.
- Split all 16 current tables into table-owned SQL files.
- Removed schema version and migration behavior.
- Added incompatible-database rejection and current-schema validation.

## Contracts and generated artifacts

- Updated SQLite, persistence, and SDK lifecycle contracts for current-schema initialization.
- No generated contracts or artifacts are used.

## Configuration and persistence

- Removed `schema_migrations`, `client_sync`, and `user_settings` from the current design.
- Made `sessions.project_id` required.
- Added non-negative token constraints and stronger identifier/credential checks.
- Added safe foreign keys for approvals, suspended turns, submissions, and provider exchanges.
- Retained focused recovery, retention, expiry, trace, and active-secret indexes.

## Tests

- Replaced historical migration tests with current-schema initialization tests.

## Documentation

- Added a current table-by-table analysis and superseded the v14 migration requirement.
