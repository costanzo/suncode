# Architecture

## Current state

The Rust database package owns SQLite initialization and normalized session tables. Runtime events are broadcast in memory; snapshots serve session history, context construction, diagnostics, approvals, and undo.

## Proposed design

```text
session
  |- session_turn
       |- session_call
       |    `- session_message (assistant/thinking)
       `- session_tool_use
            `- session_message (tool)
session_message (user)
```

`session_turn` contains the former submission and suspended continuation fields. `session_call` uses `call_id` as its physical key; the existing provider-exchange DTO and SDK method remain compatibility names. `session_tool_use` uses `(turn_id, tool_call_id)` as its stable physical key and stores request/result JSON. `session_message` has optional `turn_id` and `session_call_id` correlations and no content sequence.

## Boundaries and failure handling

Each runtime event updates its normalized projection in the same SQLite transaction that updates session activity. Tool result events update `result_json`; approval and checkpoint foreign keys validate tool ownership through `session_tool_use`. Live subscriptions do not have a durable cursor; lag produces `resync.required`.

## Compatibility and migration

This is a fresh system. Legacy tables are removed from the current manifest and existing incompatible databases are rejected rather than migrated.
