# Changes

## Source

- Replaced Rust and Avalonia `SUNCODE_LOG_*` reads with configuration supplied through SQLite and the SDK.
- Added runtime validation and immediate logger reconfiguration for known logging keys.
- Renamed the project schema resource, table, index, foreign keys, and SQL queries to `project`.

## Contracts and generated artifacts

- Updated the SQLite and persistence contracts.
- No generated contract artifacts or ABI changes.

## Configuration and persistence

- Seeded four typed global logging defaults.
- Kept data/database paths as external bootstrap configuration.
- Deliberately added no automatic conversion of the former `projects` schema.

## Tests

- Added database coverage for seeded values and current schema objects.
- Added runtime coverage for logging setting scope, types, allowed levels, and ranges.

## Documentation

- Added an ADR and updated the runtime feature and diagnostic logging records.
