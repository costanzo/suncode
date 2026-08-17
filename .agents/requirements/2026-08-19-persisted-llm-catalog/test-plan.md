# Test Plan

## Scope

Persisted provider/model catalog, key handling, dynamic LLM routing, and model-aware compaction.

## Unit tests

- Fresh schema seeds six providers and twelve models.
- Provider/model list and update methods preserve custom values.
- API keys are omitted from redacted rows.
- Custom provider rows route through the OpenAI-compatible adapter.
- Context builder honors persisted context and auto-compact thresholds.

## Regression checks

- `cargo test --workspace`.
- `cargo fmt --all -- --check`.
- `cargo clippy -p suncode-llm --all-targets -- -D warnings`.
- `cargo diff --check` equivalent `git diff --check`.

## Residual risks

- Client-facing provider/model CRUD is deferred.

## Commands and results

- `cargo test --workspace`: passed after schema and catalog integration.
- `cargo fmt --all -- --check`: passed after the final source changes.
- `cargo clippy -p suncode-llm --all-targets -- -D warnings`: passed after the final source changes.
- `git diff --check`: passed after the final source changes.
