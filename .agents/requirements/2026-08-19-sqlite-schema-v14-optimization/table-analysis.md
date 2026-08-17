# Table-by-Table Analysis

This analysis describes the version 14 physical model. Rust is the only database owner; the tables are not a client API.

## `schema_migrations`

- **Role:** Records completed schema versions.
- **Shape:** Integer version primary key plus applied timestamp.
- **Assessment:** Correct and minimal. Migration work and version insertion share the database-open transaction, so a partially applied version is not committed.
- **Remaining risk:** Versions 1-9 are compatibility history rather than independently replayable Rust migration functions. The supported upgrade floor remains version 10.

## `audit_records`

- **Role:** Append-only authority and operation audit stream.
- **Shape:** Monotonic integer row ID, optional project/session/turn correlations, event type, timestamp, and valid JSON payload.
- **Indexes:** Global timestamp plus project, session, and turn timelines.
- **Assessment:** Deliberately has no foreign keys so audit history can outlive projections. Update/delete guards are appropriate.
- **Remaining risk:** Runtime coverage of approval decisions and failed outcomes is incomplete; that is a write-path issue, not a table-shape issue.

## `session_content`

- **Role:** Durable ordered source for conversation and lifecycle projections.
- **Shape:** Composite primary key `(session_id, content_sequence)`, timestamp, event type, valid JSON payload.
- **Indexes:** Primary key for replay, `(session_id,event_type,content_sequence)` for event-specific rebuilds, and global occurrence time for future retention.
- **Assessment:** The composite key is the correct ordering authority. No session foreign key is intentional because content can outlive or rebuild projections. Version 14 removes a partial sequence index that duplicated the primary key because ephemeral deltas are not production writes.
- **Remaining risk:** Retention/compaction and replay-gap detection are not implemented yet.

## `session_sequences`

- **Role:** Prevents durable sequence reuse after content deletion or compaction.
- **Shape:** One row per session with the next positive sequence.
- **Assessment:** Necessary even though `MAX(session_content)` could allocate today; it preserves the high-water mark when retained rows disappear. It intentionally has no session foreign key.

## `projects`

- **Role:** Canonical registry of opened directory trees.
- **Shape:** Opaque ID, unique canonical root, display name, lifecycle timestamps, optional archive timestamp.
- **Indexes:** Archive state and last-opened order for the hub.
- **Assessment:** Correct projection. Canonical root is sensitive local metadata and should remain outside provider/audit payloads.

## `sessions`

- **Role:** Conversation metadata and project binding.
- **Shape:** Opaque ID, nullable legacy project FK, title/model, active/archive state, activity timestamps.
- **Indexes:** Project, state, descending activity, then ID for stable sidebar order.
- **Assessment:** Archive-state consistency is enforced with a check. Nullable project ownership is retained only for pre-v6 compatibility.

## `turns`

- **Role:** Current turn state and cumulative usage projection.
- **Shape:** Turn ID, session FK, optional submission key, state/model/timestamps/error, non-negative-by-runtime usage counters.
- **Indexes:** Session chronology and a partial non-terminal recovery index.
- **Assessment:** The per-session submission-key uniqueness is correct. Version 14 adds the recovery index because startup scans non-terminal turns globally.
- **Remaining risk:** Turn terminal events and `turn_submissions` terminal responses are still committed in separate transactions.

## `tool_calls`

- **Role:** Current state of each provider-issued tool call.
- **Shape:** Composite key `(turn_id,tool_call_id)`, turn FK, name/state/order/timestamps/error.
- **Indexes:** Turn plus state.
- **Assessment:** The composite identity correctly avoids assuming provider call IDs are globally unique. Cascade deletion follows the owning turn.

## `approval_requests`

- **Role:** Durable approval request and user decision metadata.
- **Shape:** Approval ID, project/session/turn/tool correlations, operation arguments, globally unique idempotency key, status/decision metadata and timestamps.
- **Indexes:** Session, status, creation time.
- **Assessment:** Arguments remain available for approval review. Correlations are intentionally stored as columns rather than hidden in JSON.
- **Remaining risk:** Correlation IDs are not foreign-keyed, and resolved arguments may retain large write bodies. A later security/retention delivery should define historical argument redaction.

## `suspended_turns`

- **Role:** Single-use continuation payload while an approval is pending or resuming.
- **Shape:** Approval FK/primary key, session/turn correlations, JSON snapshot, status and timestamps.
- **Indexes:** Partial index for startup discovery of `resuming` rows.
- **Assessment:** Pending and resuming rows need the complete snapshot. Version 14 replaces terminal snapshots with `{}` while retaining lifecycle metadata, eliminating the largest avoidable JSON duplication.

## `turn_submissions`

- **Role:** Durable admission and idempotent result record for client submissions.
- **Shape:** Composite key `(session_id,idempotency_key)`, status, response/error JSON, turn/input/model and lifecycle timestamps.
- **Indexes:** Unique session/turn association and global status/admission recovery order.
- **Assessment:** Separating client idempotency from the event projection is sound because admission must survive before the first turn event.
- **Remaining risk:** Pending admissions without a turn event and terminal projection/submission split-brain states need transactional recovery work.

## `checkpoint_manifests`

- **Role:** Turn-level undo unit exposed to clients.
- **Shape:** Manifest ID, session/turn FKs, aggregate restore state and expiry timestamps.
- **Indexes:** Session/status chronology, unique turn ownership, and partial available-expiry scan.
- **Assessment:** One manifest per turn matches the desktop undo model. Version 14 adds the expiry index required by a future global cleanup pass.

## `checkpoints`

- **Role:** Ordered references to opaque operations-owned checkpoint files.
- **Shape:** Checkpoint ID, manifest/session/turn/tool correlations, relative path, state, timestamps and restore ordinal.
- **Indexes:** Manifest and ordinal for reverse restoration.
- **Assessment:** The database stores metadata only; source-code pre/post images remain in operations storage. Composite tool-call FK prevents mismatched call IDs.
- **Remaining risk:** Expired metadata and checkpoint files are not yet invalidated/deleted by a retention worker.

## `session_messages`

- **Role:** Message read model for desktop history and provider context.
- **Shape:** Message ID, session FK, optional turn, unique content sequence, role, message/usage JSON and timestamp.
- **Indexes:** The unique `(session_id,content_sequence)` constraint serves ordered history and context reads.
- **Assessment:** Version 14 removes the unused `(session_id,role,content_sequence)` index because production queries do not filter by role and the unique index already covers ordering.

## `provider_exchanges`

- **Role:** Normalized per-request provider trace projection.
- **Shape:** Exchange/session/turn identity, provider/model route, state/iteration/timestamps, normalized input/output/tool/usage/error JSON.
- **Indexes:** Session chronology, turn chronology, and partial in-flight scan.
- **Assessment:** Normalized fields keep vendor wire types out of clients. Version 14 adds the in-flight index needed to reconcile `started` rows after a restart.
- **Remaining risk:** Full input context is duplicated from `session_content`; list APIs are unpaginated and currently deserialize every trace payload.

## `setting_records`

- **Role:** One normalized store for user, project, and session settings.
- **Shape:** Composite key `(scope,scope_id,key)`, valid JSON value and update timestamp.
- **Assessment:** Scope overlays are resolved in Rust from user to project to session. Version 14 migrates and removes the obsolete user-only table, leaving one source of truth.
- **Remaining risk:** Polymorphic `scope_id` cannot use a simple foreign key; runtime validation remains authoritative.

## `secret_records`

- **Role:** Provider-keyed credential history.
- **Shape:** Secret ID, provider ID, plaintext value, format/version markers, creation and invalidation timestamps.
- **Indexes:** Partial unique index permits only one active row per provider and directly serves credential lookup.
- **Assessment:** Rotation remains append-plus-invalidate and is transactional. Version 14 deterministically invalidates older duplicates before creating the uniqueness constraint.
- **Remaining risk:** Plaintext storage is an accepted decision, but data-directory and SQLite/WAL file permissions still need hardening.

## Removed in Version 14

- **`client_sync`:** Removed because direct SDK subscriptions use caller-supplied cursors and durable `session_content`; no runtime code read or wrote this disposable table.
- **`user_settings`:** Migrated into `setting_records` with newest-timestamp-wins conflict handling, then removed.
