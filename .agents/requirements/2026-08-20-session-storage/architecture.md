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
session_message (user)
```

`session_turn` contains the former submission and suspended continuation fields. `session_call` uses the SunCode-generated `call_id` as its physical key; nullable `provider_request_id` and `provider_response_id` separately preserve the provider HTTP request ID and streamed response-object ID. The existing provider-exchange DTO and SDK method remain compatibility names. `session_tool_use` uses `(turn_id, tool_call_id)` as its stable physical key and stores request/result JSON. `session_message` stores only user, assistant, and thinking roles, has optional `turn_id` and `session_call_id` correlations, and has neither a content sequence nor usage JSON. Per-call usage is owned by `session_call`; cumulative turn usage is owned by `session_turn`.

The database does not duplicate tool results as message rows. When the agent starts a later turn, the database read model merges timestamp-ordered user/assistant/thinking rows with succeeded `session_tool_use.result_json` rows and constructs transient provider-role `tool` messages. Incomplete assistant tool-call tails are still removed before provider use.

## Boundaries and failure handling

Each runtime event updates its normalized projection in the same SQLite transaction that updates session activity. Tool result events update `result_json`; approval and checkpoint foreign keys validate tool ownership through `session_tool_use`. Live subscriptions do not have a durable cursor; lag produces `resync.required`.

## Compatibility and migration

This is a fresh system. Legacy tables are removed from the current manifest and existing incompatible databases are rejected rather than migrated. A `session_message` table whose role constraint still permits `tool` or that still contains `usage_json` is incompatible with the current schema.
