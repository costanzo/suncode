# Requirement

## Background

The typed Rust stream removes FFI concerns and isolates subscriber queues by session, but clients still establish state in two calls: read `session_snapshot`, then subscribe to live events. An event committed between those calls is absent from the snapshot and was published before the subscription existed. Reversing the calls prevents loss but may deliver an event already represented by the later snapshot.

## Goals

- Add one Rust SDK operation that atomically returns a normalized session snapshot and a live typed event stream.
- Serialize session projection-plus-publish with watch establishment through one in-memory session gate.
- Guarantee that event projection and publication cannot fall into a gap between the returned snapshot and stream.
- Apply the same gate to live-only events so they are queued after watch establishment rather than delivered during snapshot creation.
- Keep event streams bounded and retain the existing lag/resync behavior.

## Non-goals

- Persisting an event cursor, revision, or replay journal.
- Migrating the C ABI, C# SDK, or Avalonia session-loading flow in this delivery.
- Converting the complete Rust SDK to async-first operation.
- Supporting cross-process subscriptions or attach.

## Requirements

- `SessionEventHub` owns a per-session synchronization gate independent from subscriber queue locks.
- Durable event projection occurs while holding the same session gate used by atomic watch establishment.
- The watch operation registers its bounded stream before reading the snapshot while holding the session gate.
- Live-only publication also respects the session gate.
- The Rust SDK exposes `SessionWatch { snapshot, events }` through `watch_session(session_id)`.
- Existing `session_snapshot` and `subscribe_session_events` methods remain available with their current individual semantics.
- Snapshot failure removes the newly registered subscriber before returning the error.
- Clients continue to apply events idempotently because some normalized lifecycle writes, such as turn admission, may intentionally precede their corresponding notification projection.

## Edge cases

- A durable event begins immediately before watch establishment.
- A durable event begins immediately after watch establishment.
- A live-only delta is emitted while snapshot creation holds the session gate.
- Snapshot loading fails after the subscription was registered.
- Two hosts inside one process watch the same session concurrently.
- A session produces enough events after watch establishment to lag the returned stream.

## Acceptance criteria

- Focused tests prove there is no gap or duplicate for the synchronized event-projection boundary.
- Existing event isolation, lag, close, Rust SDK, C ABI, and desktop tests continue to pass.
- No persistence schema or C ABI version changes occur.
- Contracts clearly distinguish atomic `watch_session` from the compatibility snapshot and subscribe methods.

## Open questions

- Resolved by `../2026-09-19-native-dormant-session-watch/`: native bindings use a dormant handle and explicit callback start after snapshot application.
