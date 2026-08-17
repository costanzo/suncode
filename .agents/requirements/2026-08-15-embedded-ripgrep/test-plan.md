# Test Plan

## Scope

Embedded content search in the Rust operations crate and its runtime integration.

## Unit tests

- Regex matching and byte columns.
- Multiple matches per line and exact truncation.
- Include glob filtering.
- Hidden and gitignored paths.
- Invalid regular expression handling.

## Integration and conformance tests

- Existing `search/find` dispatch test remains passing.

## Regression checks

- Run operations tests, runtime tests, formatting, and `git diff --check`.

## Manual checks

- None planned.

## Commands and results

- `cargo test --manifest-path runtime/Cargo.toml --workspace` passed: 13 operations tests and 25 runtime tests.
- `cargo clippy --manifest-path runtime/Cargo.toml -p suncode-tool --all-targets -- -D warnings` passed.
- `cargo fmt --manifest-path runtime/Cargo.toml --all -- --check` passed.
- `git diff --check` passed.

## Residual risks

- PCRE2 and ripgrep command-line-only features are intentionally not included.
- Workspace-wide clippy has a pre-existing warning in `runtime/crates/core/src/context.rs` outside this change.
