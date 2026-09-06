# Requirement

## Background

The Rust agent crate currently contains both the agent harness and the typed Rust SDK facade/C ABI used by Avalonia. This couples the internal harness crate to host-language binding details.

## Goals

- Keep `agent/crates/core` focused on agent execution and harness services.
- Move the typed Rust SDK facade under `sdks/rust`.
- Move the Avalonia-facing C ABI under `sdks/c`.
- Preserve the existing ABI version, symbols, DTOs, and behavior.
- Keep Python and TypeScript as explicit placeholders under `sdks`.

## Non-goals

- Changing the SDK contract or ABI semantics.
- Implementing Python or TypeScript bindings.
- Introducing a process boundary or network transport.

## Requirements

- Avalonia must build and package the native library produced by `sdks/c`.
- The C binding must depend on the typed Rust facade, not on SQLite, providers, or UI code directly.
- The typed facade may depend on the agent harness and Rust-owned data/provider/tool crates.
- `agent/crates/core` must no longer compile a `cdylib`, `staticlib`, or SDK module.

## Acceptance criteria

- `cargo test --workspace --all-targets` passes from the Rust workspace.
- Avalonia build invokes the C binding crate and copies the resulting platform library.
- The C ABI focused tests pass from `sdks/c`.
- Architecture and contract documents describe the new ownership boundary.
