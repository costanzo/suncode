# Test Plan

## Scope

Async startup, absence of nested runtimes, blocking parity, turn and continuation adaptation, MCP/LSP/browser methods, C binding compatibility, and desktop regressions.

## Unit tests

- Async SDK opens under `#[tokio::test]`.
- Blocking wrapper opens without an ambient runtime.
- Async and blocking version/health/model operations agree.
- Async session watch remains usable.

## Integration and conformance tests

- Existing Rust SDK tests exercise blocking compatibility.
- New async tests exercise native awaited operations without `block_on`.
- C ABI, C#, and desktop tests pass unchanged.

## Regression checks

- Agent workspace tests.
- Rust SDK and C binding tests.
- Desktop tests.
- Focused Clippy, formatting, and `git diff --check`.

## Manual checks

- None.

## Commands and results

- `cargo test --manifest-path agent/Cargo.toml --workspace --all-targets` — passed: 57 core tests and all other workspace crate/example tests.
- `cargo test --manifest-path sdks/rust/Cargo.toml --lib` — passed: 25 tests, including async current-thread startup and blocking compatibility coverage.
- `cargo test --manifest-path sdks/c/Cargo.toml --lib` — passed: 4 tests.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj` — passed: 107 tests.
- `cargo clippy --manifest-path sdks/rust/Cargo.toml --all-targets -- -D warnings` — passed.
- `cargo clippy --manifest-path sdks/c/Cargo.toml --all-targets -- -A clippy::missing-safety-doc -D warnings` — passed.
- Formatting checks and `git diff --check` — passed.

## Residual risks

- Synchronous SQLite methods can still block an async executor thread; this delivery makes that behavior explicit rather than silently wrapping it.
