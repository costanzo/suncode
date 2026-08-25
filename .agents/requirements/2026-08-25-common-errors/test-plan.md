# Test Plan

## Unit tests

- Serialize a provider error through `BusinessError` and verify code, message, retryability, and request ID.
- Run all existing core, provider, data, database, and tools unit tests.

## Commands and results

- `cargo check --workspace`: passed.
- `cargo test --workspace --lib`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --lib -- -D warnings`: passed.
