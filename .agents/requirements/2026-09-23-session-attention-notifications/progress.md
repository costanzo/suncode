# Progress

- Status: Implemented; installed-platform verification pending
- Last updated: 2026-09-23

## Completed

- Reviewed the current embedded SDK, session-scoped event streams, selected-session Avalonia watch, multi-window application coordination, packaging inputs, and deferred IPC boundary.
- Confirmed the first-delivery notification events and foreground suppression policy with the user.
- Approved the Rust attention-stream, Avalonia notification coordinator, and desktop-only activation IPC architecture.
- Implemented the Rust attention hub, normalized candidate query, Rust/C/C# bindings, and ABI 14 contract.
- Implemented the application-scoped coordinator, bounded local ledger, foreground suppression, lag reconciliation, ID-based primary/child navigation, and desktop-only activation IPC.
- Implemented macOS User Notifications, Windows toast protocol activation, and Linux freedesktop D-Bus delivery with action fallback.
- Added focused Rust, C ABI, C# DTO, IPC framing, ledger, foreground, and notification-content tests.

## In progress

- Installed-application display and click-activation smoke tests on macOS, Windows, GNOME, and KDE.

## Blocked

- None. Remaining installed-platform checks require their target release environments.

## Log

### 2026-09-23

- Requirement initialized and approved.
- Confirmed that successful completion, failure, approval required, and question required notify only while SunCode is not foreground.
- Confirmed that foreground-suppressed events are not replayed later.
- Confirmed that child completion/failure do not notify, while child approval does notify and routes through the parent session.
