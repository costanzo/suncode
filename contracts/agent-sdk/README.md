# Embedded Agent SDK Contract

Status: Implemented.

The SunCode agent is a native library embedded in its host process. It does not accept inbound HTTP, expose REST paths, publish a loopback endpoint, or support cross-process attach. The .NET Avalonia client references the managed C# SDK in `sdks/csharp`, which owns P/Invoke and typed DTOs over the stable C ABI. The typed Rust facade lives in `sdks/rust`, and the C ABI implementation lives in `sdks/c`; future TypeScript and Python packages wrap the same Rust facade through native bindings.

Provider adapters may make outbound HTTPS requests to configured model providers, and remote MCP clients may connect to configured Streamable HTTP endpoints. That network behavior is internal to the Rust agent and is not a client transport.

## Lifecycle

`open_default(user_id)` validates and binds a non-empty user identifier, then loads configuration, acquires the data-directory lock, opens and initializes the current SQLite schema, initializes operations and providers, performs recovery, and returns an opaque agent handle. It delegates to default `SdkOpenOptions`, whose Browser and Computer host capabilities are both enabled for compatibility. Rust hosts may use `open_with_options` or `open_with_options_and_providers` to apply an immutable typed capability ceiling. A disabled capability is neither advertised to models nor initialized, and its management methods fail before changing persisted settings. Capability ceilings cannot grant policy authority. Project and project-derived session APIs are filtered to the immutable user scope. The Phase 1 Avalonia host supplies `os:<Environment.UserName>`; this is logical ownership, not authentication. A second process opening the same data directory receives `agent_already_active`. An incompatible database is rejected.

The async Rust facade owns agent services but no executor; `AsyncAgentSdk::open_default(...).await` and all runtime-dependent methods execute on the host's Tokio runtime. `AsyncAgentSdk::shutdown(self).await` consumes the handle, rejects new agent work, cancels turns, clears queued input, releases Computer Use state, drains Browser/MCP/LSP processes, waits a bounded five seconds for active turns, closes event streams, and releases the data-directory lock. Pure local persistence and DTO operations remain synchronous and explicit. The root compatibility `AgentSdk` is the blocking wrapper: it owns one Tokio runtime, dereferences to the async facade for synchronous methods, adapts awaited operations with `block_on`, and keeps that runtime alive through its consuming `shutdown(self)`. C embeds that blocking wrapper. Host wrappers may share one handle inside a process. The Rust facade exposes a typed `SessionEventStream` implementing standard `Stream` and `FusedStream` contracts plus direct receive methods; native bindings adapt that stream to their host runtime. C subscriptions must be closed before the final agent handle is released. Closing a C subscription signals its stream and stops callback delivery before returning. Final native handle release invokes blocking shutdown; its unchanged `void` close function logs cleanup errors.

The C ABI exposes `suncode_agent_sdk_abi_version` and reports ABI version 13. Hosts use the current `agent` symbol family directly; there is no compatibility layer for prior native APIs. ABI functions and enum-like integer values are add-only within a major ABI version. Rust layouts, references, strings, vectors, and errors never cross the ABI directly.

## Methods

The Rust API uses typed inputs and outputs. Async Rust hosts use `AsyncAgentSdk`; synchronous and native hosts use `blocking::AgentSdk`, also exported at the crate root as the compatibility `AgentSdk`. The C ABI exposes one named function per operation currently consumed by native bindings. Rust-only composition helpers may be introduced before their binding lifecycle is defined; they do not authorize a language binding to reconstruct the behavior itself. Complex evolving results may be returned as method-specific UTF-8 JSON payloads owned by the SDK; this is DTO serialization, not generic routing.

| SDK method | Purpose |
| --- | --- |
| `version` | Read the Rust agent core package version without opening agent state |
| Rust `shutdown` | Consume the SDK, cancel active work, close owned runtime resources and event streams, and release the data-directory lock |
| `health` | Read agent and database health |
| `diagnostics` | Read redacted agent, recovery, credential, and active-project diagnostics |
| `list_models` | List stable models and credential-derived availability |
| `list_agents` | List the six immutable Rust-defined specialist agents and their exact tool policies |
| `list_settings` | Read effective non-secret settings with scope provenance |
| `set_setting` | Store one global, project, or session configuration value |
| `set_proxy_configuration` | Atomically store global proxy mode, endpoint, credentials, and bypass rules while returning only redacted password state |
| `list_credentials` | Read redacted provider credential status |
| `set_credential` | Store or replace one provider API key |
| `remove_credential` | Remove one provider API key |
| `set_provider_endpoint` | Validate, persist, and apply one provider API base URL |
| `computer_runtime_info` | Read global Computer Use enablement, backend availability, primary-display geometry, redacted permission state, control owner, and safe error state |
| `request_computer_capture_permission` | Explicitly ask the operating system for desktop capture permission and return refreshed redacted runtime state |
| `request_computer_input_permission` | Explicitly ask the operating system for desktop input permission and return refreshed redacted runtime state |
| `take_computer_control` | Transfer exclusive desktop-control ownership to the user, cancel active Computer Use work, release held input, and retire the current coordinate frame |
| `return_computer_control` | Return exclusive control to the agent while requiring a fresh screenshot before coordinate input |
| `set_computer_use_enabled` | Persist global Computer Use availability, install the built-in backend when enabling, and release held input when disabling |
| `emergency_stop_computer_use` | Persist disabled state, cooperatively cancel active Computer Use work, and release held keys and mouse buttons |
| `browser_runtime_info` | Read global Browser Use enablement, packaged component identity, installation health, and optional project runtime/profile state |
| `set_browser_use_enabled` | Persist global Browser Use availability and stop active project runtimes when disabling |
| `verify_browser_runtime` | Probe the bundled Node.js worker and validate its identity against the runtime lock |
| `start_browser_project` | Lazily start the selected project's persistent Chromium runtime |
| `take_browser_control` | Show the project browser and transfer its exclusive control lease to the user |
| `return_browser_control` | Return control to the agent and invalidate previous browser references |
| `restart_browser_runtime` | Restart the project worker and Chromium while preserving its profile |
| `stop_browser_runtime` | Stop the project worker and Chromium while preserving its profile |
| `clear_browser_profile` | Remove one stopped project's persistent browser data |
| `list_mcp_servers` | List global MCP definitions with the selected project's runtime status and redacted secret key names |
| `create_mcp_server` | Validate, persist, and reconcile one stdio or Streamable HTTP server |
| `update_mcp_server` | Replace one server definition using an optimistic revision and secret patch |
| `set_mcp_server_enabled` | Persist enabled state and immediately start or retire project-scoped clients |
| `delete_mcp_server` | Delete one definition and retire its clients and tools from subsequent requests |
| `retry_mcp_server` | Retry one enabled server for an active project |
| `start_mcp_project` | Start project-scoped MCP connections in the background and return initial progress |
| `mcp_load_progress` | Read project-scoped MCP startup progress without waiting for connections |
| `list_language_servers` | List global language-server definitions with selected-project runtime status and redacted environment key names |
| `create_language_server` | Validate, persist, and reconcile one local stdio language server |
| `update_language_server` | Replace one definition using optimistic revision and a write-only environment patch |
| `set_language_server_enabled` | Persist enabled state and start or retire active project runtimes |
| `delete_language_server` | Delete one definition and stop its active project runtimes |
| `retry_language_server` | Retry one enabled language server for an active project |
| `start_language_server_project` | Start matching enabled project-scoped language servers in the background |
| `list_projects` | List known active projects for the SDK user |
| `open_project` | Canonicalize and open a project for the SDK user |
| `select_project` | Select a known project belonging to the SDK user and reopen its canonical root |
| `list_project_dependencies` | List stable IDs and display names for a project's read-only source dependencies |
| `add_project_dependency` | Canonicalize and register a non-overlapping read-only source folder |
| `remove_project_dependency` | Remove one dependency registration without changing its files |
| `list_project_directory` | Lazily list one bounded project or dependency directory for Explorer |
| `read_project_file` | Read one bounded UTF-8 project or dependency file for the desktop viewer |
| `git_status` | Read the bounded Git index/worktree status and aggregate change counts for a project |
| `git_diff_file` | Read one bounded structured file diff for the all, staged, or unstaged scope |
| `list_sessions` | List active sessions in a project, including the persisted model and reasoning-effort preference |
| `list_child_sessions` | List delegated child sessions and invocation state for one primary session |
| `create_session` | Create a session with an optional title and selected model |
| `rename_session` | Rename a session |
| `archive_session` | Recoverably archive a session |
| `set_session_pinned` | Persist or clear a session's project-local pinned state in `session.pin_at` |
| `reopen_session` | Reopen an archived session |
| `list_session_images` | List persisted images that are still pending in one session composer |
| `add_session_image` | Save one uploaded image file plus thumbnail metadata for a session |
| `remove_session_image` | Remove one pending persisted image; submitted message attachments cannot be removed |
| `session_snapshot` | Read the normalized session projection; the cursor argument is ignored for compatibility |
| Rust `watch_session` | Atomically register a typed session stream and read the normalized snapshot under one session gate |
| `session_usage` | Read cumulative provider-reported token usage for a session |
| `list_provider_exchanges` | List session turns and normalized provider call summaries for a trace tree |
| `provider_exchange` | Inspect one normalized provider call with correlated messages and tool uses |
| `list_checkpoints` | List turn-level checkpoint manifests for a session |
| `checkpoint_manifest` | Inspect one manifest and its items |
| `restore_checkpoint` | Restore a manifest with ownership and post-image conflict checks |
| `submit_turn` | Idempotently submit text-only input to a session and selected model, persist the selected model and effort on the session, and normalize an omitted effort to the selected model's first advertised effort |
| `submit_turn_with_attachments` | Submit text plus up to three same-session image IDs to a model advertising image input, persisting the selected model and normalized effort on the session |
| `cancel_turn` | Cooperatively cancel a running turn |
| `retry_last_turn` | Re-submit the most recently failed turn in a session using its persisted input and model; creates a new turn with a fresh idempotency key |
| `get_approval` | Read one approval state |
| `pending_approval` | Read the current pending approval for one validated session, if present (Rust facade) |
| `resolve_approval` | Resolve one pending approval with `allow_once`, `allow_session`, or `deny` |
| `reply_question` | Submit ordered answer arrays for one pending question request |
| `reject_question` | Skip one pending question request and resume with an explicit rejected result |
| `subscribe_session_events` / C `subscribe_session` | Deliver subsequent typed live events; lagged subscribers must reload `session_snapshot` |
| C `watch_session` | Return the atomic session snapshot and a dormant callback subscription handle |
| C `subscription_start` | Start callback delivery for one dormant subscription exactly once |

Rust-generated project, session, turn, approval, checkpoint, event, and message identifiers remain authoritative. Hosts do not manufacture IDs except idempotency keys.

The built-in agent catalog contains `architect-agent`, `ui-ux-agent`, `product-agent`, `swe-agent`, `qa-agent`, and `sre-agent`. Definitions are compiled into Rust and expose a stable ID, unique machine name, display name, description, definition version, exact built-in tool allowlist, model policy, MCP policy, delegation policy, and tool-call limit. There is no create, update, enable, reorder, or delete method for this catalog.

Only a primary agent turn may invoke the core-owned `delegate_agent` conversation tool. Every invocation creates a new `kind=child` session linked to its primary parent and one durable `subagent_invocation` row. Child sessions inherit project, model, and reasoning effort; their effective tool-call limit is the smaller of the project limit and the built-in agent limit. They receive only their built-in allowlist, never MCP, `question`, or `delegate_agent`, and execution revalidates every returned call. Public submission, retry, rename, pin, archive, and reopen calls reject child sessions. Snapshot, usage, provider-trace, approval, and event read paths remain available so clients can inspect and authorize delegated work.

Delegation depth is one. Parent cancellation shares its cancellation token with an active child turn. A child approval suspends only the child continuation and updates the invocation to `awaiting_approval`; resolving it through the normal approval API resumes the child and eventually writes `completed`, `failed`, `cancelled`, or another `awaiting_approval` state. The parent turn may finish after receiving the awaiting result. Child checkpoints remain owned by the child session, so this contract does not claim parent-turn undo includes child mutations.

MCP definitions are global desired state. Live clients and runtime states are keyed by server and project; `list_mcp_servers` accepts an optional project ID and reports `not_started`, `disabled`, `connecting`, `connected`, or `failed`. Project activation is explicit: `start_mcp_project` schedules enabled server connections in the background and returns immediately; `mcp_load_progress` reports `total`, `settled`, `connected`, `failed`, and `loading`. Read DTOs include only configured environment/header key names, never values. Create derives a stable opaque server ID from its idempotency key. Update, enable, and delete require both an idempotency key and the expected revision. Secret patches use `{ set, remove }`: omitted patches preserve existing values, `set` replaces named values, and `remove` deletes named values.

MCP write request JSON at the C ABI boundary uses camelCase field names to match the managed SDK: `displayName`, `transport`, `kind`, `workingDirectory`, `startupTimeoutSeconds`, and `requestTimeoutSeconds` (with `command`, `arguments`, `environment`, `url`, `headers`, `enabled`, and `sortOrder`). Rust's persistence-layer `McpTransportConfig` intentionally retains snake_case field names inside SQLite transport JSON and is not the native request wire shape.

Local transports receive a structured executable and argument list and are started without a shell. Their working directory is either the selected project's canonical root or the application data directory. Remote endpoints require HTTPS except loopback HTTP and follow the global certificate settings. Successful mutations persist before runtime reconciliation and retire old catalog generations immediately. Existing sessions snapshot the current connected MCP definitions before every provider call, so the next model request observes create, edit, enable, disable, delete, and tool-list changes without reopening the session.

MCP stdio processes receive a Rust-owned, OS-specific environment allowlist on every launch. Unix-like launches provide standard executable lookup (`PATH` with Homebrew, user-local, Cargo, system, and host entries), `HOME`, user/login/shell identity, locale (`LANG`, `LC_ALL`), terminal/display session values, `TMPDIR`, XDG directories, and the canonical working directory (`PWD`). Windows launches provide `PATH`, `SystemRoot`/`WINDIR`, profile/application-data directories, `ComSpec`, `PATHEXT`, username/OS identity, and `TEMP`/`TMP`. Missing host values use deterministic safe defaults; MCP-configured entries are applied last and override the baseline. Loader/code-injection variables are excluded from implicit inheritance.

Language-server definitions are global desired state while processes, initialization, document versions, capabilities, and failures are project-scoped memory. The write DTO contains `displayName`, `command`, `arguments`, `languageIds`, `rootMarkers`, `initializationOptions`, write-only `environment` changes, startup/request timeouts, enabled state, and ordering. Read DTOs expose environment key names only. Runtime states are `not_started`, `disabled`, `starting`, `indexing`, `ready`, and `failed`. Processes launch without a shell in the project root and receive a filtered environment. The first delivery supports only local stdio servers and does not accept server-requested edits or arbitrary command execution.

`browser_use_enabled` is a global-only boolean defaulting to `false`. The bundled Browser Use runtime is fixed by the installed application and cannot be replaced through SDK or Settings paths. `browser_runtime_info` reports installation state (`disabled`, `ready`, `missing`, `invalid`, `unsupported`, or `verifying`), project runtime state (`not_started`, `starting`, `background`, `user_controlled`, `stopping`, or `failed`), target, Node.js/Playwright/Chromium identity and paths, worker protocol, integrity, control owner, profile path/size, active page count, visibility capability, and a bounded safe error. Runtime state is memory-only; project profiles live under the agent data directory and are identified by a one-way project-ID-derived directory name.

`computer_use_enabled` is a global-only boolean defaulting to `false`. Enabling installs the built-in Enigo-backed runtime and does not grant input authority. `computer_runtime_info` reports only redacted runtime facts: backend availability, primary-display input and screenshot dimensions, capture and input permission state (`allowed`, `denied`, `unknown`, or `unsupported`), current control owner, and a bounded safe error. Permission state may remain `unknown` where the backend cannot inspect it without performing a user-visible operation. Emergency stop disables the capability, makes active cooperative execution observe cancellation, and releases held input before returning. A later explicit enable is required to resume.

Desktop-control ownership is exclusive. User takeover removes the Computer Use toolset from subsequent provider calls, makes active cooperative execution observe cancellation, releases held input, and retires the current screenshot frame. Returning control re-enables advertisement for otherwise supported models but retains no prior coordinate frame, so the first later coordinate action requires a fresh screenshot.

Computer Use is interactive-only in the initial delivery. Observation actions follow the interactive default policy; any input-producing batch requires approval even when the session otherwise has Full Control. Screenshot bytes are transient provider context and are not returned by management DTOs or stored in durable continuation JSON.

Full-display and zoom images sent to a provider preserve aspect ratio and the input-coordinate mapping while being bounded to a 1568-pixel edge, 1,000,000 pixels, and a 5 MiB encoded PNG. The active provider context keeps image content for at most the two newest Computer Use results; older correlated tool results are retained as text omission markers so tool-use/result ordering remains valid without retaining unbounded screenshot bytes.

Browser runtime management never grants site authority. User-control handoff is exclusive and agent browser actions remain unavailable until control returns. Clearing a profile requires the runtime to be stopped and deletes browser cookies, login state, and site storage without changing project files. Chromium keeps ordinary certificate verification and does not inherit the global insecure certificate toggle.

The model-facing semantic catalog is fixed to `lsp_diagnostics`, `lsp_definition`, `lsp_references`, `lsp_hover`, and `lsp_symbols`. These calls are read-only, use one-based source positions, synchronize at most 1 MiB of UTF-8 text through the audited project/dependency read path, and normalize returned locations to project-relative paths or registered dependency aliases. Absolute external locations are not exposed. Missing servers, capabilities, timeouts, crashes, and malformed responses are recoverable `lsp_*` tool results.

`tool_call_limit` is a project-only integer setting from 1 through 256. A project without that row uses 64. Turn admission snapshots the resolved value, so changing Settings affects later turns but not an active or approval-suspended turn. If one provider response would cross the limit, all calls in that response are retained as failed with `tool_budget_exceeded`, and none enters policy or execution.

`verify_https_certificates` is a global-only boolean setting that defaults to `true`. A successful update applies immediately to subsequent built-in OpenAI-compatible provider and WebFetch requests. Setting it to `false` accepts invalid server certificate chains and hostnames, equivalent to the TLS verification behavior of `curl -k`; an already-running request keeps the policy it started with. Trusted custom provider implementations remain responsible for their own transport behavior.

`use_system_certificates` is a global-only boolean defaulting to `true`. `certificate_path` is a global-only optional PEM or DER trust-anchor file. When verification is enabled, built-in provider and WebFetch clients use the system roots when enabled and add the configured custom root when present; disabling system roots makes the supplied file the trust source. Invalid or unreadable certificate files fail the subsequent request with a stable SDK error.

`set_provider_endpoint` updates the existing URL for one registered provider without changing its provider identity, adapter, credential, models, enabled state, or ordering. The endpoint must be an absolute HTTP or HTTPS URL with a host and without embedded credentials, query parameters, or a fragment; whitespace and trailing slashes are removed. A successful update is durable and atomically replaces the in-memory route used by subsequent provider calls. A provider request that already captured its route continues with the previous endpoint.

`image_directory` is a global-only string setting that defaults to the empty string. Empty means `<data directory>/data/images`. Each persisted session image is written under `{resolved_image_directory}/{sessionId}/{imageId}.{ext}`. The `session_image` row also stores the exact saved file path so older images remain readable after the global directory changes.

Image upload accepts PNG, JPEG, GIF, WebP, BMP, and AVIF file extensions. Original files are bounded to 20 MiB and thumbnail payloads to 1 MiB. `submit_turn_with_attachments` accepts at most three unique IDs, verifies same-session ownership and file availability, and rejects models that do not advertise `capabilities.vision`. Accepted user messages persist `image_ref` content parts; provider requests resolve those references to data URLs only at call time, while provider trace input stores a redacted `[image attachment]` marker. Image-bearing submissions are rejected rather than queued behind an active turn so their files cannot be removed before admission. The original text-only ABI remains a compatibility wrapper with an empty image list.

Models advertise `capabilities.reasoning_effort` and a `reasoning_efforts` catalog. Avalonia presents the selected model's advertised values beside the model selector; unsupported models disable that selector and omit the parameter. When a turn omits effort, Rust selects the first advertised value when one exists. For OpenAI-compatible providers, a selected value is sent as the `reasoning_effort` request field, persisted with the session's selected model at turn admission, and retained in the in-memory turn continuation across approval or question suspension.

Project dependency DTOs contain `dependencyId`, `projectId`, `displayName`, and `createdAt`, but never the canonical absolute root. `list_project_directory` selects the main project when `dependencyId` is null and a registered dependency otherwise. It returns at most 500 directories/files for one level, directories first, with root-relative slash-separated paths and a `truncated` flag. Symlinks and non-file entries are omitted. Adding a dependency rejects the project root, ancestors or descendants of the project, and roots that overlap another dependency.
`read_project_file` applies the same project/dependency root selection and canonical scope checks. It accepts one root-relative regular-file path, rejects symbolic links, NUL-containing/binary content, and non-UTF-8 content, and returns at most 1 MiB for the read-only desktop viewer. Files above the bound fail with `file_too_large`; the SDK never returns a partial document or creates an artifact for this client read.

The model addresses dependency content as `dependency:<dependencyId>/<relativePath>`. Only `read`, `glob`, and `grep` accept this alias; their results preserve the same prefix so later calls cannot accidentally resolve against the main project. Writes, edits, deletion, moves, processes, Git operations, checkpoints, and other authority remain scoped to the opened project and reject dependency aliases with `scope_denied`.

Git DTOs contain only opened-project-relative paths. `git_status` returns branch and detached-head information, aggregate file/addition/deletion/conflict counts, and per-file index/worktree status. `git_diff_file` returns structured hunks and lines with old/new line numbers plus a bounded plain-text patch. The agent embeds vendored libgit2 and does not require a Git executable or a system libgit2 installation. The current Git SDK surface is read-only.

`session_usage` returns `input_tokens`, `output_tokens`, and `total_tokens` summed from the latest cumulative usage projection for every turn in the session. Providers that omit usage metadata contribute zero; the agent does not estimate missing usage.

`session_snapshot` retains its flat `messages` compatibility projection and also returns `conversationTurns`. Each conversation turn contains `turnId`, terminal or active `state`, `createdAt`, correlated normalized `messages`, and correlated `toolUses`. Message rows retain `messageId`, role, message body, optional call correlation, and creation time. A user message may contain durable `image_ref` parts whose text is the owning `session_image.image_id`; the snapshot's `images` array supplies client-local thumbnail and storage metadata for rendering those references. Tool rows retain their stable tool-call ID, name, request/result, state, ordering, timestamps, and redacted error code. Clients use this normalized turn projection to render live process activity and restore it after resync without opening SQLite. `list_session_images` excludes images already referenced by submitted messages so only pending composer images are returned.

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

Proxy configuration is global and is updated atomically through `set_proxy_configuration`. The request carries `mode`, `url`, `username`, optional replacement `password`, explicit `clearPassword`, and `bypass`. Omitting `password` preserves the stored value. Settings reads never return the persisted `proxy_password`; they return `proxy_password_configured` instead. The update response likewise contains only non-secret values and `passwordConfigured`.

## Errors

An SDK error contains:

```json
{
  "code": "session_not_found",
  "message": "session not found",
  "details": {}
}
```

Messages and details are bounded and redacted. Important codes include `invalid_arguments`, `agent_already_active`, `agent_unavailable`, `project_not_found`, `session_not_found`, `model_unavailable`, `provider_unconfigured`, `approval_required`, `authorization_denied`, `checkpoint_unavailable`, `restore_conflict`, `conflict`, `scope_denied`, `mcp_server_conflict`, `mcp_server_revision_conflict`, `mcp_tool_unavailable`, `not_git_repository`, `unsupported_git_repository`, `git_read_failed`, `git_diff_not_found`, `iteration_budget_exceeded`, `tool_budget_exceeded`, `cancelled`, and `resync_required`.

Panics are contained at native binding boundaries and converted to `agent_unavailable`; they never unwind into a host language.

## Events

Session events are live-only in-memory notifications. Normalized messages, turns, calls, tools, approvals, and checkpoints are the durable source of truth. Events do not carry a durable sequence.

Inside the Rust agent, event emission is strongly typed: `EventPayload` is a non-exhaustive enum whose variants carry named Rust payload structs, and each variant determines its `EventType`. Agent code cannot independently pair an arbitrary event-name string with an unrelated JSON object. Core publishes `AgentEvent` values through bounded session-scoped subscriber queues, so traffic from another session cannot fill or lag the selected session's queue. The Rust SDK exposes async, blocking, and nonblocking receive methods without JSON parsing. At the C SDK boundary these types retain the existing `{ session_id, occurred_at, event_type, payload }` JSON envelope and dotted event names for C and C# compatibility.

Provider exchange lifecycle events are durable: `provider.exchange.started`, `provider.exchange.completed`, and `provider.exchange.failed`. They project into the provider-exchange query surface and may be used by clients to refresh an open trace drawer.

Question events are live notifications with normalized snapshot support: `question.asked` contains `request_id`, `turn_id`, `tool_call_id`, and ordered prompts; `question.replied` contains the same correlation plus ordered answer arrays; `question.rejected` contains the request correlation and an unanswered result. A session snapshot includes `pendingQuestion` while a request is waiting.

Process tools also emit live-only `tool.output` events while a command is running. Each event contains `turn_id`, `call_id`, `tool_call_id`, `stream` (`stdout` or `stderr`), and a UTF-8 `chunk_base64` payload. Chunks are bounded (at most 8 KiB per event), are best-effort notifications, and are not persisted or replayed; the terminal `tool.result` remains the durable source of truth.

Todo state is turn-scoped and stored in the Rust-owned `session_turn_todo` table. The model-facing `todowrite` tool replaces the complete list with at most 100 items, and each item has `content`, `status` (`pending`, `in_progress`, `completed`, or `cancelled`), and `priority` (`high`, `medium`, or `low`). Successful calls emit a live `todo.updated` event containing `turn_id`, `tool_call_id`, and the complete `todos` list. Clients restore the current list from `conversationTurns[*].todos`; the corresponding `todowrite` tool result remains call history and is not the progress source.

Subscription establishment registers a bounded queue for one session. There is no SQLite replay phase. For standard Rust stream consumption, each item is `Result<Arc<AgentEvent>, SubscriptionError>`; normal close returns `None`, while lag returns one `SubscriptionError::Lagged` item and then the fused stream terminates. Direct `recv`, `blocking_recv`, and `try_recv` methods remain available; `try_recv` may return `Empty` without terminating. The C adapter converts lag to `resync.required`. The host must reload `session_snapshot` and establish a fresh subscription before continuing. Dropping or closing a stream releases its hub registration and wakes pending async or blocking receivers.

Rust hosts should use `watch_session` for initial load and resynchronization. Core serializes event projection-plus-publish and watch establishment through one in-memory gate per session. The watch registers its stream before reading the snapshot while holding that gate. An event projection completed before the watch acquired the gate is visible in the snapshot; an event projection beginning afterward publishes into the returned stream. Live-only events also wait for the gate and enter the returned stream after snapshot creation. Snapshot failure unregisters the provisional subscriber. Some normalized lifecycle writes, including turn admission, intentionally precede their corresponding notification projection, so hosts must continue applying stream events idempotently. This ordering is process-local and does not create a durable event cursor or replay log.

The existing standalone `session_snapshot` and `subscribe_session` methods remain compatibility operations and do not jointly provide the atomic watch guarantee. C `watch_session` invokes the Rust atomic operation, returns snapshot JSON, and keeps the typed stream dormant. The host applies the snapshot and calls `subscription_start` exactly once when its presentation state is ready; events queue in the bounded stream before activation. Closing a dormant handle releases it without creating a callback thread. C# exposes this lifecycle as typed `SessionWatch.Snapshot`, `Start`, and `Dispose`.

The Rust facade does not own callback threads, C strings, or raw callback pointers. The C binding starts callbacks only after explicit activation, runs them on a C-binding-owned worker thread, and serializes typed events only at that boundary. Callback hosts must copy the payload and marshal delivery to their runtime thread: Avalonia uses `Dispatcher.UIThread`. Callback payload memory is valid only for the duration of the callback unless copied by the host. Future Node.js and Python bindings adapt the typed stream with their own thread-safe-function or event-loop mechanisms rather than reusing the C callback worker.

## Authority and secrets

Embedding removes transport authentication because the host is inside the agent process trust boundary. It does not remove project/session ownership checks, policy evaluation, approval, operation auditing, canonical path validation, checkpoint conflict checks, or credential redaction.

Provider API keys remain Rust-owned plaintext values in `llm_model_provider.api_key`, which is their exclusive runtime source. Provider credential environment variables are ignored. Key values never appear in SDK results, events, diagnostics, or logs.

MCP calls have `ExternalTool` risk. Interactive policy requires approval unless the session has Full Control; non-interactive policy denies them by default. Approval records retain stable server and remote-tool identity but not configured secret values. MCP processes and remote services can make changes outside SunCode's checkpoint and undo boundary. A local stdio process runs with the user's OS authority and is not made trustworthy or OS-sandboxed by the Rust protocol boundary.

## Language bindings

Avalonia references the hand-written `sdks/csharp` C# SDK and keeps native calls off the UI thread. The managed SDK owns typed request/response models, dormant watch lifetime and activation, UTF-8 conversion, JSON envelope parsing, and the Cargo build integration for `sdks/c`. Primary-session loading applies the atomic snapshot and auxiliary presentation state, installs the watch as the current subscription, then explicitly starts callbacks. A stale or failed load disposes the dormant handle. The C binding crate emits a `cdylib` beside the managed executable. Future TypeScript and Python SDKs expose idiomatic async APIs over the same Rust methods and subscription semantics. They do not open SQLite, call providers, or implement agent behavior independently.
