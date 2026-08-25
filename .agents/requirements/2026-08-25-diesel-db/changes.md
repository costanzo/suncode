# Changes

## Source

- Replace the database crate's direct rusqlite connection and row access with Diesel SQLite access.
- Split table-owned operations into named operation modules.

## Contracts and generated artifacts

- No public protocol or DTO changes.

## Configuration and persistence

- Preserve the current 15-table schema and seed manifests.

## Tests

- Retain and adapt existing database regression tests.

## Documentation

- Update the current persistence specification after implementation.
