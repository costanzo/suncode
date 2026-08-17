# Requirement

> Historical record. The 16-table baseline described here was superseded by the persisted LLM catalog requirement, which adds `llm_model_provider` and `llm_model` and removes the legacy `secret_records` table from the current schema.

## Background

Database code was concentrated in `persistence.rs`, while every table, index, trigger, and migration marker lived in one root `schema.sql`. That shape mixed current storage behavior with obsolete upgrade paths and made table ownership difficult to review.

SunCode is a new system. It needs one current schema and does not need schema versions, migration metadata, upgrade functions, or legacy-table conversion.

## Goals

- Give database code one explicit `runtime/crates/db/` package boundary.
- Name the Cargo package `suncode-db` and its Rust library crate `suncode_db`.
- Keep the ordered schema manifest separate from per-table SQL files.
- Keep future reference/seed data in a separate ordered `data/` manifest.
- Define exactly one current 16-table baseline with focused constraints, foreign keys, indexes, and triggers.
- Reject incompatible databases without mutating or migrating them.
- Preserve current runtime persistence behavior, including clearing terminal approval continuation snapshots.

## Non-goals

- Preserve or upgrade any pre-current database.
- Add a migration/version table or migration runner.
- Add reference data before the product has a concrete dataset.
- Change SDK DTOs, retention behavior, or the accepted plaintext credential design.

## Requirements

- `runtime/crates/db/src/store.rs` owns runtime queries and transactions.
- `runtime/crates/db/src/schema/mod.rs` is the ordered schema master list.
- `runtime/crates/db/src/schema/*.sql` contains exactly one table definition and that table's indexes or triggers.
- `runtime/crates/db/src/data/mod.rs` is an explicit ordered master list and may be empty.
- Persistence DTOs used by the store API are owned and exported by the database package; core may re-export them for its public SDK surface.
- Schema and data scripts execute in one initialization transaction.
- SQL file names contain no ordering numbers; manifest position is the only execution order.
- The database contains exactly the 16 documented product tables and no schema metadata table.
- Reopening a current database is idempotent.
- A database with an unexpected application table is rejected; no object is dropped or converted.

## Edge cases

- A partially initialized transaction must roll back.
- An incompatible existing database must remain unchanged after open fails.
- Foreign keys must be valid after initialization.
- Pending/resuming suspended turns retain snapshots; terminal rows release them.
- The empty data manifest must remain valid and idempotent.

## Acceptance criteria

- Focused tests cover ordered manifests, exact tables, required indexes/triggers, idempotent initialization, incompatible database rejection, integrity, foreign keys, and terminal snapshot cleanup.
- Existing database-backed runtime and SDK tests pass.
- Rust formatting, `cargo test --workspace`, and `git diff --check` pass.

## Open questions

- A future released product that needs schema evolution must define a separate compatibility policy and migration design before changing the current schema.
