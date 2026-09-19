# Changes

## Source

- Added `agent/crates/core/src/agent/event_hub.rs` for bounded session-scoped fan-out, lag signaling, and subscription lifecycle.
- Preserved typed `AgentEvent` and non-exhaustive `EventPayload` values through core and the Rust SDK.
- Added typed checkpoint restore payloads so facade-originated live events use the same event catalog.
- Replaced the Rust callback subscription with `SessionEventStream` and its close control.
- Moved callback threading, JSON serialization, C strings, and raw pointers into `sdks/c`.

## Contracts and generated artifacts

- Preserved C ABI version 9 and the current JSON event envelope.
- Updated the embedded SDK contract and SDK package documentation.

## Configuration and persistence

- No configuration or persistence schema changes.

## Tests

- Added session isolation, unrelated-session pressure, lag, blocking-close, Rust typed-stream, and C envelope compatibility coverage.
- Passed the agent workspace, Rust SDK, C binding, and desktop test suites.

## Documentation

- Added this delivery package.
- Updated architecture, durable feature/specification records, and the accepted decision index.
