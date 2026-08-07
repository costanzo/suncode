# Implementation Plan

This is a proposed later implementation sequence. No runtime implementation is part of the current requirements delivery.

The ordering follows one rule: get a single tool call working end to end before building the mechanisms that manage many of them. Steps 6 through 9 are the vertical slice that makes every later step verifiable.

## Decisions first

1. [ ] Approve runtime scope, reference synthesis, and trust boundaries.
2. [ ] Answer the blocking questions in `requirement.md`: first provider and authentication method, canonical message schema, default approval policy, and default budgets.
3. [ ] Write the runtime-to-core contract document and its first test vectors.
4. [ ] Write the client-runtime contract document and its first test vectors.
5. [ ] Specify the three durable streams: content taxonomy, audit record shape, projections, per-stream retention, secret classification, replay.

## Vertical slice

6. [ ] Implement runtime lifecycle, layered configuration, database, and core supervision.
7. [ ] Implement the provider gateway for the first provider only, with streaming and usage normalization.
8. [ ] Implement the two-level state machine with one read-only tool, driven through policy, execution, and audit.
9. [ ] Stand up the behavioral evaluation suite against that slice. It exists before the tool catalog grows, because it is the only regression net for prompt and tool-description changes.

## Breadth

10. [ ] Implement the context engine, instruction precedence, provenance, and compaction.
11. [ ] Implement the tool registry and the remaining core-backed operation adapters.
12. [ ] Implement the policy engine, approval lifecycle, policy profiles, and audit stream.
13. [ ] Implement checkpoint capture, restore, and conflict reporting.
14. [ ] Implement input admission, delivery modes, and the turn scheduler.
15. [ ] Implement recovery: core restart, reconciliation by hash, unknown completion, projection rebuild.
16. [ ] Implement single-process mode with non-interactive output and exit codes.
17. [ ] Implement built-in skills and skill discovery.
18. [ ] Implement the MCP client, namespaced registry, and server health.
19. [ ] Implement the plugin manifest, host boundary, worker lifecycle, and quarantine.
20. [ ] Integrate the CLI/TUI surface against the client API.
21. [ ] Add conformance, security, fault-injection, performance, and cross-platform tests.
22. [ ] Promote stable behavior into features/specs and record durable decisions.

Plugins and MCP sit late deliberately: both add trust surface, and neither is needed to prove the loop works.
