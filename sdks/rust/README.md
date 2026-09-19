# SunCode Rust SDK

This crate is the typed Rust SDK facade over the `suncode-agent` harness. It owns the host-facing Rust methods, DTOs, lifecycle, and subscription API. The Avalonia-facing C ABI is implemented separately by [`../c`](../c).

The `version` method returns the embedded `suncode-agent` core package version without opening agent state. Client bindings use it for About-window component version display.

It does not open a second database or implement provider/tool behavior independently; those remain owned by the Rust agent crates.

## Source layout

- `src/lib.rs` defines the crate's public entry point and re-exports the facade and DTOs.
- `src/types.rs` owns host-facing DTOs, result aliases, and the ABI version constant.
- `src/facade/` owns `AgentSdk`, lifecycle and configuration helpers, and methods grouped by capability.
- `src/facade/subscriptions.rs` owns the typed pull-based `SessionEventStream`, its close control, and lag/closed outcomes. It contains no C callbacks, raw pointers, JSON serialization, or worker threads.
- `src/facade/tests.rs` keeps facade behavior tests beside the implementation; C ABI tests remain in `sdks/c`.

The `events` module re-exports the non-exhaustive typed core event catalog. Rust hosts receive `Arc<AgentEvent>` values and can use async `recv`, blocking `blocking_recv`, or nonblocking `try_recv`. Native callback adaptation belongs to each language binding.
