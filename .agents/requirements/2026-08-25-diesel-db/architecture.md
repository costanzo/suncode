# Architecture

## Current state

`Store` owns one mutex-protected `rusqlite::Connection`, schema/data initialization, all table operations, projections, recovery, and tests in `store.rs`.

## Proposed design

`Store` remains the public facade. It owns a mutex-protected Diesel `SqliteConnection` and delegates table-owned work to `operations/<table>.rs`. `projection.rs` handles event fan-out across normalized tables, while `recovery.rs` handles startup and suspended-turn state that spans approval and turn rows. Existing SQL schema files remain the initialization manifest; Diesel `table!` declarations describe queryable tables.

## Boundaries and dependencies

Only `suncode-db` opens SQLite. Diesel is the production database dependency. Core continues to consume `Store` and domain DTOs without database knowledge.

## Data and control flow

Store methods lock the Diesel connection, invoke an operation module, and translate Diesel/query conversion errors into `PersistenceError`. Multi-row event projection runs in one Diesel transaction.

## Security and failure handling

Foreign keys, schema validation, immutable audit triggers, JSON checks, and current database rejection remain unchanged. Transactions are used for initialization and atomic projection/approval updates.

## Compatibility and migration

There is no migration path. The current schema is initialized only for fresh/current databases, consistent with the new-project decision.

## Risks and rollback

The main risk is query/conversion drift while replacing rusqlite row callbacks. Focused database tests and workspace tests are required before closeout. Rollback is a source-level revert; no database conversion is introduced.

## Open questions

- None.
