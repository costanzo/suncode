# Architecture

## Current state

The agent already stores approval continuations in `session_turn.recovery_snapshot_json` and resumes them through the Rust SDK. The desktop subscribes to live session events and reloads normalized state after a lag.

## Proposed design

`question` is a special non-machine-affecting tool. Core validates its prompts, executes any preceding allowed calls, stores the continuation with `pending_call` and `remaining_calls`, and emits `question.asked`. The request ID is stored in the existing single-use recovery identifier column because only one recovery gate can suspend a turn.

The answer API validates the answer count, multiplicity, option labels, and custom-answer policy before atomically recording answers in the recovery snapshot and marking it `resuming`. The continuation then emits a tool result message and proceeds with sibling calls and the model loop.

## Boundaries and dependencies

Rust core owns tool semantics and continuation state. `suncode-db` owns the recovery snapshot update. The SDK serializes `pendingQuestion`, and Avalonia owns transient selection/custom-text state. No client accesses SQLite or provider APIs.

## Security and failure handling

Questions do not bypass unknown-tool denial or argument validation. Answer and reject are single-use. A pending request remains available in the snapshot while unanswered; a resolved request is cleared after continuation completion. Restart recovery dispatches question snapshots to the question continuation path.

## Compatibility and migration

This is a new current-project contract. No retired API or database compatibility behavior is added.

## Open questions

- None.
