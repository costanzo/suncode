# Client-to-Runtime Contract

Status: Draft for Phase 1 implementation.

Clients connect to the authenticated loopback runtime API when they use the HTTP adapter. They do not access SQLite, provider endpoints, or secrets. The initial server uses authenticated HTTP for requests and snapshots plus server-sent events for ordered session replay/live delivery; WebSocket remains a future transport option. Qt now uses the Rust SDK facade directly, while this contract remains the adapter contract for future compatible transports.

Implemented endpoints:

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/health` | Runtime and database health |
| `GET` | `/diagnostics` | Redacted runtime health, operation recovery, credential status, and active project diagnostics |
| `GET` | `/models` | Advertised model catalog |
| `GET` | `/settings` | Read effective non-secret settings with scope provenance |
| `PUT` | `/settings` | Set one user/project/session setting |
| `GET` | `/credentials` | Read redacted provider credential status |
| `POST` | `/credentials/{provider}` | Store or replace a provider API key in the runtime-owned SQLite store |
| `DELETE` | `/credentials/{provider}` | Remove a provider API key from the runtime-owned SQLite store |
| `GET` | `/projects` | List opened projects |
| `POST` | `/projects` | Canonicalize and open a project through the Rust core |
| `POST` | `/projects/{project_id}/open` | Reopen a known project and bind the core to it |
| `GET` | `/projects/{project_id}/sessions` | List sessions for a project |
| `POST` | `/projects/{project_id}/sessions` | Create a session in the active project |
| `PATCH` | `/sessions/{session_id}` | Rename a session |
| `DELETE` | `/sessions/{session_id}` | Archive a session without deleting history or audit data |
| `POST` | `/sessions/{session_id}/reopen` | Reopen an archived session |
| `GET` | `/sessions/{session_id}/snapshot` | Read a bounded session snapshot for initial load or replay fallback |
| `GET` | `/sessions/{session_id}/checkpoints` | List turn-level undo manifests and status |
| `GET` | `/checkpoints/{manifest_id}` | Inspect ordered checkpoint items |
| `POST` | `/checkpoints/{manifest_id}/restore` | Restore a turn manifest with conflict checks |
| `POST` | `/sessions/{session_id}/turns` | Idempotently submit a turn |
| `POST` | `/sessions/{session_id}/turns/{turn_id}/cancel` | Request cooperative cancellation of a running turn |
| `GET` | `/sessions/{session_id}/events` | Read retained ordered events |
| `GET` | `/sessions/{session_id}/events/stream` | Replay and stream events as SSE |
| `GET` | `/approvals/{approval_id}` | Read approval state |
| `POST` | `/approvals/{approval_id}` | Resolve one pending approval |

## Authentication and scope

Every connection presents the runtime credential. The runtime binds the connection to an authorized project scope. Loopback locality is not authentication. Mutating requests include an idempotency key.

Opening a project sends its path only to the trusted runtime and Rust core. Rust canonicalizes and validates the directory before the runtime creates or refreshes the project projection. A session belongs to exactly one project. Turns are accepted only for active sessions in the project currently selected by the core. Session deletion in Phase 1 is recoverable archive; it does not erase conversation history or immutable audit records.

Settings are resolved from built-in defaults, untrusted project declarations, user settings, and explicit runtime overrides. The Phase 1 client API exposes only runtime-owned user/project/session settings; it never returns API key values. Credential writes return only `{provider, configured}` and use plaintext SQLite secret records. Environment credentials are accepted only in explicit non-interactive mode.

The snapshot response contains the session projection, bounded message projection, retained ordered events, `latest_sequence`, and `replay_available`. The current implementation retains all session events and therefore returns `replay_available: true`; future compaction may return a snapshot with replay unavailable for an older cursor.

## Requests

### Start a turn

```json
{
  "type": "turn.start",
  "request_id": "request-1",
  "project_id": "project-1",
  "session_id": "session-1",
  "idempotency_key": "turn-submit-1",
  "input": {"role": "user", "content": [{"type": "text", "text": "Inspect package.json"}]},
  "model": "deepseek-v4-flash"
}
```

The runtime validates that the model is advertised and configured before admitting the turn. The request never contains an API key.

A turn that reaches an approval gate returns HTTP `202` with `status: "awaiting_approval"`, `turn_id`, `tool_call_id`, and `approval_id`. Its idempotent submission remains pending rather than failed. The durable turn state remains `resolving_calls`; approval is a wait point, not a terminal interruption.

Cancellation is cooperative. The runtime aborts the provider request and records `turn.state=cancelled` with partial messages and completed tool results retained. A cancel request for a turn that is not currently running returns `409`; it never rewrites a completed turn.

### Approve a capability

```json
{
  "type": "approval.decide",
  "request_id": "request-2",
  "project_id": "project-1",
  "session_id": "session-1",
  "approval_id": "approval-1",
  "decision": "allow_once",
  "idempotency_key": "approval-decision-1"
}
```

Phase 1 decisions are `deny` and `allow_once`. Scoped and persistent grants require the later policy-profile delivery and are not accepted by this API.

Approval resolution is single-use. An allow decision starts continuation from the durable suspended-turn snapshot and returns without holding the client request open for the resumed provider turn. The runtime executes the approved call once, resolves any remaining sibling calls from the same assistant message in order, and then calls the provider again. A deny decision records `tool.state=denied`, terminates the turn with `authorization_denied`, and fails the original pending submission. Completion or failure is observed through session events and the idempotent turn submission.

## Events

Events are ordered within a session by `content_sequence`, while audit records use their own stream and identifier. Reconnect uses `session.resume` with a previously observed sequence; the runtime applies the `after` boundary in SQLite before serializing the response. The runtime may return a snapshot followed by events, or `resume_unavailable` when retention prevents replay.

```json
{
  "type": "turn.state",
  "event_id": "event-1",
  "project_id": "project-1",
  "session_id": "session-1",
  "turn_id": "turn-1",
  "content_sequence": 7,
  "state": "calling_model"
}
```

Phase 1 event types include `session.snapshot`, `turn.state`, `assistant.delta`, `tool.requested`, `tool.state`, `approval.requested`, `approval.resolved`, `checkpoint.captured`, `checkpoint.restored`, `usage.updated`, `diagnostic`, and `turn.completed`.

`assistant.delta` is a live-only streaming event. Connected clients may receive it with `content_sequence` 0 so they can render incremental assistant text, but replay and snapshot history do not retain token deltas. The durable `message.assistant` event carries the final assistant text.

Checkpoint capture events carry both a turn-level `manifest_id` and a Rust-owned item `checkpoint_id`. Restore processes available items in reverse capture order. A changed target returns `restore_conflict`; the manifest becomes `conflict` when nothing was restored or `partial` when earlier items already restored. Successful restore events state `external_side_effects_reversed: false`; clients must explain that undo does not reverse pushes, publishes, sent requests, or other external effects. Manifests expire after 30 days by default and clients must not offer undo for `expired`, `restoring`, or `restored` states.

## Errors

Errors are typed and bounded. Important codes are `unauthenticated`, `scope_denied`, `model_unavailable`, `provider_unconfigured`, `approval_required`, `authorization_denied`, `budget_exceeded`, `conflict`, `resume_unavailable`, and `runtime_unavailable`.
