# Changes

## Source

- Added durable `usage.updated` projection into the owning `turns` row.
- Added session-wide usage aggregation and the typed `RuntimeSdk::session_usage` method.
- Added the named `suncode_runtime_sdk_session_usage` C ABI function.
- Added the Qt `sessionTotalTokens` property with session-switch race protection and live refresh.
- Added the compact footer value beside the selected model.

## Contracts and generated artifacts

- Added `session_usage` to the runtime SDK contract and shared vector.
- Documented cumulative per-turn and session aggregation semantics.

## Configuration and persistence

- Added schema version 12 migration to backfill turn counters from the latest retained usage-bearing event.
- Providers that omit usage continue to contribute zero; no estimate is synthesized.

## Tests

- Added persistence tests for cumulative replacement, cross-turn aggregation, and v11-to-v12 backfill.
- Extended SDK and C ABI tests for the named session usage method.
- Passed all 37 Rust workspace tests, Rust formatting, Qt build, QML lint, contract JSON, ABI symbol, and diff checks.
- Visually verified the real project-window QML at 1440x900 and the 900x620 minimum size with a 13.8k-token fixture.

## Documentation

- Updated architecture, decisions, runtime specification, feature records, desktop README, SQLite schema, and SDK contract.
