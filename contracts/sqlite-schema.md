# Runtime SQLite Schema

Status: Phase 1 contract, schema version 12.

The embedded Rust runtime is the only component that opens this database. Host bindings, providers, and future extensions never read or write these tables directly. Host-visible shapes are defined by the runtime SDK contract and may differ from this physical schema.

## Conventions

- SQLite foreign keys are enabled for every connection.
- Migrations are numbered, append-only, and applied transactionally through `schema_migrations`.
- Identifiers are opaque non-empty text values. Runtime-created identifiers use UUIDs; clients must not infer their format.
- Timestamps are UTC RFC 3339 strings produced by `Date.toISOString()`.
- JSON columns contain UTF-8 JSON objects or values serialized by the runtime. JSON is not used for columns that need filtering, ordering, uniqueness, or referential checks.
- Projects and sessions are archived, not normally hard-deleted.
- `session_content` is the rebuild source for durable query projections. High-volume streaming deltas are ephemeral runtime events and are not retained.

## Migration metadata

### `schema_migrations`

| Column | Type | Constraints |
| --- | --- | --- |
| `version` | INTEGER | Primary key |
| `applied_at` | TEXT | Not null |

Version 6 introduces the project, session, turn, tool-call, and checkpoint item projections. Version 7 adds the message projection and durable turn admission fields. Version 8 adds turn-level checkpoint manifests, expiry, aggregate restore state, and ordered checkpoint items. Version 9 adds scoped user/project/session settings. Version 10 makes streaming deltas ephemeral, adds a per-session content sequence high-water table, and extends `session_messages` to include tool messages for context rebuilds. Version 11 stores provider secrets as plaintext SQLite values and removes the old ciphertext/nonce columns. Earlier migrations remain in code for existing databases.

## Query projections

### `projects`

One row per opened local directory tree.

| Column | Type | Constraints |
| --- | --- | --- |
| `project_id` | TEXT | Primary key |
| `canonical_root` | TEXT | Not null, unique |
| `display_name` | TEXT | Not null |
| `created_at` | TEXT | Not null |
| `updated_at` | TEXT | Not null |
| `last_opened_at` | TEXT | Not null |
| `archived_at` | TEXT | Nullable |

`canonical_root` is an absolute, runtime-normalized host path. It is local durable state and must not be copied to provider requests, audit payloads, or logs without an explicit need.

### `sessions`

One row per conversation. `project_id` is nullable only so pre-v6 content can be represented before the user reopens and binds its project; newly created Phase 1 sessions require a project.

| Column | Type | Constraints |
| --- | --- | --- |
| `session_id` | TEXT | Primary key |
| `project_id` | TEXT | Nullable FK to `projects`, delete restricted |
| `title` | TEXT | Nullable |
| `model_id` | TEXT | Nullable |
| `status` | TEXT | `active` or `archived` |
| `created_at` | TEXT | Not null |
| `updated_at` | TEXT | Not null |
| `last_activity_at` | TEXT | Not null |
| `archived_at` | TEXT | Nullable and consistent with `status` |

The primary sidebar query is indexed by `project_id`, `status`, and descending `last_activity_at`.

### `turns`

Current turn state derived from `turn.state` content events. Prompts and responses remain in `session_content` and are not duplicated here.

| Column | Type | Constraints |
| --- | --- | --- |
| `turn_id` | TEXT | Primary key |
| `session_id` | TEXT | Not null FK to `sessions`, cascade delete |
| `submission_idempotency_key` | TEXT | Nullable, unique within a session when present |
| `state` | TEXT | One Phase 1 turn state |
| `model_id` | TEXT | Nullable |
| `created_at` | TEXT | Not null |
| `updated_at` | TEXT | Not null |
| `completed_at` | TEXT | Nullable; set for terminal states |
| `error_code` | TEXT | Nullable |
| `input_tokens` | INTEGER | Latest cumulative provider-reported input usage for the turn; defaults to zero |
| `output_tokens` | INTEGER | Latest cumulative provider-reported output usage for the turn; defaults to zero |
| `total_tokens` | INTEGER | Latest cumulative provider-reported total usage for the turn; defaults to zero |

Turn states are `admitted`, `queued`, `preparing`, `calling_model`, `resolving_calls`, `compacting`, `completed`, `failed`, `cancelled`, and `interrupted`.

Schema version 12 backfills these counters from the latest retained usage-bearing event for each turn. A session aggregate sums the turn projection; repeated cumulative events for one turn replace its counters and are never added together.

### `tool_calls`

Current child tool-call state derived from `tool.state` events.

| Column | Type | Constraints |
| --- | --- | --- |
| `turn_id` | TEXT | Part of primary key; FK to `turns`, cascade delete |
| `tool_call_id` | TEXT | Part of primary key |
| `name` | TEXT | Not null |
| `state` | TEXT | One Phase 1 tool-call state |
| `ordinal` | INTEGER | Nullable, non-negative when present |
| `created_at` | TEXT | Not null |
| `updated_at` | TEXT | Not null |
| `completed_at` | TEXT | Nullable; set for terminal states |
| `error_code` | TEXT | Nullable |

The composite key avoids assuming that provider-issued call IDs are globally unique. Tool-call states are `requested`, `validating`, `policy_check`, `denied`, `awaiting_approval`, `authorized`, `executing`, `succeeded`, `failed`, `timed_out`, `unknown_completion`, and `reconciling`.

### `checkpoint_manifests`

One row per turn that changed files. The manifest is the client-visible undo unit; Rust remains the owner of each opaque file snapshot.

| Column | Type | Constraints |
| --- | --- | --- |
| `manifest_id` | TEXT | Primary key |
| `session_id` | TEXT | Not null FK to `sessions`, cascade delete |
| `turn_id` | TEXT | Nullable FK to `turns`, unique when present |
| `status` | TEXT | `available`, `restoring`, `restored`, `partial`, `conflict`, or `expired` |
| `created_at` | TEXT | Not null |
| `updated_at` | TEXT | Not null |
| `expires_at` | TEXT | Not null |
| `restored_at` | TEXT | Nullable |

New manifests expire after 30 days in Phase 1. Expiry invalidates remaining item references but does not delete session or audit history. `partial` means some items restored before another item conflicted; `conflict` means no item was restored.

### `checkpoints`

Runtime projection of opaque Rust checkpoint references.

| Column | Type | Constraints |
| --- | --- | --- |
| `checkpoint_id` | TEXT | Primary key |
| `manifest_id` | TEXT | FK to `checkpoint_manifests` |
| `session_id` | TEXT | Not null FK to `sessions`, cascade delete |
| `turn_id` | TEXT | Nullable |
| `tool_call_id` | TEXT | Nullable |
| `relative_path` | TEXT | Nullable |
| `status` | TEXT | `available`, `restored`, or `invalidated` |
| `created_at` | TEXT | Not null |
| `restored_at` | TEXT | Nullable |
| `invalidated_at` | TEXT | Nullable |
| `ordinal` | INTEGER | Non-negative restore order |

The checkpoint payload remains owned by the runtime operations module. Items restore in reverse ordinal order so several writes to the same path return to the pre-turn image.

### `session_messages`

Message-level read projection for desktop history, context construction, and cursor pagination. It is projected from durable message events and does not store streaming deltas.

| Column | Type | Constraints |
| --- | --- | --- |
| `message_id` | TEXT | Primary key |
| `session_id` | TEXT | Not null FK to `sessions`, cascade delete |
| `turn_id` | TEXT | Nullable |
| `content_sequence` | INTEGER | Not null, unique within a session |
| `role` | TEXT | `user`, `assistant`, or `tool` in Phase 1 |
| `message_json` | TEXT | Not null, valid JSON |
| `usage_json` | TEXT | Nullable, valid JSON when present |
| `created_at` | TEXT | Not null |

The primary history query uses (`session_id`, `content_sequence`). A deterministic `event:<session_id>:<sequence>` ID is used when migrating old events or projecting tool messages that do not carry a message ID; new user and assistant runtime events carry UUID message IDs. Client conversation history normally shows user and assistant rows, while tool rows are retained for provider context and diagnostics.

## Durable streams and state

### `audit_records`

Immutable authority and operation records. Columns are `id`, optional correlation IDs (`project_id`, `session_id`, `turn_id`), `occurred_at`, `event_type`, and `payload_json`. Update and delete triggers abort every mutation. Indexes support project, session, and turn timelines. Audit rows intentionally do not foreign-key to projections because their retention and immutability outlive mutable projections.

### `session_content`

Compactable durable content stream. Its primary key is (`session_id`, `content_sequence`); the sequence is allocated transactionally per session. Other columns are `occurred_at`, `event_type`, and `payload_json`. It intentionally does not foreign-key to `sessions`, allowing independent compaction and projection rebuild.

`assistant.delta`, `reasoning.delta`, and `tool.input.delta` are explicitly ephemeral. They may be broadcast to connected clients with `content_sequence` 0, but they are not retained in `session_content` and are not returned by historical replay APIs. The final `message.assistant` event stores the complete assistant message.

### `session_sequences`

Per-session durable event sequence high-water marks.

| Column | Type | Constraints |
| --- | --- | --- |
| `session_id` | TEXT | Primary key |
| `next_content_sequence` | INTEGER | Not null, greater than 0 |

This table prevents sequence reuse after compaction or deletion of ephemeral legacy rows. Runtime append transactions reserve and advance one sequence before inserting the durable event.

### `client_sync`

Disposable reconnect cursors keyed by (`connection_id`, `session_id`) with `content_sequence` and `expires_at`. Expiration is indexed. This table is never authoritative.

### `approval_requests`

Durable approval state keyed by `approval_id`. It carries optional `project_id`, required session/turn/tool-call correlation IDs, `operation`, `arguments_json`, unique `idempotency_key`, status (`pending`, `approved`, or `denied`), optional decision metadata, and creation/update timestamps.

### `turn_submissions`

Durable turn admission and idempotent client submissions keyed by (`session_id`, `idempotency_key`). Existing rows remain compatible with the legacy nullable fields; new rows record `turn_id`, `input_json`, `model_id`, `admitted_at`, `started_at`, and `completed_at`. Status is `pending`, `completed`, or `failed`; terminal response/error JSON and creation/update timestamps are stored with it. A retry with the same key but a different input or model is rejected.

### `suspended_turns`

Approval continuation snapshots keyed by and foreign-keyed to `approval_requests.approval_id`. It carries session and turn IDs, `snapshot_json`, status (`pending`, `resuming`, `completed`, `denied`, or `failed`), and timestamps.

### `user_settings`

Settings keyed by `key`, with `value_json` and `updated_at`. Settings exposed to a client are mapped through an API DTO rather than returning this table.

### `secret_records`

Plaintext provider secret records keyed by `secret_id`, with `provider_id`, plaintext value, algorithm marker, key version, creation time, and optional invalidation time. The runtime keeps one active record per provider and invalidates the previous row on rotation. The SQLite database and its backups are sensitive.

## Projection updates and recovery

Appending a retained content event and updating its affected projection happen in one SQLite transaction. Unknown event types remain valid content and do not mutate projections. Invalid projection fields likewise leave the event available for diagnostics instead of introducing an invalid projection row.

Migration 6 backfills unbound sessions and the latest turn, tool-call, and checkpoint states from retained pre-v6 content. Startup recovery may repeat that derivation when projection integrity is uncertain. Audit data is not a projection source.

Migration 7 backfills message rows from retained `message.user` and `message.assistant` events and leaves pre-v7 turn submissions with nullable admission fields. Migration 8 groups historical checkpoint items by turn into legacy manifests. Migration 10 records per-session sequence high-water marks, removes legacy retained streaming deltas, rebuilds the message projection with tool-role support, and backfills tool messages from retained `message.tool` events. Pending submissions with a stored input are the durable recovery queue; execution recovery remains a separate policy decision and must not blindly replay an unknown provider call.

Hard deletion, retention, and compaction must respect `contracts/persistence.md`. Managed artifacts are deferred until the runtime artifact primitive exists; no speculative Phase 1 artifact table is defined here.
