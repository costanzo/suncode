# Requirement

## Background

The Rust database package currently owns persistence through a large `store.rs` implementation that directly uses `rusqlite`. Table operations are difficult to review because unrelated persistence concerns share one file.

## Goals

- Use Diesel's SQLite backend for database access in `suncode-db`.
- Keep the current 15-table schema, initialization checks, and public `Store` behavior.
- Organize persistence operations by table, with explicit modules for cross-table projections and recovery.

## Non-goals

- No schema migration runner or compatibility conversion.
- No public DTO or SDK contract redesign.
- No changes to database ownership boundaries.

## Requirements

- Production database code must not depend on `rusqlite`.
- Initialization and seeded catalog data must execute transactionally through Diesel.
- Table-specific queries and row conversion must live in table-named operation modules.
- Cross-table queries must be isolated in named aggregate operation modules.

## Edge cases

- Fresh and in-memory databases must behave identically.
- Existing incompatible databases must still be rejected without conversion.
- SQLite foreign keys and WAL/busy-timeout settings must remain enabled where applicable.
- JSON and unsigned token conversions must retain existing validation behavior.

## Acceptance criteria

- `cargo test -p suncode-db` passes.
- `cargo test --workspace --lib` passes.
- `rusqlite` is absent from the database crate's production dependencies and source.
- The database source contains table-specific operation modules rather than one monolithic operation file.

## Open questions

- None.
