# Test Plan

## Unit and integration tests

- Fresh schema contains exactly the current 13 tables and expected indexes/triggers.
- Turn usage and recovery snapshot updates target `session_turn`.
- Provider lifecycle events project to `session_call`.
- Tool request/state/result events project request/result JSON and call correlation to `session_tool_use`.
- User, assistant, thinking, and tool messages project to `session_message`.
- Message reads are ordered by `created_at,rowid` and do not require a sequence column.
- Foreign-key checks pass for approvals and checkpoints.
- Session snapshots read normalized messages, and live subscriptions do not replay deleted event-log data.

## Commands

```text
cargo fmt --all -- --check
cargo test --workspace
cargo clippy -p suncode-db --all-targets -- -D warnings
cargo clippy -p suncode-llm --all-targets -- -D warnings
git diff --check
```

## Residual risks

Live event delivery is best-effort. A lagged subscriber must reload a normalized snapshot after `resync.required`.
