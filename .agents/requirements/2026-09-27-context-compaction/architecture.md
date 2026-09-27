# Architecture

## Current state

The core builds provider messages from normalized `session_message` and `session_tool_use` rows. The prior compaction only replaced the in-memory continuation.

## Proposed design

The Rust context builder calculates a bounded retained tail and an initial local summary. The selected provider may replace that summary with valid structured JSON within a 30-second limit. Its call is projected through the existing provider-exchange events with actual usage or failure. The existing `context.compacted` projection writes an internal `session_call` checkpoint holding the summary, retained messages after stripping transient image bytes, and rowid boundaries; its usage remains null. The public event carries only summary metadata. Later context reconstruction uses the latest checkpoint and post-boundary rows.

## Boundaries and dependencies

Core owns summary generation and request budgeting. Data owns SQLite checkpoint projection and replay. Provider adapters remain unaware of compaction semantics.

## Data and control flow

The core compacts before a provider request, persists the checkpoint under the session event gate, then submits the shortened request. A recognized context overflow permits one forced compaction and retry. Later turns load the latest durable checkpoint.

## Security and failure handling

Summary input is explicitly untrusted. Failed, partial, invalid, or oversized model summaries do not replace the local summary. Audit rows remain unchanged. The extra summary request uses the selected configured provider.

## Compatibility and migration

The 18-table schema and SDK DTOs remain unchanged. The new checkpoint fields live in `session_call.output_message_json`. Earlier compaction rows without rowid boundaries are not expected from a shipped durable compaction implementation.

## Risks and rollback

Heuristic token estimates can differ from provider tokenization. The one-time provider overflow path bounds repeated requests. Rolling back the code leaves internal checkpoint rows but does not alter transcript records.

## Open questions

- Dedicated compaction model selection and exact provider token counting are deferred.
