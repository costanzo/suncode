# Test Plan

## Scope

Provider parsing, model lookup, tool schema registration, agent round trips, and the Qt SDK build.

## Unit tests

- DeepSeek SSE parser retains split tool calls and usage.
- Registry accepts advertised models and rejects unknown identifiers.
- Registry returns every built-in tool exactly once.
- Built-in tool registry advertises OpenCode-aligned names for implemented tools.

## Integration and conformance tests

- Existing agent read/write approval tests.
- Existing runtime SDK request tests.

## Regression checks

- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`

## Manual checks

- Configure and build the Qt desktop target with the repository's Qt CMake binary.

## Commands and results

- `cargo test --workspace`: 30 tests passed after OpenCode-aligned tool names.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed after OpenCode-aligned tool names.
- Qt CMake configure/build: passed.
- `QT_QPA_PLATFORM=offscreen ./build/suncode-desktop`: initialized without QML binding errors; stopped after startup smoke check.

## Residual risks

The remaining `operations/src/lib.rs` is still the largest Rust file, but now contains the audited dispatcher, path policy, journaling, and public facade rather than individual tool implementations. Further extraction should follow a behavior-driven change rather than splitting shared policy into speculative modules.
