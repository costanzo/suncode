# Progress

- Status: Complete
- Last updated: 2026-09-19

## Completed

- Reviewed the current core, Rust facade, C binding, C# binding, and Avalonia subscription path.
- Confirmed that the existing C ABI and JSON envelope can remain compatible.
- Added non-exhaustive typed `AgentEvent` values and typed checkpoint restore events.
- Added a bounded session-scoped event hub with explicit lag and close outcomes.
- Added the Rust-native `SessionEventStream` with async, blocking, and nonblocking receive methods.
- Moved the callback worker, raw pointer, JSON envelope, and resync compatibility behavior into `sdks/c`.
- Preserved C ABI version 9 and the existing C#/Avalonia envelope.
- Updated the SDK contract, durable feature/specification records, architecture, and decision index.
- Passed focused and broad Rust and Avalonia verification.

## In progress

- None.

## Blocked

- None.

## Log

### 2026-09-19

- Requirement initialized.
- Scoped this delivery to the event boundary cleanup; async-first facade and atomic snapshot/watch remain follow-up work.
- Implementation and verification completed.
