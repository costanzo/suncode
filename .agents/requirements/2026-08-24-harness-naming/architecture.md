# Architecture

## Current state

One Rust package under `agent/crates/core` owns the embedded agent SDK. Avalonia calls its method-oriented C ABI and does not access durable state directly.

## Proposed design

Use `harness` as the product-facing name for the agent orchestration and authority boundary. Keep `tokio::runtime::Runtime` as the implementation type for asynchronous scheduling. The Rust package emits `suncode_agent` and exports `suncode_agent_sdk_*` symbols. The C# wrapper exposes `AgentSdk` in `SunCode.Desktop.Agent`.

## Compatibility and migration

The ABI version is 2 because native symbol names and health/error DTO identifiers changed. Rebuilt hosts are required. The default database path is `harness.sqlite3`; an existing `runtime.sqlite3` is selected as a legacy fallback. The old macOS Keychain service remains a legacy import source.

## Risks and rollback

The main risk is an external native host still looking for the version 1 library or symbols. Rebuilding the host against the version 2 contract resolves it. Runtime behavior and stored table data are unchanged.
