# Changes

## Source

- Added `session_turn.sql`, `session_call.sql`, `session_tool_use.sql`, and `session_message.sql`.
- Removed legacy `turns.sql`, `provider_exchanges.sql`, `tool_calls.sql`, `session_messages.sql`, `suspended_turns.sql`, and `turn_submissions.sql`.
- Updated Store projections, foreign-key references, recovery queries, provider call persistence, tool result persistence, and agent call correlations.

## Contracts and documentation

- Rewrote `contracts/sqlite-schema.md` for the current normalized schema.
- Updated `contracts/persistence.md`, `.agents/ARCHITECTURE.md`, `.agents/features/runtime-phase-1/README.md`, and `.agents/specs/runtime-phase-1.md`.

## Tests

- Added coverage for persisted `thinking` messages and timestamp ordering.
- Existing recovery, provider call, tool exchange, schema, and foreign-key tests now target the new tables.
