# Architecture

## Current state
The Rust `Agent` runs one locked turn per session. A second submit waits on the same session lock and becomes an independent turn after the first finishes. Tool calls are executed one by one as they are encountered. Context compaction uses a fixed character threshold and a fixed recent message count.

## Proposed design
Adopt three PI Agent ideas in SunCode-native form:

- Running-turn submit queue: if a session lock is busy and an active turn is known, the submit is accepted into an in-memory queue and returns `status: queued`.
- Batch preflight and read-only parallelism: the agent validates and authorizes calls from one assistant message before executing eligible calls. Batches made only of read-only operations execute concurrently; writes, process execution, and approval continuations keep the existing sequential execution.
- Token-window compaction: context construction estimates tokens from content, compacts when estimated context exceeds `model_window - reserve`, and retains a recent tail by estimated tokens.

## Boundaries and dependencies
The queue is owned by `agent/crates/core/src/agent.rs` and is intentionally in-memory. SQLite schema, provider adapters, operations, and Qt DTO ownership stay unchanged. Model limits come from the existing runtime model catalog.

## Data and control flow
1. `submit` validates the model, tries to acquire the session lock, and queues the message if the lock is busy.
2. The active turn drains queued messages before provider calls and again before completing a no-tool assistant response.
3. `resolve_calls` validates and policy-checks a batch, then executes accumulated allowed calls.
4. `context::build_for_model` compacts using the active model input limit and SunCode's conservative 64k default context window.

## Security and failure handling
Approval still precedes risky execution. Parallel execution is limited to read-only operations, so checkpoint ordering and write rollback are unaffected. Queued messages are cleared on active-turn failure, cancellation, or approval denial because the queue is not durable.

## Compatibility and migration
No database migration is required. The HTTP/API response union gains `status: queued`; existing completed and approval responses are unchanged. Qt status text recognizes queued responses but the active-turn composer behavior is otherwise unchanged.

## Risks and rollback
The in-memory queue is intentionally not crash-resumable. Rollback is local to `agent.rs`, `context.rs`, and the queued response handling in the API adapter.

## Open questions
Durable queue storage should be reconsidered with any future lane/tree or crash-resumable active-turn work.
