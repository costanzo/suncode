# Architecture

## Current state

`SessionEventHub` registers bounded session-scoped queues, while event producers project durable state and then publish a typed event. Snapshot reads and subscriber registration are separately callable. The absence of a shared session gate leaves a race between the durable projection and live publication boundary.

## Proposed design

Extend `SessionEventHub` with one per-session synchronization gate. Event producers call a synchronized projection helper that acquires the gate, applies the event projection, and publishes the typed event before releasing it. Live-only events use the same synchronized publication path.

The hub exposes `subscribe_with_snapshot`, which acquires the gate, registers a new bounded subscriber, executes the supplied snapshot closure, and returns both values. Registering before reading the snapshot is safe because no synchronized producer can apply its event projection or publish until the gate is released. An earlier event projection completes before the watch lock is acquired and is visible in the snapshot. A later event projection runs after the stream exists and its event is delivered through the stream.

Some normalized lifecycle mutations intentionally happen before their corresponding notification projection, including turn admission. A watch may therefore already observe such state and later receive an idempotent notification for it. The gate closes the snapshot/subscription loss window; it does not redefine every persistence operation as an event-sourced transaction.

The Rust SDK wraps the result as `SessionWatch { snapshot, events }`.

## Boundaries and dependencies

- Core owns the session gate and ordering between projection and live publication.
- The data package remains the normalized persistence owner and does not know about subscribers.
- The Rust SDK supplies the snapshot closure and public `SessionWatch` DTO.
- C/C#/Avalonia remain on their existing compatibility calls in this delivery.

## Data and control flow

Earlier producer:

1. Acquire session gate.
2. Apply durable projection.
3. Publish typed event to currently registered streams.
4. Release gate.
5. Watch registers and reads a snapshot containing the event projection.

Later producer:

1. Watch acquires session gate.
2. Watch registers stream.
3. Watch reads snapshot.
4. Watch releases gate.
5. Producer applies its event projection and publishes into the returned stream.

## Security and failure handling

The gate changes ordering only; it does not grant authority or change persistence. Snapshot failure drops the provisional stream and unregisters it. The gate is held only for a bounded local SQLite snapshot and registration; no provider, process, network, callback, or user interaction occurs while it is held.

## Compatibility and migration

The change is additive to the Rust API. Existing Rust, C, C#, and Avalonia methods remain available. No schema, ABI, event-envelope, or dotted event-name change is required.

## Risks and rollback

Risks are deadlock, lock-order inversion, excessive gate hold time, and a producer bypassing the synchronized path. Tests cover concurrent before/after publication and snapshot failure cleanup. Rollback removes `watch_session` and returns producers to direct publish without changing persisted state.

## Open questions

- Resolved by `../2026-09-19-native-dormant-session-watch/`: native bindings create a dormant callback subscription and require explicit activation after the host applies the snapshot.
