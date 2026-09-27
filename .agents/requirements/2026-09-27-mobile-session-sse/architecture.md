# Architecture

## Current state

`RemoteMobileRepository` owned one process-wide WebSocket and synchronized projections at reconnect. Events lacked a replay cursor in the Mobile DTO.

## Proposed design

The app lifecycle and Session detail surface select one transport mode. Foreground without an open Session polls `/v1/sync` every 15 seconds. Foreground with an open Session stops polling and opens `/v1/sessions/{sessionId}/events` as SSE. Background closes both modes. A new stream sends a full `session.snapshot` as its first frame, with atomic snapshot/subscription semantics.

Each SSE application frame carries a stable event ID and per-Session monotonic sequence/revision. Mobile stores event IDs and sequences with its offline projection, sends `Last-Event-ID` on resume, ignores duplicate/older events, and resets the cursor after `410`.

## Boundaries and dependencies

Ktor owns authenticated HTTP/SSE transport. The Repository owns transport selection, projection reduction, retry, cursor persistence, and debounced cache writes. Compose owns Session navigation and reports app foreground transitions. The Remote Server owns event replay, heartbeat, and atomic snapshot/subscription behavior. Desktop's private outbound WebSocket remains unchanged.

## Data and control flow

1. Opening Session detail starts the Session SSE stream.
2. A fresh stream applies the complete snapshot before later events.
3. Events apply in sequence; unsupported event types still advance the cursor.
4. SSE deltas update observable memory state immediately; cache writes are debounced.
5. Closing detail switches to `/v1/sync` polling if foreground.

## Security and failure handling

SSE uses the existing bearer token over HTTPS. Unauthorized streams refresh credentials. Expired replay cursors reset from a fresh snapshot. Retries are bounded and jittered. No background network delivery is promised.

## Compatibility and migration

The Mobile HTTP contract remains under `/v1`, but the event endpoint and `POST /sessions` response evolve additively in the written contract. A deployed Remote Server must implement the new endpoint and response before this client can be used against it.

## Risks and rollback

The repository contains the Mobile client and protocol contract, not the Remote Server implementation. Rollback requires restoring the previous Mobile WebSocket client and contract only if server migration has not started; once the server adopts SSE, both ends must be versioned together.

## Open questions

- None for this client-side delivery.
