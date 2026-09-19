# Test Plan

## Scope

Session gate ordering, atomic snapshot-plus-stream establishment, failure cleanup, existing lag behavior, and compatibility regressions.

## Unit tests

- An event committed before the watch gate appears in the snapshot and not the stream.
- An event started after watch establishment appears in the stream and not the snapshot.
- A live-only event waits behind the watch gate and enters the returned stream.
- Snapshot failure unregisters the provisional subscriber.
- Separate sessions do not block one another.

## Integration and conformance tests

- Rust `watch_session` returns the current normalized snapshot and typed stream.
- Existing snapshot and subscribe methods continue to operate.
- The C ABI and legacy JSON envelope remain unchanged.

## Regression checks

- Agent workspace tests.
- Rust SDK tests and Clippy.
- C binding tests and focused Clippy.
- Avalonia desktop tests.
- Formatting and `git diff --check`.

## Manual checks

- None required if deterministic gate tests and desktop tests pass.

## Commands and results

- `cargo test --manifest-path agent/Cargo.toml -p suncode-agent event_hub::tests` — passed: 9 focused tests.
- `cargo test --manifest-path agent/Cargo.toml --workspace --all-targets` — passed: 57 core tests and all other workspace crate/example tests.
- `cargo test --manifest-path sdks/rust/Cargo.toml --lib` — passed: 24 tests.
- `cargo test --manifest-path sdks/c/Cargo.toml --lib` — passed: 3 tests.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj` — passed: 106 tests.
- `cargo clippy --manifest-path sdks/rust/Cargo.toml --all-targets -- -D warnings` — passed.
- `cargo clippy --manifest-path sdks/c/Cargo.toml --all-targets -- -A clippy::missing-safety-doc -D warnings` — passed.
- Formatting checks and `git diff --check` — passed.

## Residual risks

- Native and managed host adoption was completed by `../2026-09-19-native-dormant-session-watch/`.
- Some normalized lifecycle writes intentionally precede notification projection; hosts must continue applying stream events idempotently.
