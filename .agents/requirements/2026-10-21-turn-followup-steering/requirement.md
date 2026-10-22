# Requirement

## Background

The design-system specimen defines multi-message follow-ups with row-level promotion to steering, but the current agent queue injects all queued text into the active turn at loop safe points and the desktop has no corresponding queue management surface.

## Goals

- Make ordinary queued submissions run as independent turns after the active turn completes, in FIFO order.
- Let a user promote one queued item to steering for the active turn's next model request without cancelling its current request or tool execution.
- Support multiple queued messages with text and existing image attachments.
- Expose queue rows and their status through the Rust SDK and Avalonia desktop, including take-back/edit, remove with undo, and long-text disclosure.
- Preserve queue payloads across temporary steering/admission failures and make admission idempotent.

## Non-goals

- Persisting pending messages across process shutdown or sharing them across processes.
- Changing CLI behavior, adding remote queue management, or introducing a database schema migration.
- Replacing the existing explicit Stop/cancel behavior.

## Requirements

- Follow-up messages submitted during an active turn remain ordered and start new turns only after the current turn terminates.
- Steering is an explicit row action. It is delivered only before a future model request in the same turn; current provider requests and tools continue uninterrupted.
- Steering admission is single-use and idempotent. A failed promotion keeps the row and exposes retry; a turn that ends before steering is consumed leaves the item as a follow-up.
- Queue operations are session-scoped, validate message ownership and active-turn state in Rust, and never accept caller-supplied image paths or provider data.
- Text and image references move together through queueing, steering, take-back/edit, removal/undo, and follow-up admission. Model vision capability is checked when a queued turn is admitted.
- The desktop shows the queue above the composer without overlaying output. The header can collapse only the list. Long message text previews at most two lines, with a per-row disclosure when it overflows.
- Removing an item offers Undo with its original order and mode. Taking an item back restores its full payload to the draft and focuses the composer; resubmission defaults to follow-up.
- Queue state is transient and cleared on SDK shutdown. Session snapshots/reload and live queue events must reconcile the desktop view.

## Edge cases

- Multiple submissions share an active turn: preserve FIFO order and idempotency keys.
- A queued image is removed from local storage before admission: keep the row and report an admission failure rather than silently losing it.
- Model selection changes before admission: revalidate availability and image capability for each follow-up.
- Steering is requested while no active turn exists or after queue pause/stop: reject it without changing the row.
- The active turn completes before steering is consumed: downgrade the row to follow-up and do not duplicate it.
- A new draft exists when taking back a row: preserve the draft as an ordered follow-up and restore the selected row to the composer.
- Event lag, reconnect, or session switch: queue snapshot remains authoritative for current in-memory state.

## Acceptance criteria

- Agent tests prove follow-ups are not injected into the current turn and are admitted as FIFO independent turns after completion.
- Agent tests prove steering reaches the next model call while an in-flight request/tool is not cancelled.
- Agent and SDK tests cover multiple items, idempotency, image payload preservation, remove/undo, take-back, retry, turn termination, and shutdown cleanup.
- C ABI and C# contracts expose typed queue operations and queue snapshots/events.
- Avalonia implements the approved queue behavior and matches the design-system specimen across themes, narrow layouts, accessibility, and input focus.
- Focused Rust, .NET, design-system, and diff checks pass.

## Open questions

- None. Steering semantics and multi-message queue behavior were confirmed in the conversation.
