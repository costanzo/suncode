# Architecture

## Current state

Core serializes typed `EventPayload` values into a data-owned `SessionEvent` and sends them through one Tokio broadcast channel. The Rust facade creates one operating-system thread per subscription, filters the shared channel by session, serializes events to JSON, and invokes a C callback. The `after` argument is accepted but has no replay behavior.

## Proposed design

Core owns a cloneable `SessionEventHub`. Each subscription receives a dedicated bounded Tokio MPSC queue registered under one session ID. Publishing clones an `Arc<AgentEvent>` only to subscribers for that session. A full queue marks that subscriber stale; subsequent receive attempts report lag and require a snapshot resync.

`AgentEvent` retains the typed `EventPayload` and stable event metadata. `sdks/rust` wraps the core receiver as `SessionEventStream` with async, blocking, and nonblocking receive methods plus a separately cloneable close control.

`sdks/c` owns the callback worker thread. It blocks on the typed stream, serializes successful events to the existing JSON envelope, translates lag into `resync.required`, and stops on close or channel termination.

## Boundaries and dependencies

- Agent core owns event production and session-scoped fan-out.
- The data package continues applying normalized projections but does not define the new typed live event.
- The Rust SDK owns the host-facing stream and error types.
- The C binding owns callbacks, raw pointers, C strings, JSON envelope adaptation, and callback-thread lifetime.
- C# and Avalonia contracts remain unchanged in this delivery.

## Data and control flow

1. Core constructs a typed `EventPayload`.
2. Durable events are projected transactionally; live-only events skip projection.
3. Core constructs `AgentEvent` with the committed timestamp.
4. `SessionEventHub` offers the event only to subscribers registered for that session.
5. Rust hosts pull typed events from `SessionEventStream`.
6. The C callback adapter serializes the event only at the native boundary.

## Security and failure handling

Queues remain bounded. Lag invalidates the stream and requires normalized snapshot recovery. Callback arguments and event serialization failures remain bounded and logged without adding credentials or raw provider secrets. Closing a native subscription signals the stream before joining its worker thread.

## Compatibility and migration

The C ABI version, native symbols, JSON envelope, dotted event names, and C# DTOs remain unchanged. Rust source compatibility intentionally changes by replacing the callback-shaped subscription with a typed stream. No persistence schema change is required.

## Risks and rollback

The primary risks are deadlock during close, stale subscriber retention, and accidental JSON envelope changes. Focused lifecycle, lag, session-isolation, and envelope tests mitigate them. Rollback restores the previous shared broadcast channel and Rust-owned callback adapter without changing persisted data.

## Open questions

- The next delivery should decide whether `watch_session` uses an in-memory session gate or a persisted projection revision.
- The next delivery should separate the async-first facade from the blocking host wrapper.
