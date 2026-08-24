# Test Plan

## Scope

OpenAI-compatible usage parsing, provider-call persistence, and session-stable host context.

## Unit tests

- Normalize nested and top-level cache usage aliases.
- Normalize cache-miss and reasoning-token fields.
- Preserve explicit zero and nullable missing values.
- Verify the host-environment message uses a supplied stable session timestamp.

## Integration and conformance tests

- Verify normalized optional usage is retained in provider exchange storage.
- Update the shared runtime SDK vector with additive usage fields.

## Regression checks

- Run Rust formatting, workspace tests, strict Clippy, JSON validation, and diff checks.

## Manual checks

- Inspect an isolated persisted provider call after focused test execution.

## Commands and results

- `cargo test --manifest-path agent/Cargo.toml --workspace`: passed, 109 unit tests plus doc tests.
- `cargo clippy --manifest-path agent/Cargo.toml -p suncode-llm --all-targets -- -D warnings`: passed.
- `cargo clippy --manifest-path agent/Cargo.toml -p suncode-agent --lib -- -D warnings`: passed.
- `cargo clippy --manifest-path agent/Cargo.toml -p suncode-db --all-targets -- -D warnings`: passed.
- Focused Rustfmt checks for changed source files: passed.
- `jq empty contracts/vectors/provider-normalization.json contracts/vectors/runtime-sdk.json`: passed.
- `git diff --check`: passed.
- Workspace-wide strict Clippy reached a pre-existing `clippy::len_zero` error in `agent/crates/operations/src/git.rs:569`; the affected crates pass their strict checks.

## Residual risks

- Live provider verification depends on external credentials, provider cache eligibility, and provider-side cache lifetime.
- Existing persisted calls cannot recover provider fields that were not normalized at ingestion time.
