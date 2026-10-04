# Rust SDK Facade

**Status:** Implemented and focused-tested

`sdks/rust` is the typed, reusable Rust facade over the embedded agent harness. Rust hosts (the CLI) use it directly; the Avalonia desktop reaches it through the C ABI in `sdks/c` and the managed layer in `sdks/csharp`. The facade composes agent state and DTOs only. It opens no second database and has no independent provider, tool, or policy behavior. Agent loop, persistence, and operations behavior are described in [`agent-phase-1`](../agent-phase-1/README.md), [`persistence-phase-1`](../persistence-phase-1/README.md), and [`rust-core-phase-1`](../rust-core-phase-1/README.md). The public contract is [`contracts/agent-sdk/README.md`](../../../contracts/agent-sdk/README.md).

## Ownership

```text
Avalonia -> sdks/csharp -> sdks/c (C ABI) -> sdks/rust -> agent core -> data / database / llm / tools / config
CLI ------------------------------------------> sdks/rust
```

- `agent/crates/core` owns agent execution, the typed event catalog, session event fan-out, and shutdown ordering. It does not build a `cdylib`, a `staticlib`, or SDK modules. Agent logic sits in focused modules under `core/src/agent/`. Bootstrap configuration is the `suncode-config` crate at `agent/crates/config/`.
- `agent/crates/data` keeps `Store` as the single public persistence entry type. `store.rs` holds connection lifecycle, schema initialization, transactions, locking, and health. Each `operations/<table>.rs` module implements that table's methods and row decoding in its own `impl Store` block. Writes that span tables live in the `projection` (event-driven) and `recovery` (startup reconciliation) orchestration modules. The data crate knows nothing about subscribers.
- `sdks/rust` owns `AsyncAgentSdk`, the blocking adapter, the host-facing DTOs (`types.rs`), the startup options, `SessionEventStream`, and `SessionWatch`. It has no C callbacks, raw pointers, JSON envelope serialization, or worker threads.
- `sdks/c` owns `extern "C"` exports, panic containment, callback worker threads, C strings, the established `{session_id, occurred_at, event_type, payload}` JSON envelope, and the translation of lag into `resync.required`. It never touches persistence or provider internals directly.
- `sdks/csharp` owns P/Invoke, native library build integration, exact ABI version validation, typed DTOs, and handle lifetime for Avalonia.

## Async-first and blocking surfaces

- `AsyncAgentSdk` runs on the caller's Tokio runtime and creates no runtime of its own. Methods that wait on agent, turn, approval, question, Browser, MCP, LSP, or network work are native `async fn`. Bounded local SQLite and DTO operations, including `watch_session`, stay synchronous and may briefly block an executor thread.
- `blocking::AgentSdk` owns one Tokio runtime and wraps one `AsyncAgentSdk`. Through `Deref`/`DerefMut` it reuses the synchronous methods unchanged and defines blocking counterparts only for the async ones. The crate root re-exports it as `AgentSdk` for compatibility. Calling it from inside an async runtime is unsupported.
- Both surfaces share one implementation of validation, persistence, policy, DTOs, and events. They also enforce the same one-host-per-data-directory lock.

## Startup options and host capability ceilings

`open_default`, `open_with_options`, `open_default_with_providers`, and `open_with_options_and_providers` exist on both surfaces. The compatibility methods pass `SdkOpenOptions::default()`. `SdkOpenOptions { host_capabilities: SdkHostCapabilities { browser_use, computer_use } }` defaults to both enabled, which keeps desktop and C startup fully capable.

The ceiling is mapped to the immutable core `AgentHostCapabilities` and fixed for the SDK's lifetime. When a capability is disabled:

- Core advertises no model tools for it, starts no Browser worker, initializes no Enigo backend, and makes no OS permission requests.
- Runtime information may still report the persisted desired setting. It reports the capability as host-unavailable, with no active backend.
- Named and generic management, permission, and settings calls are rejected with `browser_host_unavailable` or `computer_host_unavailable` before anything is written. The shared persisted preference never changes.

A ceiling only removes capabilities. It grants no authority, and policy still evaluates every call the ceiling allows. The CLI turns off both Browser Use and Computer Use.

## Typed events and session streams

- Core events are `EventPayload` enum variants with named payload structs. Each variant fixes its stable dotted `EventType`, and `emit` accepts no arbitrary name/JSON pair. `AgentEvent` is `#[non_exhaustive]` and is re-exported under `suncode_sdk::events`. Rust hosts pattern-match typed events without parsing JSON.
- Core's `SessionEventHub` gives each subscriber its own bounded queue (256 events in the facade) registered to one session ID. Events are delivered only to subscribers of that session, so unrelated sessions cannot fill each other's queues. A full queue marks its subscriber as lagged instead of buffering without limit.
- `SessionEventStream` implements `futures_core::Stream<Item = Result<Arc<AgentEvent>, SubscriptionError>>` and `FusedStream`. A normal close ends the stream (`None`). `Lagged { missed }` is reported once, then the stream ends, and the host must establish a new watch. The direct `recv`, `blocking_recv`, and `try_recv` methods remain; `try_recv` returns `Empty` without ending the stream. A cloneable `SessionEventStreamControl` can close the stream from elsewhere and wakes any pending receiver.

## Atomic session watch

`watch_session(session_id)` returns `SessionWatch { snapshot, events }`. The hub keeps one in-memory gate per session. Event producers hold that gate while they apply a durable projection and publish the event, and live-only events are published under the same gate. A watch takes the gate, registers its stream, reads the normalized snapshot, then releases the gate. As a result, every event lands either in the snapshot or in the stream, never between them. If the snapshot read fails, the provisional subscriber is unregistered. The gate is held only for local SQLite reads and registration, never during provider, process, network, or callback work.

Some lifecycle writes, such as turn admission, happen before their notification is projected, so hosts must apply events idempotently. The standalone `session_snapshot` and `subscribe_session_events` remain available, but calling them in sequence is not atomic.

## Native dormant watch

The C ABI exposes `watch_session`, which returns snapshot JSON plus an opaque dormant subscription handle, and the single-use `subscription_start`. While a handle is dormant, events queue in its bounded typed stream and no callback runs. A second start, or a start after close, fails explicitly. Close works before or after start, and a callback may dispose its own subscription without a self-join. If the queue overflows while dormant, the stream reports lag and delivers `resync.required` once started.

C# exposes a disposable `SessionWatch` with a typed `Snapshot` and `Start`. When Avalonia loads a primary session, it applies the snapshot and auxiliary state, confirms the load is still the latest selection, installs the watch, and only then starts callbacks. Stale, failed, or cancelled loads dispose their dormant handle. Resync follows the same path. The immediate-subscription native function remains for compatibility. Child-session inspection is snapshot-only.

## Explicit shutdown

`AsyncAgentSdk::shutdown(self).await` and `blocking::AgentSdk::shutdown(self)` consume the handle and both run one idempotent core shutdown, in this order:

1. Reject new turn submissions and approval/question continuations with `agent_shutting_down`.
2. Cancel every active turn token and clear queued follow-up input.
3. Run the Computer Use emergency stop to release held input.
4. Drain Browser, MCP, and LSP managers concurrently. Late starts are retired rather than reinstalled.
5. Wait up to five seconds for active turns. A turn that is still running produces an `agent_unavailable` error.
6. Close every session stream, waking pending receivers.
7. Release SQLite ownership and the data-directory lock, even if an earlier step failed.

Cleanup continues across manager categories before any error is returned, and the same data directory can be reopened immediately afterward. The blocking adapter keeps its runtime alive until cleanup finishes. Releasing the final native handle runs blocking shutdown through the existing `void` close symbol and logs any failure. Dropping a facade without shutdown remains a non-graceful fallback.

## Not implemented

No `spawn_blocking` variants exist for heavy synchronous projections. There is no host-visible shutdown report or error-returning native close, no atomic watch for child-session inspection, and no Python or TypeScript bindings (`sdks/python` and `sdks/typescript` are placeholders).
