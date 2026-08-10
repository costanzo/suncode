# Runtime Persistence Contract

Status: Draft for Phase 1 implementation.

The Rust runtime owns SQLite, migrations, secret records, projections, and operation bookkeeping. Qt, providers, and future extensions never open the database directly.

The normative Phase 1 physical table definitions, constraints, and indexes are in `sqlite-schema.md`. Clients consume API DTOs and never depend on those table shapes.

## Streams

### Audit

Immutable and long-lived. Records authority decisions only: requested capability, canonical scope, policy result, decision source, grant lifetime, assertion ID, operation outcome, and correlation IDs. It contains no prompt, file content, secret, or provider credential. Audit records are never compacted or rewritten.

### Session content

Compactable source of truth for conversation projections. Records user and assistant messages, bounded deltas, tool requests/results, turn and tool-call transitions, context summaries, usage summaries, checkpoints, and artifact references. Each session has a strictly increasing `content_sequence` assigned transactionally.

### Client sync

Disposable cursors and snapshot metadata for reconnect optimization. It is rebuilt from session content and is never authoritative.

Approval requests, suspended-turn snapshots, and turn-submission idempotency records are durable relational state. A snapshot preserves canonical messages, model, usage, budgets, the pending call, remaining sibling calls, and the originating turn-submission key. Approval creation is idempotent by operation key, resolution and continuation claims are single-use, and every resolution emits audit plus session-content events. Turn submissions are keyed by session and idempotency key: they remain pending while awaiting approval, then hold the resumed result or authorization failure without executing twice.

## Secrets

Provider API keys are classified user secrets. The runtime stores the plaintext value, provider ID, format marker, key version, creation time, and optional invalidation time in SQLite `secret_records`. The SQLite data directory and its backups must be treated as sensitive. The key never enters a protocol message, audit record, session content event, log, or client response. Rotation invalidates the old secret reference before the new one is activated.

## Retention and compaction

Phase 1 defaults are: audit retention 365 days, session content retention 90 days, client sync retention 24 hours. Retention is configurable per stream. Compaction may summarize old session content but must retain the latest user intent, unresolved approvals, active checkpoint metadata, terminal tool outcomes, and a recoverable summary event. Audit data is excluded from compaction.

Turn checkpoint manifests retain their Rust-owned file snapshots for 30 days by default. Expiry is projected before clients offer undo. Restored or expired manifests remain as lightweight session metadata after their item payloads become unavailable.

## Recovery

On startup, rebuild projections from session content and reconcile in-flight operations by idempotency key. Unknown completion is visible as a terminal-intermediate state; retry requires reconciliation. Filesystem reconciliation compares observed hashes and reports a conflict when the result cannot be attributed safely. Provider calls are not retried after unknown completion unless the provider request is explicitly safe to replay.
