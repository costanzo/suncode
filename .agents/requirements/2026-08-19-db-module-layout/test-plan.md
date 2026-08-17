# Test Plan

## Scope

Verify the database module boundary, deterministic current-schema initialization, new constraints, and unchanged store behavior.

## Unit tests

- Assert the ordered schema manifest has one script per table.
- Assert the empty data manifest applies repeatedly.
- Assert a fresh database contains exactly 16 application tables.
- Assert required indexes and audit immutability triggers exist.
- Apply schema and data manifests twice.
- Reject a database containing an unexpected table and prove the failed open did not alter it.
- Verify terminal continuation cleanup and ordinary persistence behavior.

## Integration and conformance tests

- Run the complete Rust workspace test suite.

## Regression checks

- Run Rust formatting and `git diff --check`.
- Run SQLite `integrity_check` and `foreign_key_check` through focused tests.

## Manual checks

- Inspect the final diff and confirm no schema-version or migration implementation remains in production database code.

## Commands and results

- `cargo test -p suncode-db`: passed, 12 tests.
- `cargo test -p suncode-runtime`: passed, 18 tests.
- `cargo fmt --all -- --check`: passed.
- `cargo test --workspace`: passed, 47 tests across operations, database, and core packages.
- `cargo tree -p suncode-runtime --depth 1`: confirmed `suncode-db` is a direct core dependency and `rusqlite` is not.
- Fresh schema: exactly 16 application tables and 23 explicit indexes; both audit immutability triggers exist.
- SQLite `integrity_check` and `foreign_key_check`: passed in focused coverage.
- Incompatible database rejection: passed and verified transactional rollback.
- `git diff --check` and new-file whitespace checks: passed.

## Residual risks

- Current-schema validation checks the application table set and required initialization objects, not a canonical hash of every table's DDL.
- Schema evolution after a released database exists needs a separate design.
