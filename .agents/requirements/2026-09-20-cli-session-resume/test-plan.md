# Test Plan

## Scope

Resume grammar, prompt sources, archived reopen, pending-suspension rejection, durable context reuse, typed event/result output, cancellation, lag recovery, and shutdown.

## Unit tests

- Require exactly one resume prompt source.
- Parse model/reasoning options and continue rejecting interactive chat.

## Integration and conformance tests

- Persist and retrieve a pending approval through the Rust SDK.
- Seed an existing session, resume it through a mock provider, and verify prior messages are sent as context.
- Resume an archived session and verify it becomes active.
- Cover prompt and stdin plus JSONL final result.

## Regression checks

- CLI tests and strict Clippy.
- Agent/data workspace and Rust SDK tests.
- C binding and Avalonia tests.
- Formatting and `git diff --check`.

## Manual checks

- Automated process coverage is sufficient for the line-oriented one-shot command.

## Commands and results

- `cargo test --manifest-path apps/cli/Cargo.toml --all-targets --locked --offline` — passed: 8 unit tests and 10 integration tests.
- `cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets --locked --offline -- -D warnings` — passed.
- `cargo test --manifest-path sdks/rust/Cargo.toml --lib --locked --offline` — passed: 31 tests.
- `cargo clippy --manifest-path sdks/rust/Cargo.toml --all-targets --locked --offline -- -D warnings` — passed.
- `cargo clippy --manifest-path agent/Cargo.toml -p suncode-data --all-targets --offline -- -D warnings` — passed.
- `cargo test --manifest-path sdks/c/Cargo.toml --lib --locked --offline` — passed: 4 tests.
- `cargo test --manifest-path agent/Cargo.toml --workspace --all-targets --offline` — passed, including 61 core tests and all workspace targets.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj` — passed: 107 tests.
- CLI, agent, Rust SDK, and C SDK formatting checks — passed.
- `git diff --check` — passed.

## Residual risks

- Full agent-workspace strict Clippy remains blocked by existing `too_many_arguments` and `len_zero` findings in unchanged LLM/tool code; the affected `suncode-data` crate passes strict Clippy.
- Full C-binding strict Clippy remains blocked by existing missing `# Safety` documentation across the unchanged exported FFI surface; C tests pass and this delivery does not change the C ABI.
