# Test Plan

## Unit tests

- Global, project, and session values are stored in one table with valid ownership.
- Effective configuration resolves global, project, and session precedence.
- Non-string project default model values are rejected by the typed lookup.
- Session creation uses a configured project default model when no explicit model is supplied.
- Foreign-key and JSON constraints remain valid on a fresh schema.

## Regression checks

- `cargo test --workspace`
- `cargo fmt --all -- --check`
- `cargo clippy -p suncode-db --all-targets -- -D warnings`
- `cargo clippy -p suncode-llm --all-targets -- -D warnings`
- `git diff --check`

## Results

- Workspace tests: passed, 56 tests.
- Formatting check: passed.
- `suncode-db` and `suncode-llm` strict Clippy: passed.
- Runtime library strict Clippy: passed. Full runtime all-target Clippy still reports the pre-existing `unused_enumerate_index` warning in the test helper.
- Diff check: passed.
