# Test Plan

## Scope

The standalone LLM package, custom registration, built-in OpenAI-compatible streaming, and core integration.

## Unit tests

- Route all built-in models.
- Register and route a custom model provider.
- Reject duplicate provider and model IDs atomically.
- Parse streamed text, usage, and tool calls.

## Integration and conformance tests

- Run the existing agent provider round trip and SDK model catalog tests.

## Regression checks

- Run all workspace tests, formatting, focused clippy, and diff checks.
- Confirm the LLM dependency tree excludes database and runtime packages.

## Manual checks

- Inspect public API ownership and secret redaction paths.

## Commands and results

- `cargo test --workspace`: passed, 49 tests across database, LLM, runtime, and tool packages.
- `cargo clippy -p suncode-llm --all-targets -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo tree -p suncode-llm --depth 1`: confirmed no SunCode database, runtime, or tool dependency.
- `git diff --check`: passed.
- `cargo clippy -p suncode-runtime --lib -- -D warnings`: blocked by the pre-existing `manual_clamp` warning in `runtime/crates/core/src/context.rs`; the changed LLM package passes strict clippy.

## Residual risks

- Provider-native protocols and persisted enterprise configuration remain deferred.
