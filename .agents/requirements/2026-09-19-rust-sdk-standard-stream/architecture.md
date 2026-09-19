# Architecture

## Current state

`SessionEventStream` wraps the core `AgentEventSubscription` and exposes `recv`, `blocking_recv`, and `try_recv`. The core subscription owns a Tokio MPSC receiver, but neither layer exposes polling for the standard `Stream` trait.

## Proposed design

Add a minimal `poll_recv` method to the core subscription using the existing registration checks and Tokio receiver. Implement `futures_core::Stream` and `FusedStream` only in the public Rust SDK facade.

The stream item remains a typed `Result<Arc<AgentEvent>, SubscriptionError>`. Ordinary closure maps to the standard end-of-stream `None`. Lag is a recoverable host signal but invalidates this live stream, so it appears once as `Some(Err(Lagged))`; subsequent polls return `None` and the host establishes a new atomic watch.

## Boundaries and dependencies

- Core continues owning bounded fan-out, registration, lag detection, and wakeup behavior.
- The Rust SDK owns public standard-stream semantics.
- The C binding continues using `blocking_recv` and requires no futures dependency or ABI change.
- `futures-core` is the only new runtime dependency; test code uses `futures-util::StreamExt`.

## Data and control flow

1. A host calls `watch_session` and receives a snapshot plus `SessionEventStream`.
2. Generic async code polls the stream through `StreamExt::next`.
3. Core polls the existing Tokio receiver and performs the same pre/post registration checks as `recv`.
4. Events yield typed `Ok` items.
5. Close yields `None`; lag yields one typed error and then `None`.

## Security and failure handling

No authority, persistence, or provider boundary changes. Queue bounds remain unchanged. Lag continues to fail closed and requires a normalized snapshot resync.

## Compatibility and migration

The change is additive for Rust hosts. Direct receive methods remain available. C ABI 10, C# DTOs, JSON envelopes, and Avalonia behavior remain unchanged.

## Risks and rollback

The principal risks are inconsistent terminal behavior between polling and direct receive methods, missed wakeups on close, and repeatedly emitting lag. Focused standard-stream tests cover closure, fused behavior, and one-shot lag. Rollback removes the trait implementations and poll primitive without data changes.

## Open questions

- Whether a future major release should make the async facade and standard stream the unqualified defaults while moving all blocking compatibility into explicitly named modules.
