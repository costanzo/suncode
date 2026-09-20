# SunCode Rust SDK

This crate is the typed Rust SDK facade over the `suncode-agent` harness. `AsyncAgentSdk` is the runtime-free primary Rust facade: async hosts supply Tokio and await runtime-dependent operations directly. `blocking::AgentSdk` owns one Tokio runtime for synchronous hosts and remains re-exported at the crate root as `AgentSdk` for compatibility. Both surfaces share the same state, validation, DTO, persistence, policy, and event implementation. The Avalonia-facing C ABI is implemented separately by [`../c`](../c).

`SdkOpenOptions` carries an immutable `SdkHostCapabilities` ceiling. Defaults preserve Browser and Computer Use behavior for existing desktop/native hosts. A terminal host can disable either capability without mutating shared persisted settings; core then suppresses its model tools and external initialization, while host-facing management calls fail with stable host-unavailable errors.

The `version` method returns the embedded `suncode-agent` core package version without opening agent state. Client bindings use it for About-window component version display.

It does not open a second database or implement provider/tool behavior independently; those remain owned by the Rust agent crates.

## Source layout

- `src/lib.rs` defines the crate's public entry point and re-exports the facade and DTOs.
- `src/types.rs` owns host-facing DTOs, result aliases, and the ABI version constant.
- `src/facade/` owns `AsyncAgentSdk`, lifecycle and configuration helpers, and methods grouped by capability.
- `src/facade/blocking.rs` owns executor creation and blocking adaptation only; `Deref` reuses synchronous async-facade methods.
- `src/facade/subscriptions.rs` owns the typed `SessionEventStream`, its standard `Stream`/`FusedStream` behavior, direct receive methods, close control, and lag/closed outcomes. It contains no C callbacks, raw pointers, JSON serialization, or worker threads.
- `src/facade/tests.rs` keeps facade behavior tests beside the implementation; C ABI tests remain in `sdks/c`.

The `events` module re-exports the non-exhaustive typed core event catalog. Async Rust hosts can use `StreamExt::next` and other standard stream combinators over `Result<Arc<AgentEvent>, SubscriptionError>` items. Normal close is end-of-stream; lag is returned once and then the fused stream terminates so the host can establish a fresh atomic watch. Direct async `recv`, blocking `blocking_recv`, and nonblocking `try_recv` remain available. Native callback adaptation belongs to each language binding.

Rust hosts should establish session state with `AgentSdk::watch_session`. It returns `SessionWatch { snapshot, events }` atomically relative to durable event projection and live publication. The standalone snapshot and subscribe methods remain available for compatibility but do not collectively close the boundary race.

Async hosts should use `AsyncAgentSdk::watch_session`; the compatibility wording above applies equally because `watch_session` itself is a synchronous local composition method. Blocking hosts use the root `AgentSdk`. Calling the blocking wrapper from inside an async runtime is unsupported; use `AsyncAgentSdk` instead.

Hosts should close the SDK explicitly. `AsyncAgentSdk::shutdown(self).await` and `blocking::AgentSdk::shutdown(self)` consume their handle, cancel active turns, release Computer Use input, drain Browser/MCP/LSP resources, close session streams, and release the data-directory lock. The blocking adapter retains its Tokio runtime until cleanup completes. Dropping either facade remains a fallback but does not promise graceful asynchronous cleanup.
