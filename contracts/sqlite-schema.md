# Agent SQLite Schema

Status: Current Phase 1 contract.

The `suncode-data` package owns Diesel SQLite connections, transactions, typed table declarations, persistence DTOs, and queries. The `suncode-database` package owns the SQL schema and seed manifests, table-name manifest, and database-file setup under `suncode_database::sqlite`. Table-owned operations live under `agent/crates/data/src/operations/`, with `projection.rs` and `recovery.rs` handling cross-table workflows. Manifest order determines script execution order.

The current schema has 17 application tables.

`configuration`, `language_server`, `llm_model`, `llm_model_provider`, `mcp_server`, `project`, `project_dependency`, `session`, `session_approval_request`, `session_call`, `session_image`, `session_message`, `session_subagent_invocation`, `session_tool_use`, `session_turn`, `session_turn_change`, and `session_turn_todo`.

## Initialization and compatibility

`SCHEMA_VERSION` in `agent/crates/data/src/store.rs` is currently `1`. Every schema change requires an explicit version bump. Opening accepts either a database whose `PRAGMA user_version` equals that constant, or a fresh database with version `0` and zero application tables. Every other version returns `database_schema_incompatible`, including an unversioned database that already contains application tables. The error directs the caller to delete the incompatible database or choose a new data directory. This is an explicit user recovery action.

Each connection configures `foreign_keys=OFF`, WAL journaling, and a 5,000 ms busy timeout. Initialization then performs the following steps in one transaction.

1. Read the stored version and check admission.
2. Execute the ordered `CREATE TABLE IF NOT EXISTS` and index scripts.
3. Compare the sorted application table names with `suncode_database::sqlite::TABLE_NAMES`, excluding SQLite's internal tables. A mismatch fails initialization.
4. Execute the ordered configuration, provider, and model seed scripts.
5. Write `PRAGMA user_version=1` and commit.

An initialization failure rolls back that transaction. Reopening a current database repeats the schema and seed scripts. Seed scripts can add catalog entries and refresh built-in model metadata at the same schema version. Their explicit data updates, including saved Gemini model selections, are distinct from schema-version admission.

The compatibility check covers the stored version and table names. Existing same-version table definitions retain their columns and constraints through `IF NOT EXISTS`; the implementation relies on the version bump rule to identify schema changes. The former ordered migration runner and legacy column/provider-table conversion steps have been removed.

## Conventions

- IDs are non-empty opaque text. Rust generates kind-prefixed UUID v7 encodings for core entities, while built-in agent IDs are stable catalog identifiers.
- Timestamps are UTC RFC 3339 strings with millisecond precision.
- JSON columns must contain valid JSON. Queryable identity, state, ordering, and ownership remain relational columns.
- Foreign-key enforcement is disabled on every connection. Rust validates referenced identifiers at the owning operation boundary. Projects and sessions are archived rather than normally deleted.

## Project And Settings

### `project`

One row per opened local directory tree: `project_id` primary key, non-empty `user_id`, unique `(user_id, canonical_root)`, non-empty `display_name`, lifecycle timestamps, and nullable `archived_at`.

### `project_dependency`

One registered additional working root per row: opaque `dependency_id`, owning `project_id`, canonical root, display name, and creation timestamp. `(project_id, canonical_root)` is unique, and Rust owns referential validation and dependent-row cleanup. Registration DTOs expose stable IDs and display names. Model context advertises canonical roots for file and shell operations under the existing session policy.

### `configuration`

Unified key/value configuration for `global`, `project`, and `session` scopes. Global rows have no owner ID; project rows reference `project`; session rows reference `session`. A CHECK constraint enforces the exact owner shape for each scope, and partial unique indexes enforce one value per key at each scope. Effective reads apply `global < project < session` precedence. The project-aware `default_model` key is stored as a JSON string containing a model ID.

The project-only `tool_call_limit` key is a JSON integer from 1 through 256. When the row is absent, core uses 64. Global TLS settings include `verify_https_certificates`, `use_system_certificates`, and optional PEM/DER `certificate_path`.

Fresh and reopened current databases seed four global logging settings: `log_level` (`"INFO"`), `log_directory` (`""`), `log_max_bytes` (`10485760`), and `log_retention` (`5`), plus global `verify_https_certificates` (`true`), `use_system_certificates` (`true`), `certificate_path` (`""`), `proxy_mode` (`"system"`), `proxy_url`/`proxy_username`/`proxy_password` (`""`), `proxy_bypass` (`[]`), `browser_use_enabled`/`computer_use_enabled` (`false`), and `image_directory` (`""`). An empty log directory means `<data directory>/logs`; an empty image directory means `<data directory>/data/images`. These settings are global-only. The SDK accepts `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`, or `OFF`; a directory string; a maximum size of at least 1024 bytes; a retention count from 0 through 100; boolean HTTPS-verification and Browser Use values; and an image directory string. Disabling verification makes subsequent built-in provider and WebFetch HTTPS requests accept invalid certificate chains and hostnames, but it does not weaken bundled Chromium certificate verification.

## MCP Servers

### `mcp_server`

One row per global MCP server definition. It stores the opaque `mcp_server_id`, unique case-insensitive display name, immutable unique case-insensitive tool prefix, transport kind and versioned transport JSON, desired enabled state, ordering, optimistic revision, and timestamps. Transport JSON is either structured local stdio configuration or remote Streamable HTTP configuration. It may contain plaintext environment/header secrets; read DTOs expose key names only. Runtime connection state and discovered tools are project-scoped memory and are never persisted in this table.

The tool prefix is generated on create and does not change when the display name is edited. Each successful non-idempotent update advances `revision`. The current schema manifest creates this table during initialization.

## Language Servers

### `language_server`

One row per configured project and built-in local-stdio language-server kind. It stores an opaque storage id, a storage-only unique display name, versioned configuration JSON, desired enabled state, ordering, optimistic revision, and timestamps. Version-2 configuration JSON contains the stable project id and server kind plus the structured executable and Rust-derived argument vector, fixed language IDs and initialization options, project-relative root markers, optional Java runtime, managed or custom data path, plaintext environment values, and bounded startup/request timeouts. SDK reads expose only environment key names. Version-1 global free-form rows remain readable for compatibility but are ignored by the built-in catalog. Project runtime state, process handles, negotiated capabilities, diagnostics, and document versions are memory-only and are not persisted.

## Sessions

### `session`

One conversation per row, linked to a project. It stores optional title/model, normalized reasoning effort, `active`/`archived` status, activity timestamps, nullable `pin_at`, and archive consistency checks. `kind` is `primary` or `child`. Primary rows have null parent/agent fields; child rows require `parent_session_id`, the stable built-in `agent_id`, and `agent_version`. A non-null `pin_at` marks a primary session as pinned and records when it was pinned. Active project session listings filter to primary rows; children are listed through their parent.

### `session_subagent_invocation`

One durable delegation correlation per child session. It links the parent session, parent turn, parent `delegate_agent` tool-call ID, child session, built-in agent ID/version, task JSON, snapshotted tool allowlist, and inherited model. State is `created`, `running`, `awaiting_approval`, `completed`, `failed`, `cancelled`, or `interrupted`; result/error and lifecycle timestamps retain the latest normalized outcome. Startup marks `created` or `running` invocations interrupted, while `awaiting_approval` remains durable for explicit resolution.

### `session_turn`

The single source of truth for a turn. It combines turn lifecycle, submission idempotency, input/response/error JSON, model selection, cumulative provider usage, and approval continuation state. States are `admitted`, `queued`, `preparing`, `calling_model`, `resolving_calls`, `compacting`, `completed`, `failed`, `cancelled`, and `interrupted`.

A structured failure write can populate `error_json` and `error_code` after a failed lifecycle projection, but cannot replace completed, cancelled, or interrupted state.

Approval recovery is kept on the turn in `recovery_approval_id`, `recovery_snapshot_json`, `recovery_status`, and recovery timestamps; there is no suspended-turn table. The unique `(session_id, submission_idempotency_key)` constraint makes retries idempotent. Recovery, resuming, and session chronology indexes support startup and history queries.

### `session_turn_todo`

The authoritative current todo list for one turn. Each row is identified by `(turn_id, ordinal)` and stores bounded content, status, priority, creation/update timestamps, and completion time. A successful `todo.updated` event replaces the complete set for that turn transactionally; `session_tool_use.result_json` remains the result of the individual model call and is not used as the progress projection.

### `session_call`

One row per LLM request within a turn. `call_id` is the SunCode-owned physical primary key and is linked to `session_turn`; nullable `provider_request_id` and `provider_response_id` retain the provider's HTTP request identifier and response-object identifier independently. Provider/model/wire-model identity, iteration, lifecycle state, normalized output/tool-call/usage/error JSON, finish reason, and timestamps are retained. Model request messages are intentionally not persisted or exposed through provider trace. Normalized usage may include nullable `cache_read_tokens`, `cache_miss_tokens`, `cache_write_tokens`, and `reasoning_tokens` in addition to input, output, and total tokens; provider-private aliases are not duplicated. States are `started`, `completed`, and `failed`. Session, turn, and in-flight indexes support diagnostics and recovery.

### `session_image`

One persisted image per row, linked to `session`. The row stores opaque `image_id`, display name, source kind (`file` or `clipboard`), nullable `original_path` for file uploads, durable `storage_path` pointing to the bounded saved image file, Base64 thumbnail payload, and `created_at`. A `session_message.message_json` content part with type `image_ref` and the image ID establishes message ownership without duplicating original bytes in SQLite. Unreferenced rows are pending composer images; referenced rows cannot be independently removed. Provider-call projections use resolved image data only for execution; model request content is not retained in durable provider traces.

### `session_tool_use`

One row per tool invocation, keyed by `(turn_id, tool_call_id)`. It records the owning `session_call_id`, tool name, request JSON, result JSON, lifecycle state, ordinal, timestamps, and error code. States are `requested`, `validating`, `policy_check`, `denied`, `awaiting_approval`, `awaiting_question`, `authorized`, `executing`, `succeeded`, `failed`, `timed_out`, `unknown_completion`, and `reconciling`.

### `session_message`

Human-readable messages keyed by `message_id`. Each row links to its session, optionally to a turn and `session_call`, and stores role, message JSON, and `created_at`. Roles are `user`, `assistant`, and `thinking`; the schema rejects `tool`. Message history is ordered by `created_at` with `rowid` as a deterministic tie-breaker; no content sequence or usage column is used. Provider-reported per-call usage belongs to `session_call`; cumulative turn usage belongs to `session_turn`.

Context compaction does not add a table or a `session_message` role. A completed `session_call` with `finish_reason=context_compacted` stores the summary, retained provider messages, and message/tool rowid boundaries in `output_message_json`. Rust selects the latest such row, then appends normalized messages and tool results inserted after those boundaries. Transcript and provider-trace queries continue to read the original rows.

## Approvals and turn file changes

### `session_approval_request`

Durable approval state keyed by `approval_id`, associated with its session and the composite `(turn_id, tool_call_id)` in `session_tool_use`. It stores operation, arguments JSON, idempotency key, decision metadata, status, and timestamps. Rust validates ownership and manages related rows explicitly.

### `session_turn_change`

Dependency file changes use `dependency:<id>/<relative>` in `relative_path`, keeping independent identities for same-named files in different roots. Main-project changes retain root-relative paths. This namespace requires no schema change.

One net file change per `(turn_id, relative_path)`, with owning `session_id` and a nonnegative first-touch `ordinal`. Each row stores nullable `baseline_sha256`/`baseline_stored_on` and `current_sha256`/`current_stored_on` pairs, `is_binary`, nonnegative `additions` and `deletions`, `created_at`, `updated_at`, and `expires_at`. Indexes cover `(session_id, turn_id)` and expiry.

The `checkpoint.captured` projection from `write`, `edit`, and `delete` fixes the first-touch baseline and updates the latest version and counts on subsequent captures. A missing baseline represents creation, and a missing current version represents deletion. A path whose latest hash equals its baseline is removed from the projection, including creation followed by deletion in the same turn. Snapshot `conversationTurns[*].changes` lists unexpired net changes in first-touch order with path, status, counts, and binary state. The SDK `turn_diff(session_id, turn_id)` resolves their stored versions into per-file hunks.

The operations package stores full versions outside SQLite as gzip-compressed bytes at `<checkpoint root>/<UTC YYYY-MM-DD>/<sha256>.gz`, using a digest of the uncompressed content. A row expires at midnight UTC after 30 full days following its earliest initially referenced storage day. Reads hide expired rows immediately. Background retention deletes expired rows and then removes storage-day directories older than the 30-day cutoff. Session deletion removes change rows transactionally and releases blobs after their remaining references have been checked.

## LLM Catalog

### `llm_model_provider`

Built-in or custom provider row keyed by `provider_id`. It stores display name, endpoint, default endpoint, required `adapter_type`, credential metadata (`credential_mode`, nullable `secret_ref`, nullable `api_key`, and nullable `api_key_mask`), enabled/order state, and timestamps. Production accepts only `keychain`, stores a UUID reference and mask, and keeps the complete API key in the platform credential store. The nullable `api_key` column and original CHECK constraint remain for layout compatibility. Production writes keep `api_key` NULL and reject legacy `plaintext` modes on read or write. No plaintext-key migration is provided. `adapter_type` names a provider implementation known to `suncode-llm`; supported values are `openai` for OpenAI Responses-compatible endpoints, `chat_completions` for the Tencent Hunyuan route, and `anthropic` for the built-in Claude Messages route. API keys never enter DTOs, events, audit payloads, or logs. Schema-version admission follows the initialization rules above. Same-version definitions are subject to the seed and operation queries that use their credential columns.

### `llm_model`

Model row keyed by `model_id` and associated with `llm_model_provider` by `provider_id`. It stores display/request identifiers, context and auto-compaction token limits, optional output limit, capability flags including `supports_reasoning_effort` and `supports_computer_use`, a comma-separated `reasoning_efforts` catalog, enabled/order state, and timestamps. `auto_compact_tokens` is positive and smaller than `context_tokens`. `supports_computer_use` advertises support for SunCode's provider-neutral Computer Use function tools, requiring function calling, vision, and multimodal function results. Global enablement, host capabilities, backend availability, and policy still govern execution.

## Projection Rules

Durable projection updates occur in one transaction per agent event. The agent rebuilds provider context by merging `session_message` user/assistant/thinking rows with transient tool-role messages derived from succeeded `session_tool_use.result_json` rows, then repairs incomplete assistant/tool tails after interruption. Tool results are never duplicated in `session_message`. Agent events are broadcast in memory only; a client that misses events receives `resync.required` and reloads a fresh session snapshot. Durable approval decisions and tool outcomes remain in their normalized owning tables.
