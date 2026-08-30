# Embedded Agent SDK Contract

Status: Implemented.

The SunCode agent is a native library embedded in its host process. It does not accept inbound HTTP, expose REST paths, publish a loopback endpoint, or support cross-process attach. The .NET Avalonia client calls the stable C ABI through P/Invoke. Future TypeScript and Python packages wrap the same Rust SDK through native bindings.

Provider adapters may make outbound HTTPS requests to configured model providers. That network behavior is internal to the Rust agent and is not a client transport.

## Lifecycle

`open_default` loads configuration, acquires the data-directory lock, opens and initializes the current SQLite schema, initializes operations and providers, performs recovery, and returns an opaque agent handle. A second process opening the same data directory receives `agent_already_active`. An incompatible database is rejected; the agent does not migrate it.

The agent handle owns the Tokio runtime and all agent services. Host wrappers may share one handle inside a process. Subscriptions must be closed before the final agent handle is released. Closing a subscription stops callback delivery before returning.

The C ABI exposes `suncode_agent_sdk_abi_version` and reports ABI version 4. Hosts use the current `agent` symbol family directly; there is no compatibility layer for prior native APIs. ABI functions and enum-like integer values are add-only within a major ABI version. Rust layouts, references, strings, vectors, and errors never cross the ABI directly.

## Methods

The Rust API uses typed inputs and outputs. The C ABI exposes one named function per operation. Complex evolving results may be returned as method-specific UTF-8 JSON payloads owned by the SDK; this is DTO serialization, not generic routing.

| SDK method | Purpose |
| --- | --- |
| `health` | Read agent and database health |
| `diagnostics` | Read redacted agent, recovery, credential, and active-project diagnostics |
| `list_models` | List stable models and credential-derived availability |
| `list_settings` | Read effective non-secret settings with scope provenance |
| `set_setting` | Store one global, project, or session configuration value |
| `list_credentials` | Read redacted provider credential status |
| `set_credential` | Store or replace one provider API key |
| `remove_credential` | Remove one provider API key |
| `list_projects` | List known active projects |
| `open_project` | Canonicalize and open a project |
| `select_project` | Select a known project and reopen its canonical root |
| `list_project_dependencies` | List stable IDs and display names for a project's read-only source dependencies |
| `add_project_dependency` | Canonicalize and register a non-overlapping read-only source folder |
| `remove_project_dependency` | Remove one dependency registration without changing its files |
| `list_project_directory` | Lazily list one bounded project or dependency directory for Explorer |
| `git_status` | Read the bounded Git index/worktree status and aggregate change counts for a project |
| `git_diff_file` | Read one bounded structured file diff for the all, staged, or unstaged scope |
| `list_sessions` | List active sessions in a project |
| `create_session` | Create a session with an optional title and selected model |
| `rename_session` | Rename a session |
| `archive_session` | Recoverably archive a session |
| `set_session_pinned` | Persist or clear a session's project-local pinned state in `session.pin_at` |
| `reopen_session` | Reopen an archived session |
| `list_session_images` | List persisted placeholder images for one session |
| `add_session_image` | Save one uploaded image file plus thumbnail metadata for a session |
| `remove_session_image` | Remove one persisted placeholder image from a session |
| `session_snapshot` | Read the normalized session projection; the cursor argument is ignored for compatibility |
| `session_usage` | Read cumulative provider-reported token usage for a session |
| `list_provider_exchanges` | List session turns and normalized provider call summaries for a trace tree |
| `provider_exchange` | Inspect one normalized provider call with correlated messages and tool uses |
| `list_checkpoints` | List turn-level checkpoint manifests for a session |
| `checkpoint_manifest` | Inspect one manifest and its items |
| `restore_checkpoint` | Restore a manifest with ownership and post-image conflict checks |
| `submit_turn` | Idempotently submit input to a session and selected model, with an optional `reasoning_effort` (`low`, `medium`, or `high`) accepted only for models advertising that capability |
| `cancel_turn` | Cooperatively cancel a running turn |
| `get_approval` | Read one approval state |
| `resolve_approval` | Resolve one pending approval with `allow_once`, `allow_session`, or `deny` |
| `reply_question` | Submit ordered answer arrays for one pending question request |
| `reject_question` | Skip one pending question request and resume with an explicit rejected result |
| `subscribe_session` | Deliver subsequent live events; lagged subscribers must reload `session_snapshot` |

Rust-generated project, session, turn, approval, checkpoint, event, and message identifiers remain authoritative. Hosts do not manufacture IDs except idempotency keys.

`tool_call_limit` is a project-only integer setting from 1 through 256. A project without that row uses 64. Turn admission snapshots the resolved value, so changing Settings affects later turns but not an active or approval-suspended turn. If one provider response would cross the limit, all calls in that response are retained as failed with `tool_budget_exceeded`, and none enters policy or execution.

`verify_https_certificates` is a global-only boolean setting that defaults to `true`. A successful update applies immediately to subsequent built-in OpenAI-compatible provider and WebFetch requests. Setting it to `false` accepts invalid server certificate chains and hostnames, equivalent to the TLS verification behavior of `curl -k`; an already-running request keeps the policy it started with. Trusted custom provider implementations remain responsible for their own transport behavior.

`image_directory` is a global-only string setting that defaults to the empty string. Empty means `<data directory>/data/images`. Each persisted session image is written under `{resolved_image_directory}/{sessionId}/{imageId}.{ext}`. The `session_image` row also stores the exact saved file path so older images remain readable after the global directory changes.

Models advertise `capabilities.reasoning_effort`. Avalonia presents `low`, `medium`, and `high` beside the model selector; unsupported models disable that selector and omit the parameter. For OpenAI-compatible providers, a selected value is sent as the `reasoning_effort` request field and is retained in the in-memory turn continuation across approval or question suspension.

Project dependency DTOs contain `dependencyId`, `projectId`, `displayName`, and `createdAt`, but never the canonical absolute root. `list_project_directory` selects the main project when `dependencyId` is null and a registered dependency otherwise. It returns at most 500 directories/files for one level, directories first, with root-relative slash-separated paths and a `truncated` flag. Symlinks and non-file entries are omitted. Adding a dependency rejects the project root, ancestors or descendants of the project, and roots that overlap another dependency.

The model addresses dependency content as `dependency:<dependencyId>/<relativePath>`. Only `read`, `glob`, and `grep` accept this alias; their results preserve the same prefix so later calls cannot accidentally resolve against the main project. Writes, edits, deletion, moves, processes, Git operations, checkpoints, and other authority remain scoped to the opened project and reject dependency aliases with `scope_denied`.

Git DTOs contain only opened-project-relative paths. `git_status` returns branch and detached-head information, aggregate file/addition/deletion/conflict counts, and per-file index/worktree status. `git_diff_file` returns structured hunks and lines with old/new line numbers plus a bounded plain-text patch. The agent embeds vendored libgit2 and does not require a Git executable or a system libgit2 installation. The current Git SDK surface is read-only.

`session_usage` returns `input_tokens`, `output_tokens`, and `total_tokens` summed from the latest cumulative usage projection for every turn in the session. Providers that omit usage metadata contribute zero; the agent does not estimate missing usage.

`session_snapshot` retains its flat `messages` compatibility projection and also returns `conversationTurns`. Each conversation turn contains `turnId`, terminal or active `state`, `createdAt`, correlated normalized `messages`, and correlated `toolUses`. Message rows retain `messageId`, role, message body, optional call correlation, and creation time. Tool rows retain their stable tool-call ID, name, request/result, state, ordering, timestamps, and redacted error code. Clients use this normalized turn projection to render live process activity and restore it after resync without opening SQLite. Persisted session-image placeholders are returned by `list_session_images`, not embedded into turn or message DTOs.

Provider exchange DTOs are local session diagnostics. The list result contains every session turn, including turns without calls, plus normalized call summaries. Each call retains its SunCode `exchangeId` and nullable `providerRequestId` and `providerResponseId`; the latter two are independent because providers may use different HTTP request and response-object identifiers or omit either one. One exchange detail contains normalized input messages, assistant output, correlated `session_message` rows, correlated `session_tool_use` rows, tool calls, finish reason, redacted provider errors, and provider-reported call usage. Correlated message DTOs do not duplicate usage. Call usage includes nullable `cache_read_tokens`, `cache_miss_tokens`, `cache_write_tokens`, and `reasoning_tokens` when available; clients may derive cache hit rate as `cache_read_tokens / input_tokens` only when both values are present and input is nonzero. Provider wire aliases such as `cached_tokens` and `prompt_cache_hit_tokens` are normalized rather than retained as duplicate fields. These DTOs never include provider API keys, HTTP authorization headers, or provider-private raw wire payloads.

Provider input includes a bounded project-root `AGENTS.md` system message when that file exists. A successful project `read` may add a `repository_instructions` array to its normalized result for unseen nested `AGENTS.md` files, ordered from the target's nearest directory toward the project root. Paths in this field are project-relative; automatic instruction loading never exposes the canonical root or reads dependency/out-of-scope files.

## Outcomes

Operation results do not encode HTTP statuses. A call either returns its method-specific success type or `SdkError`.

Turn submission returns a tagged outcome:

- `completed`: the admitted turn completed before the call returned;
- `awaiting_approval`: execution is suspended at a durable approval gate;
- `awaiting_question`: the model requested structured user clarification and the turn is suspended at a durable question gate;
- `queued`: input was accepted as an in-memory continuation of the active turn.

Cancellation returns `cancellation_requested`; cancelling a turn that is not active returns `conflict`.

`allow_session` atomically approves the pending operation and persists session-scoped `full_control=true` in `configuration`. While enabled, known approval-gated tools skip interactive approval for that session, but validation, project and dependency scope, auditing, checkpoints, cancellation, and unknown-tool denial remain enforced. Writing session-scoped `full_control=false` through `set_setting` restores normal approval behavior.

## Errors

An SDK error contains:

```json
{
  "code": "session_not_found",
  "message": "session not found",
  "details": {}
}
```

Messages and details are bounded and redacted. Important codes include `invalid_arguments`, `agent_already_active`, `agent_unavailable`, `project_not_found`, `session_not_found`, `model_unavailable`, `provider_unconfigured`, `approval_required`, `authorization_denied`, `checkpoint_unavailable`, `restore_conflict`, `conflict`, `scope_denied`, `not_git_repository`, `unsupported_git_repository`, `git_read_failed`, `git_diff_not_found`, `iteration_budget_exceeded`, `tool_budget_exceeded`, `cancelled`, and `resync_required`.

Panics are contained at native binding boundaries and converted to `agent_unavailable`; they never unwind into a host language.

## Events

Session events are live-only in-memory notifications. Normalized messages, turns, calls, tools, approvals, and checkpoints are the durable source of truth. Events do not carry a durable sequence.

Provider exchange lifecycle events are durable: `provider.exchange.started`, `provider.exchange.completed`, and `provider.exchange.failed`. They project into the provider-exchange query surface and may be used by clients to refresh an open trace drawer.

Question events are live notifications with normalized snapshot support: `question.asked` contains `request_id`, `turn_id`, `tool_call_id`, and ordered prompts; `question.replied` contains the same correlation plus ordered answer arrays; `question.rejected` contains the request correlation and an unanswered result. A session snapshot includes `pendingQuestion` while a request is waiting.

Todo state is turn-scoped and stored in the Rust-owned `session_turn_todo` table. The model-facing `todowrite` tool replaces the complete list with at most 100 items, and each item has `content`, `status` (`pending`, `in_progress`, `completed`, or `cancelled`), and `priority` (`high`, `medium`, or `low`). Successful calls emit a live `todo.updated` event containing `turn_id`, `tool_call_id`, and the complete `todos` list. Clients restore the current list from `conversationTurns[*].todos`; the corresponding `todowrite` tool result remains call history and is not the progress source.

Subscription establishment registers for live events. There is no SQLite replay phase. If a receiver lags, the subscription reports `resync.required`; the host must reload `session_snapshot`, then continue receiving live events.

Callbacks run on an SDK-owned thread. Hosts must copy the callback payload and marshal delivery to their runtime thread: Avalonia uses `Dispatcher.UIThread`, Node.js uses a thread-safe function, and Python acquires the GIL and schedules on the target event loop. Callback payload memory is valid only for the duration of the callback unless copied by the host.

## Authority and secrets

Embedding removes transport authentication because the host is inside the agent process trust boundary. It does not remove project/session ownership checks, policy evaluation, approval, operation auditing, canonical path validation, checkpoint conflict checks, or credential redaction.

Provider API keys remain Rust-owned plaintext values in `llm_model_provider.api_key`, which is their exclusive runtime source. Provider credential environment variables are ignored. Key values never appear in SDK results, events, diagnostics, audit records, or logs.

## Language bindings

Avalonia uses a hand-written C# P/Invoke wrapper over the C ABI and keeps native calls off the UI thread. The Rust crate emits a `cdylib` beside the managed executable. Future TypeScript and Python SDKs expose idiomatic async APIs over the same Rust methods and subscription semantics. They do not open SQLite, call providers, or implement agent behavior independently.
