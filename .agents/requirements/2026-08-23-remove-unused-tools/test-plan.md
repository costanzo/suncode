# Test Plan

## Scope

Exact model names, policy, argument translation, operation dispatch, process execution, checkpoint restore, desktop summaries, and source-level absence of retired paths.

## Unit tests

- Assert the exact seven-tool registry and canonical policy names.
- Retain bash translation, timeout, cancellation, and output tests.
- Retain write/edit/checkpoint and WebFetch tests.

## Integration and conformance tests

- Run the complete Rust workspace.
- Run the Avalonia desktop test suite.

## Regression checks

- Run Rust formatting and production clippy.
- Scan for removed names and operation methods.
- Run `git diff --check` and inspect the final diff.

## Manual checks

- Inspect the final dispatcher and agent mapping directly.

## Commands and results

- `cargo test --manifest-path agent/Cargo.toml --workspace`: passed (35 db, 3 LLM, 32 core, 30 operations).
- `dotnet test apps/desktop-avalonia-tests/SunCode.Desktop.Tests.csproj --no-restore`: passed (39).
- `cargo fmt --manifest-path agent/Cargo.toml --all -- --check`: passed.
- `cargo clippy --manifest-path agent/Cargo.toml --workspace --lib -- -D warnings`: passed.
- `git diff --check`: passed.

## Residual risks

- Historical requirement packages retain their original decision text; current source and contracts no longer execute those names.
