# Requirement

## Background

The Mobile client used a process-wide WebSocket for all Session events. Mobile transport is moving to HTTP Server-Sent Events, scoped to the Session currently open in the foreground, with projection polling while no Session is open.

## Goals

- Receive ordered real-time events for the open Session through SSE.
- Poll the existing synchronization projection while the app is foregrounded outside Session detail.
- Resume event delivery using a persisted per-Session event cursor.

## Non-goals

- Replacing the private Desktop-to-Remote Server WebSocket.
- Implementing the Remote Server runtime, which is outside this repository.
- Background event delivery or push notifications.

## Requirements

- A new SSE connection starts with an atomic full Session snapshot and continues with ordered events.
- SSE events carry stable IDs, per-Session sequence values, and Session revisions.
- Reconnect sends `Last-Event-ID`; an expired cursor causes a snapshot refresh and cursor reset.
- Foreground Session list/other surfaces poll `/v1/sync`; Session detail owns one SSE stream.
- Backgrounding closes active transport. Returning to foreground resumes the appropriate transport.
- Streamed assistant deltas update memory immediately and persist to offline cache with debounce.
- Session creation returns the Session ID so Mobile can immediately open it.

## Edge cases

- Duplicate or older events are ignored.
- Unknown event types advance the cursor without forcing per-event Session fetches.
- A `410 cursor_expired` clears the cursor, refreshes the Session, and reconnects for a new snapshot.
- SSE authorization is refreshed on `401`; other failures use bounded exponential retry with jitter.

## Acceptance criteria

- Mobile no longer depends on WebSocket for event delivery.
- Session detail entry/exit and app lifecycle control SSE and list polling.
- Protocol documents and hand-written DTOs agree on event IDs, sequence, snapshot, and creation response.
- Android tests/build and iOS Simulator Kotlin compilation pass.

## Open questions

- Remote Server implementation and deployment must adopt this contract before the Mobile SSE client can connect successfully.
