# Test Plan

## Unit and integration tests

- Fresh schema contains exactly the current 13 tables and expected indexes/triggers.
- Turn usage and recovery snapshot updates target `session_turn`.
- Provider lifecycle events project to `session_call`.
- Provider HTTP request IDs and response-object IDs parse and persist independently when present.
- Tool request/state/result events project request/result JSON and call correlation to `session_tool_use`.
- User, assistant, and thinking messages project to `session_message`; tool messages do not.
- The `session_message` role constraint rejects `tool`.
- The `session_message` schema has no `usage_json`; usage remains available from `session_call` and `session_turn`.
- Completed tool results reconstruct transient provider-context messages from `session_tool_use`, while incomplete tool exchanges remain excluded.
- Message reads are ordered by `created_at,rowid` and do not require a sequence column.
- Foreign-key checks pass for approvals and checkpoints.
- Session snapshots read only normalized human-readable messages, and live subscriptions do not replay deleted event-log data.

## Commands

```text
cargo fmt --all -- --check
cargo test --workspace
cargo clippy -p suncode-db --all-targets -- -D warnings
cargo clippy -p suncode-runtime --lib -- -D warnings
dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj --no-restore -c Release
dotnet test apps/desktop-avalonia-tests/SunCode.Desktop.Tests.csproj --no-restore
git diff --check
```

## Current results

- `cargo fmt --manifest-path runtime/Cargo.toml --all -- --check`: passed.
- `cargo test --manifest-path runtime/Cargo.toml --workspace`: passed (74 tests plus doc tests).
- `cargo clippy --manifest-path runtime/Cargo.toml -p suncode-db --all-targets -- -D warnings`: passed.
- `cargo clippy --manifest-path runtime/Cargo.toml -p suncode-runtime --lib -- -D warnings`: passed.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj --no-restore -c Release`: passed with no warnings.
- `dotnet test apps/desktop-avalonia-tests/SunCode.Desktop.Tests.csproj --no-restore`: passed (21 tests).
- `git diff --check`: passed.

## Residual risks

Live event delivery is best-effort. A lagged subscriber must reload a normalized snapshot after `resync.required`.
