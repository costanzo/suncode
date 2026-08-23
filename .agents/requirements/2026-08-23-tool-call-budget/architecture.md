# Architecture

## Current state

The agent loop hard-codes 32 calls and increments the counter while iterating through a provider-returned batch. Turn state events and the final failure write independently project terminal state, but the latter currently excludes already failed rows.

## Proposed design

Store `tool_call_limit` in the existing project-scoped `configuration` table. Resolve it through a typed database helper when admitting a turn, default it to 64, and snapshot it into `Continuation` for approval and restart compatibility.

Before processing a provider tool-call batch, compare its length to the turn's remaining budget. On overflow, project each returned call as requested and failed, then fail the turn. This creates complete observable tool history without entering validation, policy, approval, or execution.

## Boundaries and dependencies

- Database owns typed configuration resolution and terminal persistence.
- Core SDK owns setting scope/type/range validation.
- Agent owns budget enforcement and continuation snapshots.
- Avalonia reads and writes only through the SDK facade.

## Data and control flow

```text
Settings NumberUpDown -> C# SDK wrapper -> Rust SDK validation
    -> configuration(project, tool_call_limit)
    -> turn admission resolves value or 64
    -> Continuation.tool_call_limit
    -> batch preflight before tool policy/execution
```

## Security and failure handling

Budget overflow cannot leave a returned call in `policy_check`, `authorized`, or `executing`. Terminal persistence may enrich only failed rows and cannot replace other terminal outcomes.

## Compatibility and migration

No schema migration is required. Existing projects inherit 64. Existing serialized continuations use the serde default of 64.

## Risks and rollback

Increasing the limit permits longer turns and higher provider/tool usage. The bounded project control lets users lower it, while the hard maximum prevents unbounded configuration.

## Open questions

- None.
