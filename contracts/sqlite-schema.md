# Runtime SQLite Schema

Status: Current Phase 1 contract.

The Rust `suncode-db` package is the only database owner. The schema is fresh-schema-only: there is one current table set, no version table, and no migration runner. `runtime/crates/db/src/schema/mod.rs` applies one table-owned SQL resource per manifest entry; `runtime/crates/db/src/data/mod.rs` separately applies idempotent provider/model seed data. File names do not encode execution order. Opening a database with any unexpected application table fails without conversion.

There are 13 application tables:

`approval_request`, `audit_record`, `checkpoint`, `checkpoint_manifest`, `configuration`, `llm_model`, `llm_model_provider`, `projects`, `session`, `session_call`, `session_message`, `session_tool_use`, and `session_turn`.

## Conventions

- IDs are non-empty opaque text; runtime IDs are UUIDs.
- Timestamps are UTC RFC 3339 strings with millisecond precision.
- JSON columns must contain valid JSON. Queryable identity, state, ordering, and ownership remain relational columns.
- Foreign keys are enabled on every connection. Projects and sessions are archived rather than normally deleted.

## Projects And Settings

### `projects`

One row per opened local directory tree: `project_id` primary key, unique non-empty `canonical_root`, non-empty `display_name`, lifecycle timestamps, and nullable `archived_at`.

### `configuration`

Unified key/value configuration for `global`, `project`, and `session` scopes. Global rows have no owner ID; project rows reference `projects`; session rows reference `session`. A CHECK constraint enforces the exact owner shape for each scope, and partial unique indexes enforce one value per key at each scope. Effective reads apply `global < project < session` precedence. The initial project-aware key is `default_model`, stored as a JSON string containing a model ID.

## Sessions

### `session`

One conversation per row, linked to a project. It stores optional title/model, `active`/`archived` status, activity timestamps, and archive consistency checks.

### `session_turn`

The single source of truth for a turn. It combines turn lifecycle, submission idempotency, input/response/error JSON, model selection, cumulative provider usage, and approval continuation state. States are `admitted`, `queued`, `preparing`, `calling_model`, `resolving_calls`, `compacting`, `completed`, `failed`, `cancelled`, and `interrupted`.

Approval recovery is kept on the turn in `recovery_approval_id`, `recovery_snapshot_json`, `recovery_status`, and recovery timestamps; there is no suspended-turn table. The unique `(session_id, submission_idempotency_key)` constraint makes retries idempotent. Recovery, resuming, and session chronology indexes support startup and history queries.

### `session_call`

One row per LLM request within a turn. `call_id` is the physical primary key and is linked to `session_turn`; provider/model/wire-model identity, iteration, lifecycle state, normalized input/output/tool-call/usage/error JSON, finish reason, and timestamps are retained. States are `started`, `completed`, and `failed`. Session, turn, and in-flight indexes support diagnostics and recovery.

### `session_tool_use`

One row per tool invocation, keyed by `(turn_id, tool_call_id)`. It records the owning `session_call_id`, tool name, request JSON, result JSON, lifecycle state, ordinal, timestamps, and error code. States are `requested`, `validating`, `policy_check`, `denied`, `awaiting_approval`, `authorized`, `executing`, `succeeded`, `failed`, `timed_out`, `unknown_completion`, and `reconciling`.

### `session_message`

Human-readable and provider-context messages keyed by `message_id`. Each row links to its session, optionally to a turn and `session_call`, and stores role, message JSON, optional usage JSON, and `created_at`. Roles are `user`, `assistant`, `thinking`, and `tool`. Message history and context are ordered by `created_at` with `rowid` as a deterministic tie-breaker; no content sequence column is used.

### `audit_record`

Immutable authority and operation history with optional project/session/turn correlations, event type, timestamp, and valid payload JSON. Update/delete triggers reject mutations.

## Approvals And Checkpoints

### `approval_request`

Durable approval state keyed by `approval_id`, linked to session and the composite `(turn_id, tool_call_id)` in `session_tool_use`. It stores operation, arguments JSON, idempotency key, decision metadata, status, and timestamps.

### `checkpoint_manifest`

One client-visible undo unit per turn, linked to session and optionally to `session_turn`, with lifecycle status, expiry, and restore timestamps.

### `checkpoint`

Ordered checkpoint metadata linked to a manifest/session and optionally to the composite `(turn_id, tool_call_id)` in `session_tool_use`. Source pre/post images remain owned by the operations package.

## LLM Catalog

### `llm_model_provider`

Built-in or custom provider row keyed by `provider_id`. It stores display name, endpoint, required `adapter_type`, optional plaintext `api_key`, enabled/order state, and timestamps. `adapter_type` names a provider implementation known to `suncode-llm`; the current supported value is `openai`, which means the endpoint implements the OpenAI-compatible protocol. API keys never enter DTOs, events, audit payloads, or logs.

### `llm_model`

Model row keyed by `model_id` and linked to `llm_model_provider`. It stores display/request identifiers, context and auto-compaction token limits, optional output limit, capability flags, enabled/order state, and timestamps. `auto_compact_tokens` is positive and smaller than `context_tokens`.

## Projection Rules

Durable projection updates occur in one transaction per runtime event. The runtime rebuilds provider context from `session_message`, repairing incomplete assistant/tool tails after interruption. Runtime events are broadcast in memory only; a client that misses events receives `resync.required` and reloads a fresh session snapshot. Audit rows remain immutable and independent from session projections.
