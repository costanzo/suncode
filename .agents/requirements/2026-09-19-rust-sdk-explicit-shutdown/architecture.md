# Architecture

## Current state

`AsyncAgentSdk` releases resources through ordinary drop. The blocking adapter additionally drops its Tokio runtime, which aborts runtime tasks but does not express resource ordering. Core managers expose project- or server-level lifecycle methods but no unified drain operation. The C close function only drops the boxed handle.

## Proposed design

Add consuming shutdown methods to both Rust facades. `AsyncAgentSdk::shutdown(self)` delegates to one idempotent core `Agent::shutdown`, logs the result, and then releases state and the data-directory lock as the consumed facade is dropped. `blocking::AgentSdk::shutdown(self)` keeps its owned runtime alive while awaiting the async shutdown.

Core owns a shared shutdown flag. Shutdown sets it once, cancels every active turn token, clears queued input, triggers Computer Use emergency stop, and concurrently drains Browser, MCP, and LSP managers. It then waits up to five seconds for active turn tracking to become empty and closes every event-hub subscriber.

Each process-owning manager has its own closed flag and a drain method. New starts fail after closure. Starts already in progress cannot reinstall a late resource because shutdown removes their runtime slots; any late successful client or worker is explicitly closed.

The existing C close symbol extracts its blocking SDK from the opaque handle, runs consuming shutdown, logs any cleanup failure, and returns through its unchanged `void` ABI.

## Boundaries and dependencies

- Agent core owns cancellation and resource shutdown ordering.
- Rust SDK owns handle consumption, logging, and data-directory lock lifetime.
- The blocking adapter owns executor lifetime during shutdown.
- C owns panic containment and log-only error adaptation for its existing close signature.
- C# and Avalonia continue closing subscription handles before releasing the shared agent handle.

## Data and control flow

1. Host consumes its SDK with `shutdown` or releases the final native handle.
2. Core marks the agent shutting down and rejects new turn/continuation admissions.
3. Active turn tokens are cancelled and queued follow-up input is cleared.
4. Computer, Browser, MCP, and LSP resources stop concurrently.
5. Core waits up to five seconds for active turn bookkeeping to drain.
6. Event subscriptions close and pending receivers wake.
7. The facade is consumed, releasing SQLite ownership and the data-directory lock.

## Security and failure handling

Shutdown does not expand authority. Computer Use releases held inputs before the backend is dropped. External process closure remains bounded; a non-cooperative turn produces an `agent_unavailable` shutdown error after the grace deadline. Cleanup continues across manager categories before an error is returned.

## Compatibility and migration

The Rust API change is additive. Rust hosts should call explicit shutdown; dropping remains a fallback but does not promise graceful async cleanup. C keeps the current close symbol and ABI version. C# and Avalonia need no source change because their final handle release already calls native close.

## Risks and rollback

Risks include shutdown/start races, waiting forever on a turn, closing a shared connection while a turn is unwinding, and releasing the data lock before background clones retire. Closed flags, slot draining, late-client retirement, cancellation, and a bounded grace period mitigate these risks. Rollback removes explicit shutdown and restores drop-only cleanup without persistence changes.

## Open questions

- Whether a future host-facing shutdown report should expose counts of cancelled turns and closed external processes.
