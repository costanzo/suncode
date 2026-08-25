# Architecture

`todowrite` is a special read-only model tool implemented at the Rust agent boundary. It does not dispatch to the audited machine-operation layer and does not require approval. The agent validates the complete replacement list, stores it in the in-memory continuation for the active turn, emits `todo.updated`, and appends the JSON result as a tool-role message for the next provider request.

The continuation's todo field carries the list while the turn is executing and recovering. Each successful `todo.updated` event transactionally replaces the rows for that turn in `session_turn_todo`; this table is the durable progress source. The normal `session_tool_use.result_json` remains an immutable record of what that tool call returned and is not used to derive current progress. Avalonia reads `conversationTurns[*].todos` from the snapshot and applies live `todo.updated` events to its presentation collection.

## Open questions

- None.
