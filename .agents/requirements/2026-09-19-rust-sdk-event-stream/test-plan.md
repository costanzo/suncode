# Test Plan

## Scope

Typed event construction, session-scoped fan-out, bounded lag behavior, stream close, C envelope compatibility, and existing desktop subscription behavior.

## Unit tests

- Event payloads retain their stable dotted event type and serialized payload.
- A subscription receives events only for its session.
- A full subscriber queue reports lag.
- Explicit close wakes a blocking receiver.

## Integration and conformance tests

- The Rust SDK returns typed events without JSON parsing.
- The C adapter emits the existing JSON envelope.
- C ABI version and native symbol family remain unchanged.

## Regression checks

- Agent core workspace tests.
- Rust SDK tests.
- C binding tests.
- Avalonia SDK and desktop tests.
- Formatting, Clippy where applicable, and `git diff --check`.

## Manual checks

- None required if focused callback lifecycle tests and the desktop suite pass.

## Commands and results

- `cargo test --manifest-path agent/Cargo.toml --workspace --all-targets` — passed: 52 core tests and all other workspace crate/example tests.
- `cargo test --manifest-path agent/Cargo.toml -p suncode-agent event_hub::tests` — passed: 4 focused hub tests.
- `cargo test --manifest-path sdks/rust/Cargo.toml --lib` — passed: 23 tests.
- `cargo test --manifest-path sdks/c/Cargo.toml --lib` — passed: 3 tests.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj` — passed: 106 tests.
- `cargo clippy --manifest-path sdks/rust/Cargo.toml --all-targets -- -D warnings` — passed.
- `cargo clippy --manifest-path sdks/c/Cargo.toml --all-targets -- -A clippy::missing-safety-doc -D warnings` — passed. The crate-wide strict command without the existing allowance remains blocked by pre-existing missing `# Safety` documentation on the C ABI exports.
- `git diff --check` — passed.

## Residual risks

- Atomic snapshot-plus-stream establishment remains follow-up work.
- The complete agent workspace strict Clippy run remains blocked by pre-existing `too_many_arguments` findings in the LLM and tools crates.
