# Architecture

## Current state

The C subscription function creates and starts a callback worker immediately. The managed desktop loads a snapshot and auxiliary session data first, then subscribes. This cannot consume Rust's atomic `SessionWatch` guarantee.

## Proposed design

Add a C `watch_session` function that invokes Rust `AgentSdk::watch_session`, serializes the returned snapshot into an output string, and stores the typed stream, callback, user data, and close control in an opaque dormant subscription handle. No worker thread exists yet.

Add `subscription_start(handle, error_out)`. Start atomically takes the stored stream and creates the C-binding-owned callback worker. The existing compatibility `subscribe_session` continues creating a stream and immediately starting it through the same internal handle implementation.

C# wraps the dormant native handle in `SessionWatch`, deserializes its typed `SessionSnapshot`, and exposes `Start` plus `Dispose`. Avalonia owns the activation point: after snapshot projection and auxiliary state load succeed and the load is still current, it assigns the watch to `_subscription`, marks the session loaded, then starts callback delivery.

## Boundaries and dependencies

- Rust SDK continues owning atomic snapshot/stream establishment.
- C owns native thread creation, raw callback state, snapshot string ownership, and start/close lifecycle.
- C# owns native handle lifetime and typed snapshot/event deserialization.
- Avalonia owns when its presentation state is ready for callbacks.

## Data and control flow

1. Desktop begins loading and closes the previous subscription.
2. C# calls native `watch_session` off the UI thread.
3. Rust atomically returns snapshot plus stream.
4. C returns the snapshot and dormant handle; events begin queueing.
5. Desktop projects and applies the snapshot, then loads auxiliary state.
6. Desktop verifies the load is still current and assigns the watch.
7. Desktop calls `Start`; callbacks begin and marshal to the UI thread.

## Security and failure handling

The flow changes ordering only. Callback payloads remain bounded and redacted. Start failure disposes the installed watch and surfaces the existing session-load error. Stale or failed loads dispose dormant handles. Callback self-disposal must not self-join.

## Compatibility and migration

The C ABI advances because new symbols are required and C# validates an exact ABI number. Existing native subscription symbols remain. The event JSON envelope is unchanged. No persistence or UI design-system change is required.

## Risks and rollback

Risks are handle leaks, callback activation before assignment, double start, self-join, and queued-event lag. Focused native and managed tests cover lifecycle behavior; existing desktop stale-load guards remain authoritative. Rollback restores separate snapshot and subscribe calls while leaving Rust `watch_session` available.

## Open questions

- Future bindings may prefer an async stream pump instead of an explicit dormant callback handle.
