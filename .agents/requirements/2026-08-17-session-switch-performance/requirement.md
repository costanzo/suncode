# Requirement

## Background

Switching sessions immediately after opening a project can stall the Avalonia UI while a complete session snapshot is projected and rendered. The current conversation uses a non-virtualized items control, synchronously renders Markdown for every assistant message, raises one collection notification per replayed item, and schedules one scroll operation per message.

## Goals

- Keep the project window responsive while switching sessions.
- Make only the latest session selection visible.
- Show an animated loading transition while snapshot data is unavailable.
- Batch snapshot projection changes and scroll once after replacement.
- Realize conversation rows only when needed for the viewport.

## Non-goals

- Changing the runtime SDK snapshot contract or SQLite schema.
- Paginating or truncating durable history in this delivery.
- Changing live event ordering or message content.

## Requirements

- Snapshot-to-UI projection runs away from the Avalonia UI thread.
- Snapshot messages are committed through one atomic message-source replacement; activity and touched paths use bounded collection notifications.
- Conversation rendering uses an Avalonia virtualizing control.
- Markdown controls are created only for realized conversation rows.
- The loading animation remains responsive during a switch.
- Composer editing remains available while loading, but submission remains disabled.

## Edge cases

- Rapid A-B-A selection changes.
- A stale projection completing after a newer selection.
- Empty sessions and sessions containing only user messages.
- Long Markdown messages with code blocks.
- Snapshot failure and retry.
- A live event arriving after snapshot replay and subscription creation.

## Acceptance criteria

- A stale snapshot cannot update collections or subscriptions.
- One snapshot replacement produces one message-source property change and one requested scroll.
- Opening a project and immediately selecting another session keeps the loading animation responsive.
- Debug and Release desktop builds, formatting, runtime tests, and design checks pass.

## Open questions

- None.
