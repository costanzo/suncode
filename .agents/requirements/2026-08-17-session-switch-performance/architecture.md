# Architecture

## Current state

The runtime returns the complete Phase 1 session snapshot through the existing C ABI. The desktop projects every message and retained event directly into observable collections on the UI thread. `ItemsControl` realizes every conversation row and each assistant row synchronously transforms Markdown.

## Proposed design

- Build an immutable desktop snapshot projection on a worker thread.
- Atomically replace the conversation message source after projection so Avalonia rebinding discards realized rows from the previous session.
- Replace supporting replay-owned observable collection contents through a bulk collection primitive that raises one reset notification.
- Keep live event updates incremental.
- Render messages in a virtualized `ListBox` and scroll the selected session to its last realized item once after a reset.
- Continue using the existing indeterminate loading indicator; removing UI-thread replay work keeps its animation responsive.

## Boundaries and dependencies

This is an Avalonia presentation change. The Rust runtime remains authoritative and its snapshot, event, subscription, and persistence contracts are unchanged.

## Data and control flow

1. The selected session version is captured.
2. The SDK snapshot is fetched off the UI thread.
3. A desktop projection is built off the UI thread.
4. The current selection/version is checked.
5. The message source, supporting collections, and scalar state are committed once on the UI thread.
6. Usage, checkpoints, optional traces, and the live subscription are loaded under the same version guard.
7. The conversation scrolls once after the message source replacement.

## Security and failure handling

No new data source or authority path is introduced. Snapshot failures retain the existing retryable error state. Stale projections are discarded without changing UI state.

## Compatibility and migration

No persistent data, contract, or configuration migration is required.

## Risks and rollback

Variable-height Markdown rows may expose virtualization measurement issues. Reusing one message collection across sessions may also retain realized rows after a reset, so each session snapshot receives a new message source. The change can roll back to the existing items control without affecting runtime or stored data.

## Open questions

- None.
