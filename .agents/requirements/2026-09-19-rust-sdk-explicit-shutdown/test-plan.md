# Test Plan

## Scope

Shutdown admission control, cancellation, manager draining, stream closure, runtime ownership, native close adaptation, and data-directory lock release.

## Unit tests

- Event-hub close wakes and terminates subscribers.
- Repeated core shutdown is harmless.
- Manager shutdown drains installed resources where test doubles are available.

## Integration and conformance tests

- Async SDK shutdown works inside an existing Tokio runtime.
- The same data directory reopens after successful shutdown.
- SDK shutdown closes a pending session event stream.
- C close preserves its existing symbol and signature and invokes blocking shutdown.

## Regression checks

- Agent workspace tests.
- Rust SDK tests and Clippy.
- C binding tests and Clippy.
- Avalonia tests.
- Rust formatting and `git diff --check`.

## Manual checks

- None required if focused lifecycle tests can execute.

## Commands and results

- `cargo test --manifest-path agent/Cargo.toml -p suncode-agent event_hub::tests --offline` — passed: 10 focused event-hub tests, including shutdown close-all behavior.
- `cargo check --manifest-path sdks/rust/Cargo.toml --all-targets --offline` — passed.
- `cargo check --manifest-path sdks/c/Cargo.toml --all-targets --offline` — passed.
- `cargo clippy --manifest-path sdks/rust/Cargo.toml --all-targets --offline -- -D warnings` — passed.
- `cargo clippy --manifest-path sdks/c/Cargo.toml --all-targets --offline -- -A clippy::missing-safety-doc -D warnings` — passed; the allowance covers pre-existing unsafe-export documentation debt.
- `cargo clippy --manifest-path agent/Cargo.toml -p suncode-agent --all-targets --no-deps --offline -- -D warnings -A clippy::needless_return -A clippy::unnecessary_unwrap -A clippy::collapsible_if -A clippy::unused_enumerate_index` — passed; the allowances cover pre-existing core findings outside this delivery.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj` — passed: 107 tests.
- Rust formatting checks — passed.
- `git diff --check` — passed.
- `cargo test --manifest-path sdks/rust/Cargo.toml --lib --offline` — blocked before shutdown assertions: the current seeded Anthropic adapter violates the current SQLite `adapter_type IN ('openai')` constraint.
- `cargo test --manifest-path sdks/c/Cargo.toml --lib --offline` — blocked by the same SQLite initialization issue and by the current baseline test expecting ABI 10 while the SDK constant is 13.

## Residual risks

- A hostile or non-cooperative external process may outlive the five-second active-turn grace period; shutdown reports an error and releases the SDK handle after all manager close operations have still been requested.
