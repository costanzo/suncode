# Requirement

## Background

The typed Rust SDK currently exposes session subscriptions through a C-shaped callback, raw pointer, and JSON envelope. Core creates typed event payloads but converts them to `serde_json::Value` before broadcasting them through one global channel. The C binding therefore does not own its callback adaptation, Rust hosts cannot consume a typed pull-based stream, and unrelated sessions share lag pressure.

## Goals

- Preserve typed agent events from core through the Rust SDK.
- Expose a bounded pull-based session event stream for synchronous and asynchronous Rust hosts.
- Scope subscription queues per session so unrelated session traffic cannot cause lag.
- Move callback threads, JSON serialization, C strings, and raw pointers into `sdks/c`.
- Preserve the existing C ABI and Avalonia event envelope.
- Keep lag fail-closed through an explicit resync-required outcome.

## Non-goals

- Replacing the SDK with a service or IPC protocol.
- Persisting an event journal or replaying transient deltas.
- Completing the async-first SDK facade in this delivery.
- Adding the atomic snapshot-plus-stream `watch_session` API in this delivery.
- Changing Avalonia event rendering or interaction behavior.

## Requirements

- Core publishes `AgentEvent` values containing a typed `EventPayload`.
- A session event hub owns bounded subscriber queues and routes only matching-session events.
- A slow subscriber becomes stale and receives a lag error rather than unbounded buffering.
- Dropping or explicitly closing a subscription wakes blocking consumers and releases hub state.
- The Rust SDK public API contains no C callback alias or `c_void` subscription argument.
- `sdks/c` converts typed events to the established `{session_id, occurred_at, event_type, payload}` envelope and emits `resync.required` when the Rust stream reports lag.
- Existing native method names and ABI version remain unchanged.

## Edge cases

- One session produces a high event volume while another session has a slow subscriber.
- A subscriber is closed while blocked waiting for an event.
- A subscriber queue fills while tool output or assistant deltas are streaming.
- A callback consumer closes its subscription in response to `resync.required`.
- The agent shuts down before a subscription handle is closed.

## Acceptance criteria

- Rust code can receive and pattern-match typed events without JSON parsing.
- C# continues receiving the current JSON envelope without ABI changes.
- Unrelated session traffic does not enter or fill another session's queue.
- Focused core, Rust SDK, C ABI, and Avalonia tests pass.
- `git diff --check` passes.

## Open questions

- The later async-first facade and atomic `watch_session` API will be designed after this boundary cleanup is verified.
