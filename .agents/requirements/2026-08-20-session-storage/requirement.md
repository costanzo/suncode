# Requirement

## Background

Session persistence previously split turn admission, turns, provider exchanges, tool calls, messages, and suspended continuations across separate legacy tables. The new system needs one coherent model for a turn, its model calls, tool uses, and readable messages.

## Goals

- Keep `session` as the conversation root.
- Use `session_turn` as the single turn record, including idempotent submission, lifecycle, usage, and approval recovery.
- Store every LLM request in `session_call`.
- Store every tool request, result, and lifecycle state in `session_tool_use`.
- Store user, assistant, thinking, and tool messages in `session_message`, linked to the relevant turn and call.
- Order readable messages by timestamp, without a message content sequence.
- Remove the legacy turn, provider exchange, tool call, suspended turn, turn submission, and session message tables.

## Non-goals

- Database migrations or compatibility conversion.
- Persisting a duplicate session event journal or replay cursor table.
- Changing the public SDK method names in this delivery.

## Acceptance criteria

- A fresh database contains exactly 14 application tables and no legacy session/event tables.
- Approval recovery is represented on `session_turn` and terminal resolution clears its snapshot.
- Provider calls, tool uses, results, and messages are written directly to normalized tables in one transaction.
- `thinking` messages persist and message reads use `created_at` ordering with deterministic ties.
- Live subscriptions do not replay from SQLite; lagged clients receive `resync.required` and reload a snapshot.
