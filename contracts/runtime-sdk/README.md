# Embedded Runtime SDK Contract

Status: Implemented.

The SunCode runtime is a native library embedded in its host process. It does not accept inbound HTTP, expose REST paths, publish a loopback endpoint, or support cross-process attach. The .NET Avalonia client calls the stable C ABI through P/Invoke. Future TypeScript and Python packages wrap the same Rust SDK through native bindings.

Provider adapters may make outbound HTTPS requests to configured model providers. That network behavior is internal to the Rust runtime and is not a client transport.

## Lifecycle

`open_default` loads configuration, acquires the data-directory lock, opens and migrates SQLite, initializes operations and providers, performs recovery, and returns an opaque runtime handle. A second process opening the same data directory receives `runtime_already_active`.

The runtime handle owns the Tokio runtime and all runtime services. Host wrappers may share one handle inside a process. Subscriptions must be closed before the final runtime handle is released. Closing a subscription stops callback delivery before returning.

The C ABI exposes `suncode_runtime_sdk_abi_version`. ABI functions and enum-like integer values are add-only within a major ABI version. Rust layouts, references, strings, vectors, and errors never cross the ABI directly.

## Methods

The Rust API uses typed inputs and outputs. The C ABI exposes one named function per operation. Complex evolving results may be returned as method-specific UTF-8 JSON payloads owned by the SDK; this is DTO serialization, not generic routing.

| SDK method | Purpose |
| --- | --- |
| `health` | Read runtime and database health |
| `diagnostics` | Read redacted runtime, recovery, credential, and active-project diagnostics |
| `list_models` | List stable models and credential-derived availability |
| `list_settings` | Read effective non-secret settings with scope provenance |
| `set_setting` | Store one user, project, or session setting |
| `list_credentials` | Read redacted provider credential status |
| `set_credential` | Store or replace one provider API key |
| `remove_credential` | Remove one provider API key |
| `list_projects` | List known active projects |
| `open_project` | Canonicalize and open a project |
| `select_project` | Select a known project and reopen its canonical root |
| `git_status` | Read the bounded Git index/worktree status and aggregate change counts for a project |
| `git_diff_file` | Read one bounded structured file diff for the all, staged, or unstaged scope |
| `list_sessions` | List active sessions in a project |
| `create_session` | Create a session with an optional title and selected model |
| `rename_session` | Rename a session |
| `archive_session` | Recoverably archive a session |
| `reopen_session` | Reopen an archived session |
| `session_snapshot` | Read a bounded session projection and retained events after a cursor |
| `session_usage` | Read cumulative provider-reported token usage for a session |
| `list_checkpoints` | List turn-level checkpoint manifests for a session |
| `checkpoint_manifest` | Inspect one manifest and its items |
| `restore_checkpoint` | Restore a manifest with ownership and post-image conflict checks |
| `submit_turn` | Idempotently submit input to a session and selected model |
| `cancel_turn` | Cooperatively cancel a running turn |
| `get_approval` | Read one approval state |
| `resolve_approval` | Resolve one pending approval with `allow_once` or `deny` |
| `subscribe_session` | Replay retained events after a cursor and deliver subsequent live events |

Rust-generated project, session, turn, approval, checkpoint, event, and message identifiers remain authoritative. Hosts do not manufacture IDs except idempotency keys.

Git DTOs contain only opened-project-relative paths. `git_status` returns branch and detached-head information, aggregate file/addition/deletion/conflict counts, and per-file index/worktree status. `git_diff_file` returns structured hunks and lines with old/new line numbers plus a bounded plain-text patch. The runtime embeds vendored libgit2 and does not require a Git executable or a system libgit2 installation. The current Git SDK surface is read-only.

`session_usage` returns `input_tokens`, `output_tokens`, and `total_tokens` summed from the latest cumulative usage projection for every turn in the session. Providers that omit usage metadata contribute zero; the runtime does not estimate missing usage.

## Outcomes

Operation results do not encode HTTP statuses. A call either returns its method-specific success type or `SdkError`.

Turn submission returns a tagged outcome:

- `completed`: the admitted turn completed before the call returned;
- `awaiting_approval`: execution is suspended at a durable approval gate;
- `queued`: input was accepted as an in-memory continuation of the active turn.

Cancellation returns `cancellation_requested`; cancelling a turn that is not active returns `conflict`.

## Errors

An SDK error contains:

```json
{
  "code": "session_not_found",
  "message": "session not found",
  "details": {}
}
```

Messages and details are bounded and redacted. Important codes include `invalid_arguments`, `runtime_already_active`, `runtime_unavailable`, `project_not_found`, `session_not_found`, `model_unavailable`, `provider_unconfigured`, `approval_required`, `authorization_denied`, `checkpoint_unavailable`, `restore_conflict`, `conflict`, `scope_denied`, `not_git_repository`, `unsupported_git_repository`, `git_read_failed`, `git_diff_not_found`, `budget_exceeded`, `cancelled`, and `resync_required`.

Panics are contained at native binding boundaries and converted to `runtime_unavailable`; they never unwind into a host language.

## Events

Session events use a strictly increasing durable `content_sequence`. Streaming deltas are live-only and may carry sequence zero. Final messages and lifecycle events are durable.

Subscription establishment follows this invariant:

1. register for live events;
2. read retained events after the supplied cursor;
3. deliver replay in sequence order;
4. deliver buffered and subsequent live events, discarding durable sequences already replayed.

This prevents a replay-to-live loss window. A lagged receiver must recover from SQLite after its last delivered durable sequence. If retained history cannot satisfy recovery, the subscription reports `resync_required`; it never silently drops events.

Callbacks run on an SDK-owned thread. Hosts must copy the callback payload and marshal delivery to their runtime thread: Avalonia uses `Dispatcher.UIThread`, Node.js uses a thread-safe function, and Python acquires the GIL and schedules on the target event loop. Callback payload memory is valid only for the duration of the callback unless copied by the host.

## Authority and secrets

Embedding removes transport authentication because the host is inside the runtime process trust boundary. It does not remove project/session ownership checks, policy evaluation, approval, operation auditing, canonical path validation, checkpoint conflict checks, or credential redaction.

Provider API keys remain Rust-owned plaintext SQLite secrets. Their values never appear in SDK results, events, diagnostics, audit records, or logs.

## Language bindings

Avalonia uses a hand-written C# P/Invoke wrapper over the C ABI and keeps native calls off the UI thread. The Rust crate emits a `cdylib` beside the managed executable. Future TypeScript and Python SDKs expose idiomatic async APIs over the same Rust methods and subscription semantics. They do not open SQLite, call providers, or implement agent behavior independently.
