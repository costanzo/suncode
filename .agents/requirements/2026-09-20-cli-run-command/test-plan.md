# Test Plan

## Scope

Run grammar, prompt sources, model/reasoning selection, atomic event consumption, tail draining, text/JSONL output, suspension status, cancellation wiring, and shutdown regressions.

## Unit tests

- Require exactly one prompt source.
- Preserve stable exit status mappings.
- Keep JSONL envelope schema version 1.

## Integration and conformance tests

- Seed a temporary data directory through the SDK and route a supported model to a local mock OpenAI-compatible SSE provider.
- Assert typed `assistant.delta` output and the completed `run.result` response.
- Run a second turn with piped stdin plus `SUNCODE_MODEL` and `SUNCODE_REASONING_EFFORT`.
- Assert text stdout contains only final assistant content and progress appears on stderr.
- Reject run without a prompt before opening the SDK.

## Regression checks

- CLI tests and strict Clippy.
- Rust SDK and C binding tests.
- Agent workspace tests.
- Avalonia desktop tests.
- Rust formatting and `git diff --check`.

## Manual checks

- Automated child-process tests cover the user-visible run modes; no external provider credential is required.

## Commands and results

- `cargo test --manifest-path apps/cli/Cargo.toml --all-targets --locked --offline` — passed: 7 unit tests and 6 integration tests.
- `cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets --locked --offline -- -D warnings` — passed.
- `cargo test --manifest-path sdks/rust/Cargo.toml --lib --locked --offline` — passed: 30 tests.
- `cargo test --manifest-path sdks/c/Cargo.toml --lib --locked --offline` — passed: 4 tests.
- `cargo test --manifest-path agent/Cargo.toml --workspace --all-targets --offline` — passed, including 61 core tests and all workspace crate targets.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj` — passed: 107 tests.
- CLI, agent, Rust SDK, and C SDK formatting checks — passed.
- `git diff --check` — passed.

## Residual risks

- OS signal delivery timing is implemented against Tokio but is not driven by a process-level signal integration test.
- Interactive approvals/questions and session continuation are intentionally outside this delivery.
