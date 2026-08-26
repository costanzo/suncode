# Persistence And SDK Contracts Phase 1

**Status:** Implemented and focused-tested

Rust is the sole database owner. `suncode-database` owns SQLite resources, manifests, seed data, and database-file setup; `suncode-data` owns Diesel connections, ORM declarations, persistence DTOs, and table operations. Core consumes the data package as a library.

## Current storage model

Phase 1 has one current 15-table schema with no schema-version metadata or general migration runner. Initialization applies schema and seed manifests transactionally, is idempotent for the current schema, and rejects unexpected or incompatible application tables without conversion. The one explicit additive bootstrap is creation of an empty `project_dependency` table for an otherwise-current 13-table database.

The normalized tables are:

`project`, `project_dependency`, `configuration`, `session`, `session_turn`, `session_turn_todo`, `session_call`, `session_tool_use`, `session_message`, `audit_record`, `approval_request`, `checkpoint_manifest`, `checkpoint`, `llm_model_provider`, and `llm_model`.

Session messages, calls, tool uses, approvals, checkpoints, settings, credentials, and recovery snapshots are durable source data. Streaming events are live-only. `session_turn` owns admission, lifecycle, cumulative usage, and approval recovery; `session_turn_todo` owns the current todo projection; `session_call` owns provider diagnostics and normalized usage; `session_tool_use` owns tool lifecycle; and `session_message` stores timestamp-ordered user, assistant, and thinking content.

## SDK contract

The embedded SDK exposes named methods through C ABI version 4, opaque handles, explicit ownership/free functions, UTF-8 JSON DTO payloads where appropriate, and direct live subscriptions. Hosts receive domain errors rather than HTTP statuses. The authoritative hand-written contract is [`contracts/agent-sdk/README.md`](../../../contracts/agent-sdk/README.md); storage and retention rules are [`contracts/persistence.md`](../../../contracts/persistence.md) and [`contracts/sqlite-schema.md`](../../../contracts/sqlite-schema.md).

Provider traces expose normalized calls, messages, tool uses, timing, finish state, provider request/response identifiers, token usage, and cache/reasoning counters without credentials or raw authorization headers. Contract behavior is verified by focused Rust and Avalonia tests.
