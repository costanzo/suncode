# Agent Persistence Contract

Status: Current Phase 1 contract.

The Rust agent owns SQLite initialization, provider secrets, projections, and operation bookkeeping. Avalonia, providers, and future extensions never open the database directly.

The normative Phase 1 physical table definitions, constraints, and indexes are in `sqlite-schema.md`. Clients consume API DTOs and never depend on those table shapes.

Configuration is stored in one `configuration` table across `global`, `project`, and `session` scopes. Effective reads apply global, project, then session precedence. `tool_call_limit` is project-only, accepts JSON integers from 1 through 256, and defaults in core to 64 when absent.

Logging policy is durable global configuration rather than process-environment configuration. `log_level`, `log_directory`, `log_max_bytes`, and `log_retention` configure both production loggers; the Avalonia client and Rust agent write separate `desktop.log` and `agent.log` files. Session-image storage is also durable global configuration through `image_directory`, where an empty string means `<data directory>/data/images`. The database and data directory must still be located before configuration can be read, so data/database path inputs remain bootstrap configuration outside SQLite.

Both loggers initialize a default file before SQLite settings are loaded, flush each record, and retain startup, lifecycle, SDK, subscription, turn, and unhandled-exception diagnostics. Error records include operation names, safe correlation IDs, error codes, retryability, and provider request IDs when available. They must not include API keys, authorization headers, prompts, model responses, tool arguments/results, file contents, or raw provider payloads. Exception text is normalized to one line and bounded; logging failures fall back to stderr without replacing the original failure.

HTTPS server verification is durable global configuration. `verify_https_certificates` is a JSON boolean that defaults to `true`. When `true`, built-in Rust HTTPS clients validate the server certificate chain and hostname. When `false`, subsequent built-in provider and WebFetch requests accept invalid certificates and hostnames, equivalent to the TLS verification behavior of `curl -k`. The insecure value does not bypass URL validation, redirect restrictions, network approval, credential redaction, or other policy checks.

SunCode currently has one 16-table schema and no database migration or version metadata. Initialization applies the ordered schema and data manifests transactionally. Reopening the current schema is idempotent; a database with an unexpected application table is rejected without conversion. As explicit additive bootstrap extensions, initialization adds the empty `project_dependency` table to an otherwise-current 13-table database and the empty `session_image` table to an otherwise-current 15-table database before validating the 16-table manifest. It does not provide a general migration mechanism. The current project identity table is singular `project`; a database containing the former `projects` table is therefore incompatible and is not renamed automatically.

Project source dependencies are stored in `project_dependency`. Rust persists their canonical roots, rejects self/nested/overlapping roots at the SDK boundary, and deletes registrations with the owning project. The SDK never exposes a dependency's absolute root to Avalonia or to the model; external code refers to it by an opaque dependency ID.

## Streams

### Audit

Immutable and long-lived. Records authority decisions only: requested capability, canonical scope, policy result, decision source, grant lifetime, assertion ID, operation outcome, and correlation IDs. It contains no prompt, file content, secret, or provider credential. Audit records are never compacted or rewritten.

### Session content

The normalized session tables are the source of truth: user/assistant/thinking messages, LLM calls, tool uses/results, turn state, and session-owned images live directly in `session_message`, `session_call`, `session_tool_use`, `session_turn`, and `session_image`. `session_message` does not accept the `tool` role or store usage; provider context derives transient tool messages from succeeded `session_tool_use` results. Per-call usage belongs to `session_call`, including nullable normalized cache-read, cache-miss, cache-write, and reasoning token counts when reported; cumulative turn usage belongs to `session_turn` and includes only input, output, and total tokens. A submitted user message owns images through `image_ref` content parts containing opaque `session_image.image_id` values. Full image bytes remain in bounded files outside SQLite; provider traces retain only a redacted attachment marker. Images without a message reference are pending composer state, while referenced images remain durable conversation content and cannot be removed independently. Streaming and lifecycle events are broadcast in memory only. `session_message` history is ordered by `created_at` and does not contain a sequence column. A client that misses live events reloads a fresh session snapshot.

The database stores no client replay cursor or duplicate event journal. Subscription callbacks are live-only; lagged subscribers receive `resync.required` and recover through the snapshot API.

Approval requests and turn-submission idempotency are durable relational state. While pending or resuming, `session_turn.recovery_snapshot_json` preserves canonical messages, model, usage, budgets, the pending call, remaining sibling calls, and the originating submission key. The snapshotted tool-call limit defaults to 64 when reading a legacy continuation. Terminal resolution clears that recovery-only payload while retaining lifecycle metadata. A final failure write may enrich an already failed turn with structured `error_json` and `error_code`, but it never overwrites completed, cancelled, or interrupted state. Approval creation is idempotent by operation key, resolution and continuation claims are single-use, and every resolution emits an audit row plus a live event. Turn submissions are represented directly by `session_turn` and remain idempotent across retries.

## Secrets

Provider API keys are classified user secrets. The agent stores and resolves the plaintext value exclusively through `llm_model_provider.api_key`; provider credential environment variables are not a fallback or override. The SQLite data directory and its backups must be treated as sensitive. The key never enters a protocol message, audit record, session content event, log, or client response. Updating a provider key replaces the current value in one row.

`llm_model_provider` and `llm_model` are the durable source for provider endpoints, adapter compatibility, built-in and custom model identities, request model codes, context lengths, auto-compaction thresholds, output limits, capability flags including reasoning-effort support, enabled state, and ordering. Every provider row names a known `suncode-llm` adapter; currently `openai` is the supported OpenAI-compatible adapter and is the default for custom endpoints. `suncode-data` exposes these rows to agent core; `suncode-llm` and `suncode-database` remain database-driver independent.

The named SDK provider-endpoint update modifies only the endpoint of an existing provider row. It validates an absolute HTTP or HTTPS URL with a host and rejects embedded credentials, queries, and fragments. The update preserves the provider's API key and all other provider/model fields, then changes the live Rust route for subsequent calls without requiring a process restart.

## Retention and compaction

Phase 1 audit retention defaults to 365 days. Normalized session rows are bounded through future per-table retention; there is no duplicate session event stream to compact. Audit data is excluded from session cleanup.

Turn checkpoint manifests retain their Rust-owned file snapshots for 30 days by default. Expiry is projected before clients offer undo. Restored or expired manifests remain as lightweight session metadata after their item payloads become unavailable.

## Recovery

On startup, reconcile normalized in-flight rows and operations by idempotency key. Unknown completion is visible as a terminal-intermediate state; retry requires reconciliation. Filesystem reconciliation compares observed hashes and reports a conflict when the result cannot be attributed safely. Provider calls are not retried after unknown completion unless the provider request is explicitly safe to replay.
