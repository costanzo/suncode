# Progress

- Status: Draft
- Last updated: 2026-08-07

## Completed

- Reviewed approved product and architecture boundaries.
- Synthesized reference patterns from Codex, Claude Code, and OpenCode into Suncode-specific principles.
- Defined runtime responsibilities for providers, context, agent loop, tools, skills, plugins, MCP, permissions, sessions, and observability.
- Defined proposed topology, turn flow, extension trust boundaries, and recovery behavior.
- Consolidated the OpenCode amendments into `requirement.md` and `architecture.md`; that file is now history rather than a normative source.
- Replaced the single linear turn state machine with the two-level turn and tool-call machine, so several tool calls in one assistant message are the normal case.
- Corrected approval ordering: authorization strictly precedes execution everywhere.
- Split durable state into audit, session content, and client sync streams with independent retention.
- Added non-interactive execution: single-process mode, policy profiles, typed denial instead of blocking prompts.
- Added layered configuration, replacing the interface-only rule that conflicted with project-level skill discovery.
- Added checkpoints so the agent's filesystem changes are reversible.
- Applied the local-first scope and hand-written-contract decisions, removing tenancy and generated-type assumptions.
- Aligned vocabulary on project, session, and turn.

## In progress

- Product, security, persistence, and protocol review.

## Blocked

- Functional implementation is intentionally not started.
- The blocking questions in `requirement.md` must be answered first, starting with the first provider and its authentication method, since credential storage, first-run experience, and cost presentation all depend on it.

## Log

### 2026-08-07

- Requirement package initialized as documentation-only work.
- Kept Rust as the machine-affecting execution authority and Node.js/TypeScript as the runtime state owner.
- Revised the package after architecture review. The substantive corrections were the state-machine shape, approval ordering, and stream separation; the rest followed from the local-first and no-code-generation decisions.
- Restated the core boundary's value as containment of third-party code and auditability rather than OS-enforced isolation, since the runtime spawns the core as a child under the same user and can bypass it.
