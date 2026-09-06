# SunCode Rust SDK

This crate is the typed Rust SDK facade over the `suncode-agent` harness. It owns the host-facing Rust methods, DTOs, lifecycle, and subscription API. The Avalonia-facing C ABI is implemented separately by [`../c`](../c).

It does not open a second database or implement provider/tool behavior independently; those remain owned by the Rust agent crates.

## Source layout

- `src/lib.rs` defines the crate's public entry point and re-exports the facade and DTOs.
- `src/types.rs` owns host-facing DTOs, result and callback aliases, and the ABI version constant.
- `src/facade/` owns `AgentSdk`, lifecycle and configuration helpers, and methods grouped by capability.
- `src/facade/subscriptions.rs` owns the live event subscription lifecycle.
- `src/facade/tests.rs` keeps facade behavior tests beside the implementation; C ABI tests remain in `sdks/c`.
