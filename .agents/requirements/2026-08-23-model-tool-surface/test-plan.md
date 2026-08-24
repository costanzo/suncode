# Test Plan

## Scope

Model tool registration, host prompt guidance, and retained internal compatibility.

## Unit tests

- Assert the registry advertises exactly six expected tools.
- Retain existing process and patch argument translation tests.

## Integration and conformance tests

- Run the runtime core test target.
- Run the runtime workspace tests if focused verification passes.

## Regression checks

- Run Rust formatting checks.
- Search current registry and host prompt for removed model tools.
- Run `git diff --check`.

## Manual checks

- Inspect the outgoing schema registry in source.

## Commands and results

- `cargo test --manifest-path agent/Cargo.toml -p suncode-agent tools::tests::built_in_tool_names_match_the_model_contract -- --exact`: passed.
- `cargo test --manifest-path agent/Cargo.toml --workspace`: 35 database, 3 LLM, 35 runtime, and 24 operations tests passed.
- `cargo fmt --manifest-path agent/Cargo.toml --all -- --check`: passed.
- `cargo clippy --manifest-path agent/Cargo.toml --workspace --lib -- -D warnings`: passed.
- Removed-tool registry and host-prompt source scan: no matches.
- `git diff --check`: passed.
- `cargo clippy --manifest-path agent/Cargo.toml --workspace --all-targets -- -D warnings`: blocked by unrelated existing test warnings in `agent/crates/operations/src/git.rs:569` (`len() >= 1`) and `agent/crates/core/src/agent.rs:2126` (discarded enumerate index).

## Residual risks

- Provider request payloads are covered through the shared registry rather than a provider-specific captured request fixture.
- The existing all-target clippy warning remains outside this change; production library targets are clean.
