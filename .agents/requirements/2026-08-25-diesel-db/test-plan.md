# Test Plan

## Scope

Database initialization, table CRUD, event projection, recovery, provider catalog, configuration, and schema rejection behavior.

## Unit tests

- Operation row conversion and JSON/unsigned integer validation.
- Fresh schema manifest and seed idempotency.

## Integration and conformance tests

- Existing `suncode-db` tests, followed by workspace library tests.

## Regression checks

- Public `Store` API remains callable by core.
- No `rusqlite` production dependency remains in `suncode-db`.

## Manual checks

- Inspect operation module layout and `git diff --check`.

## Commands and results

- `cargo test -p suncode-db`: passed, 8 tests.
- `cargo test --workspace --lib`: passed, all crate tests.
- `cargo clippy --workspace --lib -- -D warnings`: passed.
- `git diff --check`: passed.

## Residual risks

- Legacy projection SQL remains explicit because it spans several normalized tables; it executes through Diesel transactions and typed query results.
