# Persistence And SDK Contracts Phase 1

**Status:** Implemented and focused-tested

Rust is the sole database owner. `suncode-database` owns SQLite resources, manifests, seed data, and database-file setup; `suncode-data` owns Diesel connections, ORM declarations, persistence DTOs, and table operations. Core consumes the data package as a library.

## Current storage model

Phase 1 has one current 17-table schema at `SCHEMA_VERSION = 1`. Startup accepts the exact stored `PRAGMA user_version`, or version `0` with zero application tables. A different version returns `database_schema_incompatible` and requires an explicit reset or a new data directory. Initialization checks admission, executes schema scripts, validates the exact table-name manifest, applies seed scripts, and records the version transactionally. Schema changes require a version bump. The former migration runner and legacy conversion steps have been removed. Existing same-version definitions rely on that versioning discipline because `IF NOT EXISTS` preserves their columns and constraints.

The normalized tables are listed in `contracts/sqlite-schema.md`. Provider adapters are `openai` for Responses-compatible routes, `chat_completions` for Tencent Hunyuan, and `anthropic` for Claude Messages. Current-version seed scripts add missing catalog rows and refresh built-in model metadata while preserving provider endpoint and credential configuration.

Session messages, calls, tool uses, approvals, turn file changes, settings, credentials, recovery snapshots, child invocation correlations, and session-owned images are durable source data. Streaming events are live-only. `session_turn` owns admission, lifecycle, cumulative usage, and approval recovery. `session_subagent_invocation` owns primary/child delegation state, `session_turn_todo` owns the current todo projection, `session_call` owns provider diagnostics and normalized usage, and `session_tool_use` owns tool lifecycle. `session_message` stores timestamp-ordered user, assistant, and thinking content plus message-owned `image_ref` parts. `session_image` stores image metadata and thumbnails, while bounded full image bytes remain on disk. Unreferenced images are pending composer state.

`session_turn_change` stores one first-touch baseline and latest version per turn/path for `write`, `edit`, and `delete`. Net-zero changes disappear. Snapshot `changes` and SDK `turn_diff` expose retained changes and hunks. Content-addressed gzip versions live in UTC day folders outside SQLite, with 30-day retention and explicit reference checks during session deletion. Child turns retain their own change projections.

## Verification status

Fresh database creation/reopening, catalog seeding, and SDK routing have focused passing tests. Five existing data-layer tests still assume the removed compatibility paths or the previous newer-version error wording. They currently fail and require alignment with exact-version admission before the data-layer suite can pass in full.

## SDK contract

The embedded SDK exposes an async-first Rust facade plus an explicit runtime-owning blocking adapter. Its typed session subscription implements standard fused Rust stream semantics while retaining direct receive methods. Consuming shutdown drains runtime-owned resources and session streams before releasing SQLite ownership and the single-instance data-directory lock; final native handle release invokes the same blocking cleanup. The current C ABI consumes the blocking adapter through opaque handles, explicit ownership/free functions, UTF-8 JSON DTO payloads where appropriate, and direct live subscriptions. Atomic session watch returns snapshot JSON with a dormant subscription handle whose callback delivery starts explicitly after the host applies the snapshot. Hosts receive domain errors rather than HTTP statuses. The authoritative hand-written contract is [`contracts/agent-sdk/README.md`](../../../contracts/agent-sdk/README.md); storage and retention rules are [`contracts/persistence.md`](../../../contracts/persistence.md) and [`contracts/sqlite-schema.md`](../../../contracts/sqlite-schema.md).

Provider traces expose normalized calls, messages, tool uses, timing, finish state, provider request/response identifiers, token usage, and cache/reasoning counters without credentials or raw authorization headers. Provider API keys use the Rust-owned `suncode-secrets` boundary: system keychain by default, or explicit provider-scoped SQLite plaintext. Contract behavior is verified by focused Rust and Avalonia tests.
