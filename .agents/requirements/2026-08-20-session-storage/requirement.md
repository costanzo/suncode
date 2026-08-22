# Requirement

## Background

Session persistence previously split turn admission, turns, provider exchanges, tool calls, messages, and suspended continuations across separate legacy tables. The new system needs one coherent model for a turn, its model calls, tool uses, and readable messages.

## Goals

- Keep `session` as the conversation root.
- Use `session_turn` as the single turn record, including idempotent submission, lifecycle, usage, and approval recovery.
- Store every LLM request in `session_call`.
- Retain nullable provider HTTP request and response-object identifiers independently on each model call.
- Store every tool request, result, and lifecycle state in `session_tool_use`.
- Store user, assistant, and thinking messages in `session_message`, linked to the relevant turn and call.
- Keep tool request, result, state, and ordering exclusively in `session_tool_use`; do not duplicate tool-result messages in `session_message`.
- Order readable messages by timestamp, without a message content sequence.
- Remove the legacy turn, provider exchange, tool call, suspended turn, turn submission, and session message tables.

## Non-goals

- Database migrations or compatibility conversion.
- Persisting a duplicate session event journal or replay cursor table.
- Changing the public SDK method names in this delivery.

## Acceptance criteria

- A fresh database contains exactly 13 application tables and no legacy session/event tables.
- Approval recovery is represented on `session_turn` and terminal resolution clears its snapshot.
- Provider calls, tool uses/results, and human-readable messages are written directly to their normalized owner tables in one transaction.
- Provider request IDs from recognized response headers and provider response IDs from streamed response objects are exposed by model-call traces when reported.
- `session_message.role` rejects `tool`, while provider context reconstructs completed tool-result messages from `session_tool_use` when needed.
- `session_message` contains no usage column; per-call usage belongs to `session_call` and cumulative turn usage belongs to `session_turn`.
- `thinking` messages persist and message reads use `created_at` ordering with deterministic ties.
- Live subscriptions do not replay from SQLite; lagged clients receive `resync.required` and reload a snapshot.
