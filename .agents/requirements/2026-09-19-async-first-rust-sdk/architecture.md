# Architecture

## Current state

`AgentSdk` stores a Tokio runtime beside the agent state. Async core operations are exposed as synchronous methods that invoke `block_on`. All facade modules are implemented directly on that runtime-owning type.

## Proposed design

Rename the runtime-free facade to `AsyncAgentSdk`. It owns the data-directory lock, data directory, and composed `AgentState`, but uses the caller's active Tokio runtime for startup and awaited operations. Pure local operations remain synchronous methods on this type.

Add `facade::blocking::AgentSdk`, containing one Tokio runtime and one `AsyncAgentSdk`. It implements `Deref<Target = AsyncAgentSdk>` so all synchronous methods are reused directly. It defines blocking counterparts only for methods that are async on `AsyncAgentSdk`, delegating with its owned runtime.

The crate root exports `AsyncAgentSdk` and continues exporting `AgentSdk` as the blocking compatibility type. C remains unchanged apart from importing the compatibility type.

## Boundaries and dependencies

- Async facade owns SDK semantics and state composition.
- Blocking wrapper owns executor creation and blocking adaptation only.
- Core agent remains async where lifecycle or external processes require it.
- SQLite and bounded local operations remain synchronous.

## Data and control flow

Async host:

1. Host runtime calls `AsyncAgentSdk::open_default(...).await`.
2. SDK composes state and performs recovery on that runtime.
3. Awaited SDK methods directly await core operations.
4. Host selects event streams, cancellation, signals, and turns naturally.

Blocking host:

1. `blocking::AgentSdk::open_default` creates its owned Tokio runtime.
2. It blocks on async facade construction.
3. Async operations are adapted through wrapper methods; synchronous methods dereference to the shared facade.

## Security and failure handling

Runtime ownership does not change authority or trust. The blocking wrapper converts runtime creation failure to the existing unavailable error. Async and blocking facades own the same lock semantics, so one data directory still has one host process.

## Compatibility and migration

The C ABI and C# APIs remain unchanged. Rust code using root `AgentSdk` retains blocking behavior. New async Rust hosts use `AsyncAgentSdk`. A future major Rust API may make the async type the unqualified default after downstream migration.

## Risks and rollback

Risks include missing one `block_on`, accidental method-resolution changes through `Deref`, runtime drop ordering, and duplicated wrapper signatures drifting from async methods. Compilation of C and desktop bindings plus parity tests mitigate them. Rollback restores the runtime-owning facade without persistence changes.

## Open questions

- Whether the next Rust API major should rename `AsyncAgentSdk` to `AgentSdk` and move compatibility exclusively under `blocking`.
