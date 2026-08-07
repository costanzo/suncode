# Agent Runtime Phase 1

- Date: 2026-08-07
- Status: Draft
- Related features: None; the agent runtime is not implemented.
- Related specifications: `../../ARCHITECTURE.md`
- Related decisions: `../../DECISIONS.md` — `ADR-20260807-tool-call-state-machine`, `ADR-20260807-durable-stream-separation`, `ADR-20260807-local-first-scope`, `ADR-20260807-hand-written-protocol-contracts`, `ADR-20260807-runtime-owns-durable-state`, `ADR-20260807-domain-vocabulary`
- Related requirements: `../2026-08-07-os-layer-phase-1/README.md`, `../2026-08-07-desktop-ui-phase-1/README.md`

## Summary

Define the TypeScript/Node.js agent runtime that sits between Suncode clients and the Rust OS core. The runtime owns provider integrations, context construction, the agent loop, tool orchestration, skills, plugins, MCP connections, approvals, session persistence, settings, and client-facing events. This is a requirements and architecture delivery only; no runtime behavior is implemented here.

## Documents

- `requirement.md` - behavior, scope, and acceptance criteria
- `architecture.md` - runtime modules, state machines, extension boundaries, and flows
- `changes.md` - affected documentation
- `plan.md` - later implementation sequence
- `progress.md` - documentation status
- `todo.md` - decisions and future work
- `test-plan.md` - verification strategy

## Reference review

- `opencode-comparison.md` - source-level comparison with OpenCode
- `opencode-amendments.md` - **consolidated into `requirement.md` and `architecture.md` on 2026-08-07.** Retained as reasoning history; it is no longer a separate normative source.
- `opencode-review-status.md` - review record

## Current state

`requirement.md` and `architecture.md` are the only normative documents in this package. Both were revised on 2026-08-07 to adopt the two-level tool-call state machine, the three durable streams, non-interactive policy profiles, layered configuration, checkpoints, and the local-first and hand-written-contract decisions.

