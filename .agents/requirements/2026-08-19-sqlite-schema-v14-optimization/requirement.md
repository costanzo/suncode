# Requirement

Status: Superseded by `../2026-08-19-db-module-layout/`. Version tracking, upgrade compatibility, and legacy data conversion described below are not part of the current system.

## Background

The version 13 SQLite schema retains two unused legacy tables, keeps full approval continuation snapshots after they are no longer recoverable work, and lacks indexes for several documented recovery and retention paths. The bootstrap schema is also stored as one statement per line, which makes physical review unnecessarily difficult.

## Goals

- Make the normative schema readable and easier to review.
- Remove storage structures that have no Phase 1 production consumer.
- Release sensitive, high-volume approval continuation data once it is no longer needed.
- Add narrowly justified indexes for active-secret lookup, startup recovery, retention, and checkpoint expiry.
- Preserve all live settings, credentials, sessions, messages, events, approvals, and projections while upgrading an existing database.

## Non-goals

- Redesign event sourcing, provider trace DTOs, or client APIs.
- Implement the broader retention and compaction service.
- Change the accepted plaintext-secret decision.
- Repair turn/submission transaction boundaries or audit completeness in this delivery.

## Requirements

- Schema version 14 removes `client_sync`, which has no runtime reader or writer and is disposable by contract.
- Schema version 14 migrates legacy `user_settings` rows into user-scoped `setting_records` before dropping the legacy table.
- Terminal `suspended_turns` rows retain lifecycle metadata but replace recovery-only `snapshot_json` content with an empty JSON object.
- Future terminal suspended-turn transitions release their snapshot in the same update.
- Only one active `secret_records` row may exist per provider.
- Redundant indexes already covered by primary or unique indexes are removed.
- New recovery and retention indexes must correspond to concrete runtime or contract queries.

## Edge cases

- A version 13 database contains both a legacy and scoped value for the same setting key.
- Historical data contains more than one active provider secret.
- Pending or resuming approvals must retain their continuation snapshots.
- A newly created empty database must reach the same version 14 shape as an upgraded database.

## Acceptance criteria

- Focused tests cover legacy settings migration, terminal snapshot release, active-secret uniqueness, legacy-table removal, and index shape.
- Existing credential, approval, session, provider trace, and SDK tests pass.
- `cargo test --workspace`, formatting, `git diff --check`, SQLite integrity, and foreign-key checks pass.

## Open questions

- Provider trace bounding and full retention execution remain follow-up work.
