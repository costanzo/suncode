# Changes

## Source

- Added the `todowrite` schema and exact built-in registry entry.
- Added Rust validation, current-turn continuation state, result messages, and `todo.updated` events.
- Classified `todowrite` as read-only and added Avalonia todo projection and sidebar presentation.

## Contracts and generated artifacts

- Documented the live `todo.updated` event and normalized tool-use result behavior in the embedded SDK contract.

## Configuration and persistence

- Added `session_turn_todo` as the authoritative current-turn progress projection. `session_tool_use` still retains the tool result history but is not used as the todo state store.

## Tests

- Added Rust validation coverage.
- Added Avalonia snapshot and live-event projection coverage.

## Documentation

- Updated current agent and desktop feature/specification facts.
