# Current Table-by-Table Analysis

> Historical record. The table inventory below describes the pre-catalog 16-table baseline; the current schema also includes `llm_model_provider` and `llm_model`.

Rust is the only database owner. The current schema has 16 product tables and no migration metadata.

## `audit_records`

- **Purpose:** Immutable authority and operation audit history.
- **Design:** Integer append order, optional correlation IDs, timestamp, event type, and validated JSON payload. Correlations intentionally have no foreign keys so audit can outlive projections.
- **Optimization:** Update/delete triggers enforce immutability. Global, project, session, and turn time indexes serve audit timelines.
- **Residual risk:** Audit write-path completeness is separate from physical schema design.

## `session_content`

- **Purpose:** Ordered durable source for conversation and lifecycle projections.
- **Design:** Composite primary key `(session_id, content_sequence)` with timestamp, event type, and validated JSON payload; no session FK so compaction/projection rebuilding remain independent.
- **Optimization:** The primary key serves replay. Event-type and global retention indexes cover other query shapes without a duplicate sequence index.
- **Residual risk:** Retention execution and replay-gap signaling are not implemented here.

## `session_sequences`

- **Purpose:** Preserve each session's next durable content sequence after compaction.
- **Design:** One positive high-water value per non-empty session ID.
- **Optimization:** Avoids `MAX()` allocation and prevents sequence reuse.

## `projects`

- **Purpose:** Registry of opened local directory trees.
- **Design:** Opaque ID, unique canonical root, display name, lifecycle timestamps, and recoverable archive timestamp.
- **Optimization:** One covering order index supports active/archive filtering and recently opened display.

## `sessions`

- **Purpose:** Conversation metadata bound to one project.
- **Design:** Required project FK, title/model, active/archive state, and activity timestamps. A check keeps archive status and timestamp consistent.
- **Optimization:** Removed the old nullable-project compatibility shape. The project/activity index supports stable sidebar queries.

## `turns`

- **Purpose:** Current turn lifecycle and cumulative provider usage projection.
- **Design:** Session FK, optional submission key, constrained state, lifecycle/error fields, and non-negative token counters.
- **Optimization:** Session chronology and partial non-terminal recovery indexes avoid indexing completed history for startup scans.
- **Residual risk:** Turn events and submission terminal results still use separate transactions.

## `tool_calls`

- **Purpose:** Current child tool-call state.
- **Design:** Composite `(turn_id, tool_call_id)` identity, owning turn FK, constrained state, ordinal, timestamps, and error.
- **Optimization:** Composite identity does not assume provider call IDs are globally unique; cascade cleanup follows the turn.

## `approval_requests`

- **Purpose:** Durable approval request and decision metadata.
- **Design:** Project/session references, composite tool-call FK, operation arguments, unique idempotency key, status/decision data, and timestamps.
- **Optimization:** Enforced correlations prevent approvals for nonexistent runtime work. Session/status/time index serves pending and history queries.
- **Residual risk:** Historical arguments may retain large or sensitive write content; redaction policy is separate work.

## `suspended_turns`

- **Purpose:** Single-use continuation while approval is pending or resuming.
- **Design:** Approval primary/FK plus session and turn FKs, validated snapshot, constrained state, and timestamps.
- **Optimization:** A partial `resuming` index serves recovery. Terminal updates replace the recovery payload with `{}`.

## `turn_submissions`

- **Purpose:** Durable admission and idempotent client result record.
- **Design:** Session FK and composite `(session_id, idempotency_key)` key with input/model, lifecycle, response, and error fields.
- **Optimization:** Unique session/turn association prevents duplicate admission mapping; recovery order is indexed. `turn_id` is not foreign-keyed because admission is written before the turn projection exists.
- **Residual risk:** Submission and turn terminal states are not yet one atomic projection update.

## `checkpoint_manifests`

- **Purpose:** One client-visible undo unit per turn.
- **Design:** Session/turn FKs, constrained aggregate restore state, expiry, and restore timestamps.
- **Optimization:** Unique non-null turn ownership, session history, and partial available-expiry indexes match runtime queries.

## `checkpoints`

- **Purpose:** Ordered metadata for operations-owned checkpoint payloads.
- **Design:** Manifest/session ownership, optional composite tool-call correlation, relative path, state, timestamps, and ordinal.
- **Optimization:** Manifest/ordinal index serves reverse restoration; cascade behavior removes metadata with its owner.
- **Residual risk:** File payload expiry and deletion remain operations work.

## `session_messages`

- **Purpose:** Message read model for history and provider context.
- **Design:** Session FK, optional turn correlation, unique content sequence, constrained role, validated message/usage JSON, and timestamp.
- **Optimization:** The unique `(session_id, content_sequence)` index already serves ordered reads, so no redundant role index is kept.

## `provider_exchanges`

- **Purpose:** Normalized per-request provider diagnostics.
- **Design:** Session and turn FKs, stable provider/model route, state/iteration/timestamps, and validated normalized input/output/tool/usage/error JSON.
- **Optimization:** Session and turn chronology indexes plus a partial in-flight index cover inspection and recovery.
- **Residual risk:** Full input context can be large; pagination and retention limits are separate work.

## `setting_records`

- **Purpose:** Single settings source for user, project, and session scopes.
- **Design:** Composite `(scope, scope_id, key)` key with validated JSON value and update time.
- **Optimization:** One normalized table replaces overlapping user-only storage. Polymorphic scope ownership remains runtime-validated.

## `secret_records`

- **Purpose:** Provider credential history.
- **Design:** Non-empty ID/provider/plaintext/algorithm fields, non-negative format version, creation time, and optional invalidation time.
- **Optimization:** A partial unique index both enforces and efficiently looks up one active credential per provider.
- **Residual risk:** Plaintext storage is accepted, so data-directory and backup permissions remain security-critical.
