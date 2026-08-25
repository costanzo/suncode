# Architecture

## Current state

The combined `suncode-db` package contains SQL scripts, seed data, Diesel table declarations, SQLite initialization, and all Store operations.

## Proposed design

`suncode-database` is the backend-resource layer. Its `sqlite` child module contains schema/data SQL manifests, the current table manifest, and `ensure_database(path)` which creates the parent directory and empty database file when absent. It has no Diesel dependency.

`suncode-data` is the ORM/data layer. It owns the Diesel `SqliteConnection`, table declarations, domain DTOs, table operations, projections, recovery, and `Store`. During open it asks `suncode-database::sqlite` to ensure the file exists, opens it with Diesel, applies the SQLite resources in a transaction, and validates the current schema.

## Boundaries and dependencies

```text
suncode-agent -> suncode-data -> suncode-database
                              \-> diesel (SQLite)
suncode-database -> std only
```

The database package exposes resources and setup metadata, not ORM rows or Diesel types. The data package is the only package that opens a Diesel connection.

## Backend extension

Future `mysql` and `postgresql` modules belong under `suncode-database/src/`, each owning its scripts and file/connection setup contract. `suncode-data` can later add backend-specific Diesel feature bindings without changing core's Store facade.

## Compatibility and migration

This is a new package layout with no migration or compatibility alias. The current SQLite schema and public Store methods remain unchanged.
