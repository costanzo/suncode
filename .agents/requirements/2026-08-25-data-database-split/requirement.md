# Requirement

## Background

The current database crate combines SQL resources, database initialization, Diesel schema declarations, ORM operations, and domain persistence behavior.

## Goals

- Create `suncode-database` for database resources and backend-specific database setup.
- Create `suncode-data` for Diesel connections, ORM schema, persistence DTOs, and table operations.
- Put SQLite implementation under `suncode-database::sqlite` so other database backends can be added later.
- Keep the existing `Store` API and current schema behavior for core.

## Non-goals

- No MySQL or PostgreSQL implementation in this change.
- No migration or legacy database conversion path.
- No public SDK DTO redesign.

## Requirements

- `suncode-database` must not depend on Diesel.
- `suncode-database::sqlite` owns all SQLite SQL scripts, seed scripts, table manifest, and database-file creation/existence checks.
- `suncode-data` owns Diesel and executes the script resources through its connection layer.
- Core depends on `suncode-data`; the old combined `suncode-db` package is removed.

## Acceptance criteria

- Workspace contains `crates/database` and `crates/data` packages named `suncode-database` and `suncode-data`.
- `cargo test -p suncode-database`, `cargo test -p suncode-data`, and workspace library tests pass.
- Diesel appears only in `suncode-data` among the two persistence packages.
- `git diff --check` and strict workspace clippy pass.
