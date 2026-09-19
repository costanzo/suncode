# Persistence And SDK Contracts Phase 1

**Status:** Implemented and focused-tested

Rust is the sole database owner. `suncode-database` owns SQLite resources, manifests, seed data, and database-file setup; `suncode-data` owns Diesel connections, ORM declarations, persistence DTOs, and table operations. Core consumes the data package as a library.

## Current storage model

Phase 1 has one current 18-table schema with no schema-version metadata or general migration runner. Initialization applies schema and seed manifests transactionally, is idempotent for the current schema, supports the narrow additive language-server, MCP, and built-in-agent compatibility steps, and rejects unexpected or incompatible application tables without conversion.

The normalized tables are:

`project`, `project_dependency`, `configuration`, `session`, `session_turn`, `session_turn_todo`, `session_call`, `session_tool_use`, `session_message`, `session_image`, `approval_request`, `checkpoint_manifest`, `checkpoint`, `llm_model_provider`, `llm_model`, `mcp_server`, `language_server`, and `subagent_invocation`.

Session messages, calls, tool uses, approvals, checkpoints, settings, credentials, recovery snapshots, child invocation correlations, and session-owned images are durable source data. Streaming events are live-only. `session_turn` owns admission, lifecycle, cumulative usage, and approval recovery; `subagent_invocation` owns primary/child delegation state; `session_turn_todo` owns the current todo projection; `session_call` owns provider diagnostics and normalized usage; `session_tool_use` owns tool lifecycle; `session_message` stores timestamp-ordered user, assistant, and thinking content plus message-owned `image_ref` parts; and `session_image` stores image metadata plus thumbnails while bounded full image bytes remain on disk. Unreferenced image rows are pending composer state; referenced rows remain attached to their user message.

## SDK contract

The embedded SDK exposes an async-first Rust facade plus an explicit runtime-owning blocking adapter. Its typed session subscription implements standard fused Rust stream semantics while retaining direct receive methods. Consuming shutdown drains runtime-owned resources and session streams before releasing SQLite ownership and the single-instance data-directory lock; final native handle release invokes the same blocking cleanup. The current C ABI consumes the blocking adapter through opaque handles, explicit ownership/free functions, UTF-8 JSON DTO payloads where appropriate, and direct live subscriptions. Atomic session watch returns snapshot JSON with a dormant subscription handle whose callback delivery starts explicitly after the host applies the snapshot. Hosts receive domain errors rather than HTTP statuses. The authoritative hand-written contract is [`contracts/agent-sdk/README.md`](../../../contracts/agent-sdk/README.md); storage and retention rules are [`contracts/persistence.md`](../../../contracts/persistence.md) and [`contracts/sqlite-schema.md`](../../../contracts/sqlite-schema.md).

Provider traces expose normalized calls, messages, tool uses, timing, finish state, provider request/response identifiers, token usage, and cache/reasoning counters without credentials or raw authorization headers. Contract behavior is verified by focused Rust and Avalonia tests.
