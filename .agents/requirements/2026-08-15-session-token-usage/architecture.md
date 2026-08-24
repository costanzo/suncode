# Architecture

## Current state

`usage.updated` is retained in `session_content`, while `turns.input_tokens`, `output_tokens`, and `total_tokens` remain zero. Qt receives events but has no SDK-owned session aggregate.

## Proposed design

Treat `turns` as the durable usage projection. Applying `usage.updated` replaces the turn's cumulative counters. `Store::session_usage` sums those counters, and `AgentSdk::session_usage` exposes the aggregate through a named C ABI operation. Qt stores only the returned session total and renders it beside the selected model.

## Boundaries and dependencies

Rust owns persistence, migration, aggregation, and the SDK DTO. Qt owns compact formatting and presentation. No client queries SQLite.

## Data and control flow

1. A provider response reports usage.
2. The agent emits durable `usage.updated` with cumulative turn usage.
3. SQLite updates the owning `turns` row in the same append transaction.
4. Qt receives the event and calls `session_usage`.
5. Rust sums the selected session's turn projections and returns a typed DTO.
6. Qt updates the footer property.

## Security and failure handling

Usage contains no credentials or prompts. Invalid, negative, or overflowing counters fail the projection transaction. SDK errors follow existing redacted domain-error handling.

## Compatibility and migration

Schema v12 backfills token counters from the latest retained `usage.updated` event per turn. The C ABI adds one function without changing ABI version 1 or existing functions.

## Risks and rollback

Providers may omit usage, so totals are provider-reported rather than estimated. Rollback removes the additive SDK method and footer property; the token projection remains harmless.

## Open questions

- None.
