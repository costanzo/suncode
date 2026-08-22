# Architecture

## Current state

Rust owns SQLite and exposes effective configuration through the embedded SDK. Avalonia owns presentation and must not open SQLite. The schema initializer accepts only one exact current table set and has no migration runner.

## Proposed design

The `configuration` schema resource seeds four global logging values. Runtime startup opens and validates SQLite first, reads those values, then configures `runtime.log`. Avalonia starts with stderr-only diagnostics, obtains the same values through `list_settings`, and configures `desktop.log`.

The project identity resource, physical table, index, foreign keys, and Store SQL use singular `project`. Public domain vocabulary and SDK response fields remain project/projects as grammatically appropriate; this change concerns only physical SQLite naming.

## Boundaries and dependencies

- `suncode-db` owns table definitions, seed rows, and setting persistence.
- Runtime core validates known logging keys and owns Rust logger reconfiguration.
- Avalonia reads settings DTOs and owns only its file writer.
- The SDK ABI method set and DTO shapes remain unchanged.

## Data and control flow

```text
bootstrap data/database path
    -> open current SQLite schema
    -> read global logging configuration
    -> configure runtime.log
    -> list_settings through SDK
    -> configure desktop.log
```

SDK writes to a known global logging key persist first and then rebuild the Rust logger state from effective configuration.

## Security and failure handling

Logs remain files with bounded rotation, not database content. Credential and content logging remains prohibited. Invalid logging values fail with `invalid_arguments`; file failures fall back to stderr.

## Compatibility and migration

No automatic migration is added. The schema manifest now expects `project`, so an existing database containing `projects` is rejected as incompatible without conversion or deletion. A fresh database receives the new table and seeded settings.

## Risks and rollback

The table rename prevents older development databases from opening until a separately approved migration exists or the user explicitly chooses a fresh data directory. Reverting the schema and query rename restores compatibility only with the former exact schema.

## Open questions

- When a released schema requires evolution, define versioning, backups, and transactional migration separately.
