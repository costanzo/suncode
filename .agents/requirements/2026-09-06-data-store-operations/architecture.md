# Architecture

## Current state

`Store` owns a shared SQLite connection, but almost every table operation and row conversion is implemented in one 2,000-line `store.rs`. Most table modules under `operations/` are empty placeholders.

## Proposed design

Keep `Store` as the stable public entry type. Rust permits separate inherent `impl Store` blocks, so each table module implements the methods it owns without changing callers. `store.rs` retains connection setup, schema validation, locking, and reusable transaction helpers.

## Boundaries and dependencies

- `operations/<table>.rs`: SQL and row mapping owned by one table.
- `operations/projection.rs`: event-driven writes spanning normalized tables.
- `operations/recovery.rs`: startup reconciliation spanning tables.
- `store.rs`: connection lifecycle and shared infrastructure only.

## Compatibility and migration

All existing `Store::method` signatures remain available. This is a source-layout refactor with no schema, ABI, or wire-format change.

## Risks and rollback

The primary risk is changing transaction boundaries or query ordering during movement. Methods will be moved without semantic rewrites and verified by existing focused and downstream tests.
