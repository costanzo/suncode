# Test Plan

## Scope

Snapshot contract field from storage through the desktop projection.

## Unit tests

- Rust data: per-turn paths, deduplication within a turn, ordering, empty-path and null-path exclusion, turns without checkpoints.
- C# SDK: deserializes `changedPaths`; missing field yields null.
- Desktop: projection folds multiple turns without duplicates.

## Integration and conformance tests

- Rust SDK facade `session_snapshot` JSON contains `conversationTurns[*].changedPaths`.

## Regression checks

- Existing snapshot, watch, and projection tests.

## Manual checks

- Reopen a session whose turn wrote files; the Review panel lists them.

## Commands and results

- Recorded in `progress.md` on completion.

## Residual risks

- Paths are only as complete as checkpoint coverage; tools that do not checkpoint are not listed.
