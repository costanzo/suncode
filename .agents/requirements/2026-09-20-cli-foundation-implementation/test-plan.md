# Test Plan

## Scope

Argument grammar, environment precedence, SDK startup profile, administrative commands, secret input, text/JSONL separation, exit statuses, and shutdown.

## Unit tests

- Parse implemented commands and reject unimplemented commands.
- Explicit options override `SUNCODE_` environment values.
- Invalid environment values fail closed.
- JSONL reports use schema version 1.
- SDK business errors map to stable exit statuses.

## Integration and conformance tests

- Execute `doctor --output jsonl` against a fresh isolated data directory.
- Execute `models` and observe seeded model output.
- Execute `auth set` without a TTY and receive exit 2 without reading a secret.
- Verify help exits 0 without opening the SDK.
- Verify unimplemented conversational commands exit 2 without opening the SDK.

## Regression checks

- CLI tests and Clippy.
- Agent workspace tests.
- Rust SDK tests and Clippy.
- C binding tests and Clippy.
- Avalonia tests.
- Locked/offline builds, Rust formatting, and `git diff --check`.

## Manual checks

- Run JSONL doctor and text auth-list commands against one temporary data directory.

## Commands and results

- `cargo test --manifest-path apps/cli/Cargo.toml --all-targets --locked --offline` — passed: 6 unit tests and 5 integration tests.
- Manual JSONL doctor and text auth-list smoke — passed.
- `cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets --locked --offline -- -D warnings` — passed.
- `cargo check --manifest-path apps/cli/Cargo.toml --all-targets --locked --offline` — passed.
- `cargo test --manifest-path agent/Cargo.toml --workspace --all-targets --offline` — passed: 61 core tests and all other agent workspace tests.
- `cargo test --manifest-path sdks/rust/Cargo.toml --lib --locked --offline` — passed: 30 tests.
- `cargo test --manifest-path sdks/c/Cargo.toml --lib --locked --offline` — passed: 4 tests.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj` — passed: 107 tests.
- Rust formatting checks — passed.
- `git diff --check` — passed.

## Residual risks

- Conversational event rendering and approval UX remain unimplemented by design.
