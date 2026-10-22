# Architecture

## Current state

`AgentRuntime` owns a per-session in-memory `VecDeque<QueuedMessage>`. The queue stores stable IDs, idempotency keys, complete text/image references, FIFO order, and either follow-up or steering mode. It is exposed through Rust SDK, C ABI 25, and managed DTOs.

## Proposed design

Keep transient queue ownership in the Rust agent. Provide typed SDK operations to query rows, promote a row to steering, remove/restore, and take a row back into the composer. When a draft exists, the take-back operation atomically replaces the selected queue row with that draft as a follow-up, preserving order and avoiding a lossy intermediate state.

The running turn checks for pending steering immediately before every provider request. It atomically claims one steering row, validates its images against the active model, appends the complete user message to context and emits the normal user-message projection correlated to the active turn. If the turn terminates first, unconsumed steering rows become follow-ups. The currently running provider future or tool is never cancelled by steering.

When a turn completes, core releases its active-turn/session guard before admitting queued follow-ups. A worker drains FIFO rows one at a time through the normal turn-admission path, revalidating model and attachment capability. Admission failure retains the row and pauses automatic draining for user recovery. Cancellation, failure, or suspension keeps all remaining rows queued. A follow-up starts a distinct durable turn and does not inherit the completed turn's cancellation token, iteration budget, or checkpoint.

Queue snapshots and live events are Rust-owned. Avalonia consumes them through the existing SDK watch/event stream and renders the design-system states. Draft, file storage, and focus remain desktop-owned; all queue image IDs are validated by Rust. Queue lifetime ends with the embedded agent process.

## Boundaries and dependencies

- `agent/crates/core` owns queue invariants, ordering, steering consumption, turn transitions, and events.
- `sdks/rust` owns public typed methods/DTOs and error translation.
- `sdks/c` owns additive native ABI operations; `sdks/csharp` owns matching interop and managed DTO adaptation.
- `apps/desktop-avalonia` owns rendering, undo affordance, draft restoration, disclosure state, and queue collapse state.
- `design-system/` remains specification/review tooling and defines visual/interaction behavior; it is not a production dependency.

## Data and control flow

1. Desktop submits during active work; Rust validates IDs and stores a complete follow-up row, returning its stable ID/order.
2. The desktop refreshes/receives a queue event and updates the bounded list.
3. A row-level steering command changes only that row's mode after validating the active turn.
4. The active loop claims steering at the next model boundary and emits its message under the existing turn.
5. At terminal turn cleanup, remaining follow-ups are drained in order and each is admitted through standard new-turn validation.
6. Queue mutation or event lag triggers an authoritative queue snapshot refresh.

## Security and failure handling

Queue input is untrusted user data and cannot carry filesystem paths or bypass model validation. Images remain agent-owned references. Queue operations are scoped to user-accessible primary sessions, never specialist child sessions. Lock scopes do not cross `await`; session ordering uses the existing per-session serialization primitives. Idempotency prevents duplicate queue rows and follow-up turns. Queue admission errors preserve the complete row for user recovery.

## Compatibility and migration

No SQLite migration is required. Additive SDK/ABI methods retain existing symbols and event envelope. The old `queued` response gains a queue ID and true after-turn semantics; `MessageUser` correlation remains available. Existing clients continue to compile but will only see their current generic queued status until upgraded. C# managed and native ABI versions must be updated together.

## Risks and rollback

The main risk is re-entering turn admission while completion still owns the active-turn/session lock. Follow-up draining must run after terminal projection and lock release, with explicit state transitions if a next item cannot be admitted. In-memory queues are lost on process shutdown by design. Rollback can keep the additive SDK surface while reverting to normal-turn-only behavior, but must not reintroduce safe-point injection under the follow-up label.

## Open questions

- None.
