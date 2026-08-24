# Changes

## Source

- Added `session_turn.sql`, `session_call.sql`, `session_tool_use.sql`, and `session_message.sql`.
- Removed legacy `turns.sql`, `provider_exchanges.sql`, `tool_calls.sql`, `session_messages.sql`, `suspended_turns.sql`, and `turn_submissions.sql`.
- Updated Store projections, foreign-key references, recovery queries, provider call persistence, tool result persistence, and agent call correlations.
- Removed the `tool` role from `session_message`, stopped projecting `message.tool` into that table, and rebuilt transient provider tool messages from succeeded `session_tool_use` results.
- Persisted normalized tool results and per-call ordinals on `session_tool_use` so reconstructed provider context matches the live agent context.
- Removed duplicate message-level `usage_json` storage and the corresponding trace-message DTO field; call and turn usage remain in their authoritative tables.
- Added nullable provider request/response identifiers to `session_call`, captured them from OpenAI-compatible response headers and SSE objects, and exposed them in the Trace panel.

## Contracts and documentation

- Rewrote `contracts/sqlite-schema.md` for the current normalized schema.
- Updated `contracts/persistence.md`, `.agents/ARCHITECTURE.md`, `.agents/features/agent-phase-1/README.md`, and `.agents/specs/agent-phase-1.md`.

## Tests

- Added coverage for persisted `thinking` messages and timestamp ordering.
- Added coverage that the schema rejects the retired tool-message role, tool events create no message rows, and completed tool exchanges are reconstructed from `session_tool_use` for later provider context.
- Added coverage that current `session_message` omits `usage_json` and rejects databases that still contain it.
- Added coverage for provider request/response ID parsing, persistence, and current-schema validation.
- Existing recovery, provider call, tool exchange, schema, and foreign-key tests now target the new tables.
