# Requirement

## Background

The async-first SDK owns no runtime, but it also has no explicit asynchronous shutdown contract. Dropping the SDK releases ordinary Rust values, while background approval recovery, MCP connection monitoring, language servers, Browser Use workers, Computer Use state, active turns, and event subscriptions may still be attached to a host runtime. The blocking adapter currently relies on runtime destruction to abort remaining work.

## Goals

- Provide an explicit consuming `AsyncAgentSdk::shutdown().await` operation.
- Provide a synchronous consuming shutdown operation on `blocking::AgentSdk`.
- Stop accepting new turn and continuation work once shutdown begins.
- Cancel active turn work and clear queued follow-up input.
- Stop Computer Use, Browser Use, MCP, and language-server resources.
- Close live session streams and release the data-directory lock.
- Make native handle close execute the same shutdown path without changing the C ABI.

## Non-goals

- Adding a client-visible shutdown progress protocol.
- Persisting a new shutdown state or schema field.
- Guaranteeing indefinite waits for a non-cooperative third-party process.
- Changing ordinary turn cancellation, recovery, or approval contracts.
- Adding CLI behavior.

## Requirements

- Shutdown consumes the SDK handle so safe Rust cannot call it afterward.
- Core shutdown is idempotent for internal cloned agent handles.
- New turn submissions and approval/question continuations fail with `agent_shutting_down` once shutdown starts.
- Every active cancellation token is cancelled and queued submissions are discarded.
- Computer Use releases held input and retires the active frame.
- Browser workers, MCP connections, and language-server clients are drained and closed.
- Runtime managers reject or retire connections that finish starting after shutdown begins.
- Active turns receive a bounded five-second grace period after cancellation.
- All session event streams close before shutdown returns.
- The data-directory lock is released when the consuming shutdown completes, including error completion.
- Native close logs cleanup errors because its existing `void` ABI has no error return.

## Edge cases

- Shutdown during Browser, MCP, or LSP startup.
- Shutdown while an approval or question continuation runs in the background.
- A pending async or blocking event receiver.
- Repeated internal shutdown calls from cloned agent state.
- A Computer Use backend failure while releasing held input.
- A turn that does not observe cancellation before the grace deadline.

## Acceptance criteria

- Rust SDK and core compile with the explicit shutdown API.
- A shutdown closes pending session streams.
- A successfully shut down async SDK can reopen the same data directory immediately.
- C close invokes blocking shutdown while preserving its symbol and signature.
- Focused tests, formatting, applicable Clippy, and `git diff --check` pass, or unrelated repository blockers are recorded precisely.

## Open questions

- A future native ABI may add an error-returning asynchronous close method if desktop shutdown diagnostics need to be user-visible rather than log-only.
