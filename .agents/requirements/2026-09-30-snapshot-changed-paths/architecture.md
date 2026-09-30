# Architecture

## Current state

`Store::session_conversation_turns` builds each `SessionConversationTurn` from `session_turn` plus per-turn `session_message`, `session_tool_use`, and `session_turn_todo` rows. `checkpoint.captured` events are projected into `session_checkpoint` with `turn_id` and `relative_path`, but no snapshot reader exposes them by turn.

## Proposed design

- Add `changed_paths: Vec<String>` to `SessionConversationTurn` (JSON `changedPaths` via the existing camelCase rename).
- Add `load_changed_paths(connection, turn_id)` beside the checkpoint operations and call it on the same locked connection as the other per-turn loaders.
- Add `session_checkpoint_turn_ordinal_idx` on `(turn_id, ordinal)` so the per-turn read does not scan the table. Schema scripts are idempotent, so existing databases gain the index on next open.
- The Rust SDK facade, C ABI, and `watch_session` already pass the snapshot through serde JSON and need no code change.
- The C# `SessionConversationTurn` record gains an optional trailing `ChangedPaths` parameter.
- The desktop `ProjectSnapshot` folds each turn's paths into the projection's changed-path list.

## Boundaries and dependencies

Only the Rust data crate computes the field. Clients do not open SQLite. No new SDK method, event type, or ABI symbol.

## Data and control flow

`session_checkpoint` → `load_changed_paths` → `SessionConversationTurn.changed_paths` → `session_snapshot` JSON → C ABI envelope → C# `SessionConversationTurn.ChangedPaths` → desktop `SessionSnapshotProjection.ChangedPaths`.

## Security and failure handling

Paths are the validated project-relative arguments already stored for undo; no absolute roots are exposed. A query failure fails the snapshot like the other per-turn loaders.

## Compatibility and migration

Additive JSON field; no schema migration beyond an idempotent index. Older desktop builds ignore it; the new C# parameter defaults to null.

## Risks and rollback

Low. Rollback is removing the field and index; stored data is unchanged.

## Open questions

- None.
