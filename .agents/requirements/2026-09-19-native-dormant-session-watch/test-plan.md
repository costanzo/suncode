# Test Plan

## Scope

Dormant native handle creation, explicit start, close-before-start, double start, snapshot serialization, managed lifetime, desktop stale-load behavior, and regression compatibility.

## Unit tests

- C watch returns the snapshot and no callback before start.
- Starting delivers queued typed events through the legacy envelope.
- Closing before start releases the stream without a worker.
- Starting twice fails with `conflict`.
- Callback self-disposal does not self-join.

## Integration and conformance tests

- C# deserializes the snapshot and typed events.
- Desktop applies snapshot before callback activation.
- Stale/failing loads dispose dormant watches.
- Resync uses the same atomic path.

## Regression checks

- Agent workspace, Rust SDK, C binding, C# SDK, and desktop tests.
- Formatting, focused Clippy, and `git diff --check`.

## Manual checks

- No visual review is required because loading visuals and copy are unchanged.

## Commands and results

- `cargo test --manifest-path agent/Cargo.toml --workspace --all-targets` — passed: 57 core tests and all other workspace crate/example tests.
- `cargo test --manifest-path sdks/rust/Cargo.toml --lib` — passed: 24 tests.
- `cargo test --manifest-path sdks/c/Cargo.toml --lib` — passed: 4 tests.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj` — passed: 107 tests.
- `cargo clippy --manifest-path sdks/rust/Cargo.toml --all-targets -- -D warnings` — passed.
- `cargo clippy --manifest-path sdks/c/Cargo.toml --all-targets -- -A clippy::missing-safety-doc -D warnings` — passed; the allowance covers the crate's pre-existing undocumented unsafe C exports.
- Rust formatting checks and `git diff --check` — passed.

## Residual risks

- Child-session read-only inspection remains snapshot-only.
- The complete agent workspace strict Clippy run remains blocked by previously documented `too_many_arguments` findings outside this delivery.
