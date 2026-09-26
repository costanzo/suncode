# Agent SQLite Schema

Status: Current Phase 1 contract.

The `suncode-data` package is the only ORM/database-connection owner and uses Diesel's SQLite backend for connections, transactions, typed table declarations, and query execution. The `suncode-database` package owns backend resources: `suncode_database::sqlite` contains the current SQL manifests, seed data, table manifest, and database-file creation/existence check. There is one current 18-table set, no version table, and no general migration runner. File names do not encode execution order. Table-owned ORM operations live under `agent/crates/data/src/operations/`, with `projection.rs` and `recovery.rs` reserved for cross-table workflows. Opening a database with any unexpected application table fails without conversion. Initialization applies the current manifest transactionally. `session_turn_todo` is the authoritative per-turn todo projection and is replaced transactionally by `todo.updated` events.

There are 18 application tables:

`configuration`, `language_server`, `llm_model`, `llm_model_provider`, `mcp_server`, `project`, `project_dependency`, `session`, `session_approval_request`, `session_call`, `session_checkpoint`, `session_checkpoint_manifest`, `session_image`, `session_message`, `session_subagent_invocation`, `session_tool_use`, `session_turn`, and `session_turn_todo`.

## Conventions

- IDs are non-empty opaque text; agent IDs are UUIDs.
- Timestamps are UTC RFC 3339 strings with millisecond precision.
- JSON columns must contain valid JSON. Queryable identity, state, ordering, and ownership remain relational columns.
- Foreign-key enforcement is disabled on every connection. Rust validates referenced identifiers at the owning operation boundary. Projects and sessions are archived rather than normally deleted.

## Project And Settings

### `project`

One row per opened local directory tree: `project_id` primary key, non-empty `user_id`, unique `(user_id, canonical_root)`, non-empty `display_name`, lifecycle timestamps, and nullable `archived_at`.

### `project_dependency`

One registered read-only source root per row: opaque `dependency_id`, owning `project_id`, canonical root, display name, and creation timestamp. `(project_id, canonical_root)` is unique, and deleting a project cascades to its dependency registrations. The canonical root remains internal to Rust persistence and operations; client DTOs expose only the stable dependency ID and display name.

### `configuration`

Unified key/value configuration for `global`, `project`, and `session` scopes. Global rows have no owner ID; project rows reference `project`; session rows reference `session`. A CHECK constraint enforces the exact owner shape for each scope, and partial unique indexes enforce one value per key at each scope. Effective reads apply `global < project < session` precedence. The project-aware `default_model` key is stored as a JSON string containing a model ID.

The project-only `tool_call_limit` key is a JSON integer from 1 through 256. When the row is absent, core uses 64. Global TLS settings include `verify_https_certificates`, `use_system_certificates`, and optional PEM/DER `certificate_path`.

Fresh and reopened current databases seed four global logging settings: `log_level` (`"INFO"`), `log_directory` (`""`), `log_max_bytes` (`10485760`), and `log_retention` (`5`), plus global `verify_https_certificates` (`true`), `browser_use_enabled` (`false`), and `image_directory` (`""`). An empty log directory means `<data directory>/logs`; an empty image directory means `<data directory>/data/images`. These settings are global-only. The SDK accepts `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`, or `OFF`; a directory string; a maximum size of at least 1024 bytes; a retention count from 0 through 100; boolean HTTPS-verification and Browser Use values; and an image directory string. Disabling verification makes subsequent built-in provider and WebFetch HTTPS requests accept invalid certificate chains and hostnames, but it does not weaken bundled Chromium certificate verification.

## MCP Servers

### `mcp_server`

One row per global MCP server definition. It stores the opaque `mcp_server_id`, unique case-insensitive display name, immutable unique case-insensitive tool prefix, transport kind and versioned transport JSON, desired enabled state, ordering, optimistic revision, and timestamps. Transport JSON is either structured local stdio configuration or remote Streamable HTTP configuration. It may contain plaintext environment/header secrets; read DTOs expose key names only. Runtime connection state and discovered tools are project-scoped memory and are never persisted in this table.

The tool prefix is generated on create and does not change when the display name is edited. Each successful non-idempotent update advances `revision`. Initialization retains the narrow additive path that added this table to the prior 15-table schema; built-in agents add a separate current compatibility step for `session_subagent_invocation` and session classification columns.

## Language Servers

### `language_server`

One row per global local-stdio language-server definition. It stores the opaque `language_server_id`, unique case-insensitive display name, versioned configuration JSON, desired enabled state, ordering, optimistic revision, and timestamps. Configuration JSON contains the structured executable and argument vector, language IDs, project-relative root markers, initialization options, plaintext environment values, and bounded startup/request timeouts. SDK reads expose only environment key names. Project runtime state, process handles, negotiated capabilities, diagnostics, and document versions are memory-only and are not persisted.

## Sessions

### `session`

One conversation per row, linked to a project. It stores optional title/model, normalized reasoning effort, `active`/`archived` status, activity timestamps, nullable `pin_at`, and archive consistency checks. `kind` is `primary` or `child`. Primary rows have null parent/agent fields; child rows require `parent_session_id`, the stable built-in `agent_id`, and `agent_version`. A non-null `pin_at` marks a primary session as pinned and records when it was pinned. Active project session listings filter to primary rows; children are listed through their parent.

### `session_subagent_invocation`

One durable delegation correlation per child session. It links the parent session, parent turn, parent `delegate_agent` tool-call ID, child session, built-in agent ID/version, task JSON, snapshotted tool allowlist, and inherited model. State is `created`, `running`, `awaiting_approval`, `completed`, `failed`, `cancelled`, or `interrupted`; result/error and lifecycle timestamps retain the latest normalized outcome. Startup marks `created` or `running` invocations interrupted, while `awaiting_approval` remains durable for explicit resolution.

### `session_turn`

The single source of truth for a turn. It combines turn lifecycle, submission idempotency, input/response/error JSON, model selection, cumulative provider usage, and approval continuation state. States are `admitted`, `queued`, `preparing`, `calling_model`, `resolving_calls`, `compacting`, `completed`, `failed`, `cancelled`, and `interrupted`.

### `session_turn_todo`

The authoritative current todo list for one turn. Each row is identified by `(turn_id, ordinal)` and stores bounded content, status, priority, creation/update timestamps, and completion time. A successful `todo.updated` event replaces the complete set for that turn transactionally; `session_tool_use.result_json` remains the result of the individual model call and is not used as the progress projection.

A structured failure write can populate `error_json` and `error_code` after a failed lifecycle projection, but cannot replace completed, cancelled, or interrupted state.

Approval recovery is kept on the turn in `recovery_approval_id`, `recovery_snapshot_json`, `recovery_status`, and recovery timestamps; there is no suspended-turn table. The unique `(session_id, submission_idempotency_key)` constraint makes retries idempotent. Recovery, resuming, and session chronology indexes support startup and history queries.

### `session_call`

One row per LLM request within a turn. `call_id` is the SunCode-owned physical primary key and is linked to `session_turn`; nullable `provider_request_id` and `provider_response_id` retain the provider's HTTP request identifier and response-object identifier independently. Provider/model/wire-model identity, iteration, lifecycle state, normalized output/tool-call/usage/error JSON, finish reason, and timestamps are retained. Model request messages are intentionally not persisted or exposed through provider trace. Normalized usage may include nullable `cache_read_tokens`, `cache_miss_tokens`, `cache_write_tokens`, and `reasoning_tokens` in addition to input, output, and total tokens; provider-private aliases are not duplicated. States are `started`, `completed`, and `failed`. Session, turn, and in-flight indexes support diagnostics and recovery.

### `session_image`

One persisted image per row, linked to `session`. The row stores opaque `image_id`, display name, source kind (`file` or `clipboard`), nullable `original_path` for file uploads, durable `storage_path` pointing to the bounded saved image file, Base64 thumbnail payload, and `created_at`. A `session_message.message_json` content part with type `image_ref` and the image ID establishes message ownership without duplicating original bytes in SQLite. Unreferenced rows are pending composer images; referenced rows cannot be independently removed. Provider-call projections use resolved image data only for execution; model request content is not retained in durable provider traces.

### `session_tool_use`

One row per tool invocation, keyed by `(turn_id, tool_call_id)`. It records the owning `session_call_id`, tool name, request JSON, result JSON, lifecycle state, ordinal, timestamps, and error code. States are `requested`, `validating`, `policy_check`, `denied`, `awaiting_approval`, `authorized`, `executing`, `succeeded`, `failed`, `timed_out`, `unknown_completion`, and `reconciling`.

### `session_message`

Human-readable messages keyed by `message_id`. Each row links to its session, optionally to a turn and `session_call`, and stores role, message JSON, and `created_at`. Roles are `user`, `assistant`, and `thinking`; the schema rejects `tool`. Message history is ordered by `created_at` with `rowid` as a deterministic tie-breaker; no content sequence or usage column is used. Provider-reported per-call usage belongs to `session_call`; cumulative turn usage belongs to `session_turn`.

## Approvals And Checkpoints

### `session_approval_request`

Durable approval state keyed by `approval_id`, linked to session and the composite `(turn_id, tool_call_id)` in `session_tool_use`. It stores operation, arguments JSON, idempotency key, decision metadata, status, and timestamps.

### `session_checkpoint_manifest`

One client-visible undo unit per turn, linked to session and optionally to `session_turn`, with lifecycle status, expiry, and restore timestamps.

### `session_checkpoint`

Ordered checkpoint metadata linked to a manifest/session and optionally to the composite `(turn_id, tool_call_id)` in `session_tool_use`. Source pre/post images remain owned by the operations package.

## LLM Catalog

### `llm_model_provider`

Built-in or custom provider row keyed by `provider_id`. It stores display name, endpoint, required `adapter_type`, optional plaintext `api_key`, enabled/order state, and timestamps. `adapter_type` names a provider implementation known to `suncode-llm`; supported values are `openai` for OpenAI-compatible endpoints and `anthropic` for the built-in Claude Messages route. API keys never enter DTOs, events, audit payloads, or logs. Opening the immediately previous current-schema database narrowly rebuilds this table when its check constraint permits only `openai`, preserving provider endpoints, credentials, enablement, ordering, and timestamps before selecting the built-in Claude adapter.

### `llm_model`

Model row keyed by `model_id` and associated with `llm_model_provider` by `provider_id`. It stores display/request identifiers, context and auto-compaction token limits, optional output limit, capability flags including `supports_reasoning_effort` and `supports_computer_use`, a comma-separated `reasoning_efforts` catalog, enabled/order state, and timestamps. `auto_compact_tokens` is positive and smaller than `context_tokens`. `supports_computer_use` means the model route may receive a provider-native Computer Use toolset when global enablement and the local backend are also available; it does not grant desktop authority.

## Projection Rules

Durable projection updates occur in one transaction per agent event. The agent rebuilds provider context by merging `session_message` user/assistant/thinking rows with transient tool-role messages derived from succeeded `session_tool_use.result_json` rows, then repairs incomplete assistant/tool tails after interruption. Tool results are never duplicated in `session_message`. Agent events are broadcast in memory only; a client that misses events receives `resync.required` and reloads a fresh session snapshot. Audit rows remain immutable and independent from session projections.
