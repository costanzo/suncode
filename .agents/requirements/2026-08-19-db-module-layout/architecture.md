# Architecture

> Historical record. This baseline predates the persisted LLM catalog and its current 17-table schema.

## Current state

The original core crate had a monolithic persistence module and schema file. Database open applied version probes and upgrade functions for historical schema shapes.

## Proposed design

```text
runtime/crates/db/
  Cargo.toml
  src/
    lib.rs
    domain.rs
    store.rs
    schema/
      mod.rs
      audit_records.sql
      ...
      secret_records.sql
    data/
      mod.rs
```

The Cargo package is `suncode-db`, and its Rust library name is `suncode_db`.

`schema/mod.rs` is the source-controlled master list and the only source of execution order. SQL files use table names without numeric prefixes. No applied-script IDs or versions are written to SQLite. Each SQL file owns one table plus indexes and triggers whose lifecycle belongs to that table.

`data/mod.rs` provides the same ordered-script shape for future immutable reference data. Its current list is intentionally empty.

Database open enables connection pragmas, starts one transaction, applies the complete current schema, validates the exact application table set, applies data scripts, and commits. Any failure rolls back the initialization transaction.

## Boundaries and dependencies

- The database package owns store-facing domain records and SQLite access.
- Its `schema` and `data` modules contain initialization resources only; they do not expose runtime query APIs.
- The core package depends on `suncode-db` and does not depend on `rusqlite`.
- The database package does not depend on core, preventing a cyclic crate graph.
- Clients, providers, and operations do not open SQLite.

## Data and control flow

```text
RuntimeSdk::open
  -> Store::open
     -> configure SQLite connection
     -> begin transaction
     -> apply ordered schema scripts
     -> verify exact 16-table set
     -> apply ordered data scripts
     -> commit
```

Normal store methods continue to own their existing transaction boundaries. Terminal suspended-turn updates clear `snapshot_json` in the same statement that changes status.

## Security and failure handling

The database contains plaintext provider credentials and remains sensitive. SQL files contain no secrets or machine-specific values. An incompatible database produces an explicit open error. Initialization neither drops unexpected objects nor attempts best-effort conversion.

## Compatibility and migration

There is intentionally no database compatibility or migration behavior. Existing schemas from earlier development are unsupported. Reopening the exact current schema is idempotent; changing it after release requires a new approved design.

## Risks and rollback

- Splitting SQL can introduce dependency-order errors; focused initialization tests cover the complete manifest.
- Extra foreign keys can conflict with valid write ordering; only relationships guaranteed by current runtime flow are enforced.
- A pre-current local database will no longer open. This is intentional for the new-system phase and is reported rather than silently modified.

## Open questions

- None for this delivery.
