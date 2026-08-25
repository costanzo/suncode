# Test Plan

## Unit tests

- SQLite database file creation is idempotent.
- SQLite resource manifests contain all current tables and seed scripts.

## Integration tests

- `suncode-data` opens fresh and in-memory databases and preserves Store behavior.
- Workspace core tests continue to pass through `suncode-data`.

## Commands

- `cargo test -p suncode-database`
- `cargo test -p suncode-data`
- `cargo test --workspace --lib`
- `cargo clippy --workspace --lib -- -D warnings`
- `git diff --check`
