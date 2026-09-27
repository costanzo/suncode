# Test Plan

## Scope

Durable context reuse, complete tool exchanges, model summary fallback, budgeting, and overflow classification.

## Unit tests

- `cargo test -p suncode-agent context::tests`

## Integration and conformance tests

- `cargo test -p suncode-data context_messages_replay_persisted_compaction_and_new_messages`
- `cargo test -p suncode-agent`
- `cargo test -p suncode-llm`

## Regression checks

- `cargo fmt --all -- --check`
- `git diff --check`

## Manual checks

- Review projected checkpoint payload and confirm public events exclude retained messages.

## Commands and results

- Core (71 tests), LLM (15 tests), and focused data replay tests passed. `cargo fmt --all -- --check` and `git diff --check` passed. The full data suite passed 16 tests and failed one unrelated seeded-model catalog assertion (`deepseek-flash` versus `deepseek-v4-flash-vision-exp`).

## Residual risks

- Token estimation remains approximate across providers; the overflow retry provides a bounded recovery path.
