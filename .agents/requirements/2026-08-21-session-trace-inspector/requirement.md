# Requirement

## Background

The ProjectWorkspace provider trace drawer currently presents a flat list of model calls and raw JSON sections. It does not expose the turn hierarchy or the normalized messages and tool uses related to one call.

## Goals

- Present every turn in the selected session as the root of an expandable trace tree.
- Present each model call beneath its owning turn.
- Show the elapsed time consumed by each turn.
- Inspect one call in a right-side detail surface modeled after the Git viewer.
- Show associated messages, tool uses, request/response data, token usage, cache tokens, and cache hit rate.

## Non-goals

- Change session persistence or provider execution behavior.
- Store duplicate trace records.
- Expose raw provider HTTP payloads, authorization headers, or credentials.

## Requirements

- Turns without model calls remain visible.
- Every turn row displays elapsed time from its start until completion, or until now while it is running.
- Trace filtering preserves the turn/call hierarchy.
- Cache metrics remain unavailable when the provider does not report them.
- Call detail reads normalized `session_call`, `session_message`, and `session_tool_use` rows through the Rust SDK.
- Loading, empty, running, completed, failed, and missing-usage states remain legible.

## Edge cases

- A running call has no response or completed timestamp.
- A failed call may have an error but no usage.
- A call may have no directly correlated messages or tool uses.
- A provider may report cache-read tokens, cache-write tokens, both, or neither.

## Acceptance criteria

- The trace drawer shows a turn/call tree for the current session.
- Each turn in the trace tree shows its elapsed time.
- Selecting a call loads its complete normalized detail.
- Usage metrics include input, output, cache read, cache write, total, and cache hit rate.
- Runtime and desktop focused tests/builds pass.

## Open questions

- None.
