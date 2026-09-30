# Requirement

## Background

The desktop Review panel lists files the agent changed. Live `checkpoint.captured` events add their `path`, but a session opened or resumed from `session_snapshot` restores an empty list. The typed `SessionSnapshot` introduced on 2026-09-13 carries no changed paths, and the earlier event-derived collection loop was removed without a replacement.

## Goals

- `session_snapshot` and `watch_session` return, for each conversation turn, the project-relative paths the agent checkpointed during that turn.
- The desktop restores its changed-file list from the snapshot so reopening a session matches the live view.

## Non-goals

- Changing which tools create checkpoints, or recording paths for `bash` or Computer Use.
- Changing the desktop from a session-wide list to a per-turn list, or its wording.
- Remote, mobile, and CLI presentation.

## Requirements

1. Each `conversationTurns[*]` item has a `changedPaths` array of strings, always present, empty when the turn changed nothing.
2. Paths come from Rust-owned `session_checkpoint` rows for the turn with a non-empty `relative_path`, including restored and invalidated rows, because the agent did change the file in that turn.
3. Each path appears once per turn, ordered by first checkpoint ordinal, then creation time, then path.
4. The field is additive: older clients ignore it and newer clients treat a missing field as empty.
5. The desktop folds all turns' paths, in turn order and without duplicates, into its existing changed-file list.

## Edge cases

- Turns without a checkpoint manifest, and checkpoint rows with a null or empty `relative_path`.
- The same file written several times in one turn, or in several turns.
- Sessions created before this change: existing rows already carry `relative_path`, so no backfill is needed.

## Acceptance criteria

- Rust data and SDK tests assert `changedPaths` in the snapshot with ordering and deduplication.
- The C# SDK deserializes `changedPaths` and tolerates its absence.
- The desktop projection restores changed paths from a snapshot; existing live behavior is unchanged.
- Contract documents describe the field.

## Open questions

- None blocking. The desktop's session-wide accumulation with "in this turn" wording is recorded as a follow-up.
