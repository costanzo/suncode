# Test Plan

## Scope

Nested session grammar, project selection, list rendering, archive mutation through the SDK, JSONL results, errors, and shutdown regression.

## Unit tests

- Parse list and archive.
- Continue rejecting resume and chat.

## Integration and conformance tests

- Seed primary sessions through `AsyncAgentSdk` in an isolated data directory.
- List them through the CLI and verify stable IDs/statuses in JSONL.
- Archive one through the CLI and verify the returned record plus subsequent list state.

## Regression checks

- CLI tests and strict Clippy.
- Rust SDK, C binding, agent workspace, and Avalonia tests.
- Formatting and `git diff --check`.

## Manual checks

- Automated child-process coverage is sufficient for this line-oriented administrative slice.

## Commands and results

- `cargo test --manifest-path apps/cli/Cargo.toml --all-targets --locked --offline` — passed: 8 unit tests and 7 integration tests.
- `cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets --locked --offline -- -D warnings` — passed.
- `cargo test --manifest-path sdks/rust/Cargo.toml --lib --locked --offline` — passed: 30 tests.
- `cargo test --manifest-path sdks/c/Cargo.toml --lib --locked --offline` — passed: 4 tests.
- `cargo test --manifest-path agent/Cargo.toml --workspace --all-targets --offline` — passed, including 61 core tests and all workspace crate targets.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj` — passed: 107 tests.
- CLI, agent, Rust SDK, and C SDK formatting checks — passed.
- `git diff --check` — passed.

## Residual risks

- Resume was outside this delivery and was later implemented as a separate one-shot command.
