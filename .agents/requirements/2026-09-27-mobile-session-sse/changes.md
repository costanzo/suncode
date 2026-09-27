# Changes

## Source

- Replaced Mobile WebSocket event consumption with a Session-scoped Ktor SSE stream.
- Added foreground list polling, foreground/background transport switching, Session stream lifecycle, event cursor caching, replay deduplication, cursor reset, and debounced cache persistence for streamed deltas.
- Session creation now returns its ID and opens the created Session.

## Contracts and generated artifacts

- Added SSE AsyncAPI and updated Remote Control OpenAPI and README.
- Retained Desktop-to-Remote Server WebSocket only as a private boundary note.

## Configuration and persistence

- Persisted per-Session SSE event IDs and sequences in the existing offline cache.

## Tests

- Added coverage for SSE snapshot/event cursor DTO decoding.
- Updated Mobile cache constructor test for additive cursor fields.

## Documentation

- Updated the contracts index and this delivery package.
