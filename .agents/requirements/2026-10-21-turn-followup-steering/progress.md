# Progress

- Status: Implementation complete; visual QA pending
- Last updated: 2026-10-21

## Completed

- Reviewed current agent loop, SDK boundary, and Avalonia submission path.
- Confirmed the design-system proposal and user-approved interaction semantics.
- Implemented in-memory follow-up queueing, next-request steering, queue row mutations, ABI 25 and managed bindings.
- Implemented the Avalonia queue list, row actions, image previews, collapse, and long-message disclosure.
- Added focused tests for independent follow-up turns, steering within the active turn, FIFO follow-ups, and row payload/order operations.

## Completed

- Queue behavior, SDK/ABI bindings, desktop interaction, feature documentation, and SDK contract updates are complete.

## Blocked

- None.

## Log

### 2026-10-21

- Started production implementation for after-turn follow-ups and next-request steering.
- Added the queue surface to Avalonia and integrated additive queue operations through the Rust SDK, C ABI 25, and managed SDK.
- Focused queue tests, C ABI check, Avalonia build, Rust formatting for touched Rust files, and `git diff --check` pass. The broader desktop test suite has 4 failures in pre-existing Browser Preview layout and turn-duration assertions.
- Full `suncode-core` run also has an unrelated existing policy expectation failure; all three focused queue tests pass.
- Running-desktop visual verification remains outstanding. The Avalonia client builds cleanly, but it has not been launched for theme or narrow-window inspection.
