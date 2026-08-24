# Test Plan

## Scope

Verify the version 14 physical schema, migration preservation, terminal snapshot release, and unchanged runtime behavior.

## Unit tests

- Upgrade a version 13-shaped database with legacy settings and terminal/pending snapshots.
- Verify active-secret reconciliation and uniqueness.
- Verify fresh databases omit legacy tables and redundant indexes while exposing new indexes.
- Verify terminal suspended-turn updates clear snapshots and non-terminal rows retain them.

## Integration and conformance tests

- Run the complete Rust workspace test suite.

## Regression checks

- Run Rust formatting and `git diff --check`.
- Run SQLite `integrity_check` and `foreign_key_check` against a migrated temporary database through tests.

## Manual checks

- Inspect the final schema and query plans for recovery and expiry predicates.

## Commands and results

- `cargo test -p suncode-agent persistence::tests`: passed, 12 tests.
- `cargo test --workspace`: passed, 47 tests.
- `cargo fmt --all -- --check`: passed.
- Fresh in-memory schema: 17 tables, 23 explicit indexes, no retired objects, integrity check passed, and no foreign-key errors.
- SQLite query-plan inspection used the intended turn recovery, suspended resume, checkpoint expiry, provider in-flight, and active-secret indexes.
- `git diff --check`: passed.

## Residual risks

- Turn/submission terminal atomicity, audit completeness, database/checkpoint file permissions, checkpoint payload cleanup, provider trace bounding, and replay-gap detection remain follow-up work documented in `table-analysis.md`.
