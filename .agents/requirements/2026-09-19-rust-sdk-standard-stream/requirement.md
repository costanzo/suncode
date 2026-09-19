# Requirement

## Background

The Rust SDK now exposes a typed `SessionEventStream` with async, blocking, and nonblocking receive methods, but it is still a custom receiver rather than a standard Rust `Stream`. Async hosts cannot use `StreamExt`, ordinary stream combinators, or generic stream-consuming code without writing an adapter.

## Goals

- Implement the standard `futures_core::Stream` contract on `SessionEventStream`.
- Make terminal behavior explicit and compatible with `FusedStream`.
- Preserve direct `recv`, `blocking_recv`, and `try_recv` methods for existing hosts and the C adapter.
- Preserve typed events, bounded queues, session isolation, atomic watch, and resync-on-lag behavior.

## Non-goals

- Adding a CLI, TUI, or other client.
- Persisting or replaying events.
- Changing the C ABI, C# API, or Avalonia callback contract.
- Replacing Tokio channels or introducing a second event implementation.

## Requirements

- `SessionEventStream` implements `futures_core::Stream<Item = Result<Arc<AgentEvent>, SubscriptionError>>`.
- Normal closure produces `None` rather than a synthetic stream item.
- Lag produces one `SubscriptionError::Lagged` item and then terminates the stream.
- The stream implements `FusedStream` and remains terminated after returning `None`.
- Existing receive methods continue to return their established typed results.
- Core exposes only the minimal poll primitive required by the facade.

## Edge cases

- Close control wakes a pending `next()` call.
- Explicit stream close becomes terminal.
- Lag is reported once even when buffered events remain.
- Polling again after closure or lag returns `None`.
- `try_recv` still reports `Empty` without terminating.

## Acceptance criteria

- Async Rust code can use `StreamExt::next` directly on a session watch stream.
- Standard stream closure and lag behavior have focused tests.
- Existing Rust, C, C#, and Avalonia subscription behavior remains compatible.
- Formatting, focused Clippy, and `git diff --check` pass.

## Open questions

- A future major Rust API may replace the compatibility receive methods with separate async and blocking stream types, but this delivery keeps source compatibility.
