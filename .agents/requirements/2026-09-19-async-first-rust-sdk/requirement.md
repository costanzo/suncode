# Requirement

## Background

The Rust facade owns a hidden Tokio runtime and implements asynchronous agent operations by calling `Runtime::block_on`. This suits C and C# but is unsuitable for Rust CLI/TUI and other async hosts: it hides runtime ownership, prevents natural `select!`, and risks nested-runtime misuse.

## Goals

- Make the primary Rust facade async-first and free of an internally created Tokio runtime.
- Keep pure persistence/DTO operations synchronous where they do not await agent work.
- Provide an explicit blocking wrapper for C, C#, and other synchronous hosts.
- Keep one implementation of validation, persistence, policy, and DTO behavior.
- Preserve the C ABI and managed API behavior.

## Non-goals

- Making Diesel or filesystem operations asynchronous.
- Changing event, snapshot, policy, persistence, or provider contracts.
- Adding CLI commands in this delivery.
- Replacing Tokio with a runtime-neutral executor abstraction.

## Requirements

- `AsyncAgentSdk::open_default` runs on the caller's Tokio runtime and creates no hidden runtime.
- Methods that await core agent, MCP, LSP, Browser Use, approval, question, or turn work are native async methods.
- Pure local reads/writes remain ordinary synchronous methods on the async facade.
- `blocking::AgentSdk` owns one Tokio runtime and delegates async methods through it.
- The blocking wrapper reuses synchronous facade methods without duplicating their bodies.
- Root compatibility may continue exporting the blocking `AgentSdk`; the async facade must be explicitly exported.
- C continues using the blocking wrapper with no wire change or ABI bump.

## Edge cases

- Opening from an existing Tokio runtime.
- Blocking wrapper creation outside any runtime.
- Dropping an async facade while background resources exist.
- Calling sync persistence methods from async code.
- Approval and question continuations.
- MCP/LSP DTO creation that itself queries async runtime status.

## Acceptance criteria

- Async tests open and use the SDK under `#[tokio::test]` without nested runtime panic.
- No `Runtime` or `block_on` remains in the async facade.
- The blocking C/C# path and desktop tests pass unchanged.
- Focused Clippy and formatting checks pass.

## Open questions

- A future delivery may add `spawn_blocking` variants for expensive synchronous SQLite projections if profiling justifies it.
