# Progress

- Status: Complete
- Last updated: 2026-09-06

## Completed

- Mapped existing Store methods to table and orchestration modules.
- Confirmed the public Store API can remain source-compatible through separate inherent impl blocks.

## In progress

- Table operations are now implemented in table-owned operation modules; cross-table projection and startup recovery remain explicit orchestration modules.
- `store.rs` now contains connection lifecycle, schema initialization, transactions, locking, shared row decoding helpers, and health checks.
- `store.rs` no longer contains table-specific row decoding helpers; those are owned by the corresponding operation modules.
- Store tests were moved to `src/tests.rs`.

## Blocked

- None.
