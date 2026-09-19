# Test Plan

## Scope

Standard stream polling, typed event delivery, closure, fused termination, one-shot lag, and compatibility with the existing receive methods and native adapter.

## Unit tests

- `StreamExt::next` returns a typed event.
- Explicit close wakes the stream and returns `None`.
- A terminated stream remains terminated.
- Lag produces one typed error and then terminates.

## Integration and conformance tests

- Rust SDK tests cover the public standard stream.
- C binding tests cover unchanged blocking adaptation.
- Avalonia tests cover unchanged managed callback behavior.

## Regression checks

- Agent workspace tests.
- Rust SDK tests and Clippy.
- C binding tests and Clippy.
- Avalonia desktop tests.
- Rust formatting and `git diff --check`.

## Manual checks

- None required.

## Commands and results

- `cargo test --manifest-path agent/Cargo.toml --workspace --all-targets` — passed: 57 core tests and all other workspace crate/example tests.
- `cargo test --manifest-path sdks/rust/Cargo.toml --lib` — passed: 28 tests.
- `cargo test --manifest-path sdks/c/Cargo.toml --lib` — passed: 4 tests.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj` — passed: 107 tests.
- `cargo clippy --manifest-path sdks/rust/Cargo.toml --all-targets -- -D warnings` — passed.
- `cargo clippy --manifest-path sdks/c/Cargo.toml --all-targets -- -A clippy::missing-safety-doc -D warnings` — passed; the allowance covers the binding's pre-existing unsafe-export documentation debt.
- `cargo clippy --manifest-path agent/Cargo.toml -p suncode-agent --all-targets --no-deps -- -D warnings -A clippy::needless_return -A clippy::unnecessary_unwrap -A clippy::collapsible_if -A clippy::unused_enumerate_index` — passed; the allowances cover pre-existing findings outside the changed event hub.
- Rust formatting checks — passed.
- `git diff --check` — passed.

## Residual risks

- Synchronous SQLite projection methods remain intentionally synchronous on the async facade.
- Full strict agent-workspace Clippy remains blocked by pre-existing `too_many_arguments`, `useless_conversion`, `needless_return`, `unnecessary_unwrap`, `collapsible_if`, and `unused_enumerate_index` findings outside this delivery.
