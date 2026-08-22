# Runtime Persistence Contract

Status: Draft for Phase 1 implementation.

The Rust runtime owns SQLite initialization, provider secrets, projections, and operation bookkeeping. Avalonia, providers, and future extensions never open the database directly.

The normative Phase 1 physical table definitions, constraints, and indexes are in `sqlite-schema.md`. Clients consume API DTOs and never depend on those table shapes.

Configuration is stored in one `configuration` table across `global`, `project`, and `session` scopes. Effective reads apply global, project, then session precedence.

Logging policy is durable global configuration rather than process-environment configuration. `log_level`, `log_directory`, `log_max_bytes`, and `log_retention` configure both production loggers; the Avalonia client and Rust runtime write separate `desktop.log` and `runtime.log` files. The database and data directory must still be located before configuration can be read, so data/database path inputs remain bootstrap configuration outside SQLite.

SunCode currently has one schema and no database migration or version metadata. Initialization applies the ordered schema and data manifests transactionally. Reopening the current schema is idempotent; a database with an unexpected application table is rejected without conversion. The current project identity table is singular `project`; a database containing the former `projects` table is therefore incompatible and is not renamed automatically.

## Streams

### Audit

Immutable and long-lived. Records authority decisions only: requested capability, canonical scope, policy result, decision source, grant lifetime, assertion ID, operation outcome, and correlation IDs. It contains no prompt, file content, secret, or provider credential. Audit records are never compacted or rewritten.

### Session content

The normalized session tables are the source of truth: user/assistant/thinking messages, LLM calls, tool uses/results, and turn state live directly in `session_message`, `session_call`, `session_tool_use`, and `session_turn`. `session_message` does not accept the `tool` role or store usage; provider context derives transient tool messages from succeeded `session_tool_use` results. Per-call usage belongs to `session_call`, and cumulative turn usage belongs to `session_turn`. Streaming and lifecycle events are broadcast in memory only. `session_message` history is ordered by `created_at` and does not contain a sequence column. A client that misses live events reloads a fresh session snapshot.

The database stores no client replay cursor or duplicate event journal. Subscription callbacks are live-only; lagged subscribers receive `resync.required` and recover through the snapshot API.

Approval requests and turn-submission idempotency are durable relational state. While pending or resuming, `session_turn.recovery_snapshot_json` preserves canonical messages, model, usage, budgets, the pending call, remaining sibling calls, and the originating submission key. Terminal resolution clears that recovery-only payload while retaining lifecycle metadata. Approval creation is idempotent by operation key, resolution and continuation claims are single-use, and every resolution emits an audit row plus a live event. Turn submissions are represented directly by `session_turn` and remain idempotent across retries.

## Secrets

Provider API keys are classified user secrets. The runtime stores the plaintext value on `llm_model_provider.api_key`. The SQLite data directory and its backups must be treated as sensitive. The key never enters a protocol message, audit record, session content event, log, or client response. Updating a provider key replaces the current value in one row.

`llm_model_provider` and `llm_model` are the durable source for provider endpoints, adapter compatibility, built-in and custom model identities, request model codes, context lengths, auto-compaction thresholds, output limits, capability flags, enabled state, and ordering. Every provider row names a known `suncode-llm` adapter; currently `openai` is the supported OpenAI-compatible adapter and is the default for custom endpoints. `suncode-db` exposes these rows to runtime core; `suncode-llm` remains database-free.

## Retention and compaction

Phase 1 audit retention defaults to 365 days. Normalized session rows are bounded through future per-table retention; there is no duplicate session event stream to compact. Audit data is excluded from session cleanup.

Turn checkpoint manifests retain their Rust-owned file snapshots for 30 days by default. Expiry is projected before clients offer undo. Restored or expired manifests remain as lightweight session metadata after their item payloads become unavailable.

## Recovery

On startup, reconcile normalized in-flight rows and operations by idempotency key. Unknown completion is visible as a terminal-intermediate state; retry requires reconciliation. Filesystem reconciliation compares observed hashes and reports a conflict when the result cannot be attributed safely. Provider calls are not retried after unknown completion unless the provider request is explicitly safe to replay.
