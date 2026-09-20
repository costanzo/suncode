# Test Plan

## Scope

Default compatibility, option-aware startup, Browser and Computer capability enforcement, persisted-setting isolation, backend initialization, and native/desktop regression.

## Unit tests

- Disabled Browser manager returns no tool catalog and rejects starts.
- Disabled Computer manager returns no native toolset and performs no backend work.

## Integration and conformance tests

- Open SDK with both persisted settings true and both host capabilities false.
- Runtime information reports desired enablement but unavailable effective capability.
- Named mutations return stable errors and settings remain true.
- Default SDK and native paths retain prior behavior.

## Regression checks

- Agent workspace tests.
- Rust SDK tests.
- C binding tests.
- Avalonia tests.
- Rust formatting, applicable Clippy, locked/offline builds, and `git diff --check`.

## Manual checks

- None required.

## Commands and results

- `cargo test --manifest-path agent/Cargo.toml --workspace --all-targets --offline` — passed: 61 core tests and all other workspace crate/example tests.
- `cargo test --manifest-path sdks/rust/Cargo.toml --lib --locked --offline` — passed: 30 tests.
- `cargo test --manifest-path sdks/c/Cargo.toml --lib --locked --offline` — passed: 4 tests.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj` — passed: 107 tests.
- `cargo clippy --manifest-path sdks/rust/Cargo.toml --all-targets --locked --offline -- -D warnings` — passed.
- `cargo clippy --manifest-path sdks/c/Cargo.toml --all-targets --locked --offline -- -A clippy::missing-safety-doc -D warnings` — passed; the allowance covers pre-existing unsafe-export documentation debt.
- `cargo clippy --manifest-path agent/Cargo.toml -p suncode-agent --all-targets --no-deps --offline -- -D warnings -A clippy::needless_return -A clippy::unnecessary_unwrap -A clippy::collapsible_if -A clippy::unused_enumerate_index` — passed; allowances cover pre-existing core findings outside this delivery.
- Rust formatting checks — passed.
- Rust SDK and C binding locked/offline checks — passed.
- `git diff --check` — passed.

## Residual risks

- The future CLI must explicitly pass disabled Browser and Computer capabilities; default options intentionally preserve desktop behavior.
