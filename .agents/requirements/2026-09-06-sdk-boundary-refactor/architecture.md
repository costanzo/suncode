# Architecture

## Current state

`agent/crates/core/src/sdk.rs` contains the typed `AgentSdk` facade and the exported C ABI. Avalonia builds `suncode-agent` directly.

## Proposed design

`agent/crates/core` exports harness services and domain primitives only. `sdks/rust` owns `AgentSdk` and its Rust DTO facade. `sdks/c` owns the stable `extern "C"` boundary and emits `libsuncode_agent` for Avalonia. Python and TypeScript remain future native-binding surfaces.

## Boundaries and dependencies

```text
Avalonia -> sdks/c (P/Invoke C ABI) -> sdks/rust (typed facade) -> agent core harness
                                                        |-> data/config/llm/tools
```

The C binding never accesses persistence or provider internals directly.

## Compatibility and migration

The ABI remains version 4 with the same symbol family and JSON envelopes. Only the Cargo package that emits the native library changes.

## Risks and rollback

The main risk is visibility/API churn while extracting the facade. Rollback is a source-level revert because no persisted schema or wire contract changes.
