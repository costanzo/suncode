# Requirement

**Status: Complete**

## Background

The agent needs a structured, user-visible task list for multi-step work within the current turn. The OpenCode `todowrite` tool provides the reference interaction: each call replaces the complete list and returns the resulting list to the model.

## Goals

- Advertise a model-facing `todowrite` tool with structured task status and priority.
- Keep the list in the active turn continuation and feed the latest list back into the next model request.
- Emit a live update and expose the latest completed tool result through the normalized session snapshot.
- Show the current turn's list in the Avalonia agent sidebar.

## Non-goals

- A durable cross-turn todo database or project task manager.
- Client-authored todo mutation APIs.
- Migration or compatibility behavior for retired tool names.

## Requirements

1. The tool accepts a complete replacement list of at most 100 todos.
2. Each todo requires non-empty `content`, `status` (`pending`, `in_progress`, `completed`, or `cancelled`), and `priority` (`high`, `medium`, or `low`).
3. At most one todo may be `in_progress` at a time.
4. Successful calls emit `todo.updated`, persist the normal tool result, and continue the same turn without approval.
5. Todo state is scoped to the current turn, stored in `session_turn_todo`, and restored from the normalized turn snapshot after a reload.

## Edge cases

- An empty list clears the current turn's visible todos.
- Invalid list shape, item fields, or duplicate active items is returned as a recoverable `invalid_arguments` tool error.
- Completed and cancelled items remain visible until the model replaces or clears the list.

## Acceptance criteria

- Rust registry, validation, execution, and continuation tests pass.
- Avalonia restores the latest `todowrite` result from a session snapshot and updates from `todo.updated`.
- Rust workspace tests, Avalonia tests, formatting, and diff validation pass.

## Open questions

- None.
