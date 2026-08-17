# Architecture

## Current state

`client_sync` and `user_settings` are present but unused. Scoped settings live in `setting_records`. Completed and failed `suspended_turns` retain complete continuation JSON indefinitely. The event and message primary/unique indexes are duplicated by narrower secondary indexes, while global retention and recovery scans lack purpose-built indexes.

## Proposed design

Schema version 14 is an additive transactional migration plus two safe removals. It migrates legacy settings, drops obsolete tables and redundant indexes, releases terminal snapshots, reconciles duplicate active secrets by retaining the newest row, and creates focused partial indexes. `schema.sql` remains the fresh-database definition and is reformatted by table responsibility.

## Boundaries and dependencies

Rust remains the only SQLite owner. No SDK DTO, C ABI, Avalonia model, provider contract, or operations contract changes.

## Data and control flow

1. The runtime opens SQLite and executes the current bootstrap definitions transactionally.
2. A pre-v14 database runs `migrate_schema_to_v14`.
3. Legacy user settings merge into `setting_records`; the newer `updated_at` value wins.
4. Duplicate active secrets are invalidated except for the newest deterministic row.
5. Terminal suspended snapshots are replaced by `{}` and obsolete objects are dropped.
6. Version 14 is recorded only when every step succeeds.

## Security and failure handling

Pending and resuming snapshots are never cleared. Terminal snapshots are recovery-dead data and may contain prompts and tool arguments, so releasing them reduces sensitive-data exposure. Secret reconciliation invalidates old rows without deleting secret history. The entire upgrade uses the existing open transaction.

## Compatibility and migration

The migration accepts version 13 and earlier schemas supported by the existing migration path. Legacy setting rows are preserved. `client_sync` is disposable and currently has no runtime consumer. Existing SDK contracts do not expose either removed table.

## Risks and rollback

Rolling back the binary after migration leaves a version newer than the old binary supports, which is the existing migration policy. Terminal continuation payloads cannot be reconstructed after release, but terminal rows are never resumed.

## Open questions

- None for v14.
