# Progress

- Status: Complete
- Last updated: 2026-08-19

## Completed

- Diagnosed the session switch path and measured local snapshot volume.
- Confirmed that client rendering and notification amplification dominate SQLite read time.
- Agreed to keep the runtime SDK contract unchanged for this delivery.
- Moved snapshot projection off the UI thread, atomically replaced the conversation message source, and committed supporting collections through one reset each.
- Replaced the conversation items control with an explicit virtualizing stack panel.
- Coalesced scroll-to-end requests and added a version-aware delayed loading animation.
- Added and passed focused collection, projection, and message-source rebinding tests.

## In progress

- None.

## Blocked

- None.

## Log

### 2026-08-17

- Requirement initialized.
- Implementation and focused verification completed.

### 2026-08-19

- Diagnosed a remaining startup-only display issue caused by virtualized rows surviving collection resets.
- Replaced the complete conversation message source for each selected session and moved scroll signaling away from a specific collection instance.
- Added a regression test for source replacement and the binding property notification.
