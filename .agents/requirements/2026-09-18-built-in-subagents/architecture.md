# Architecture

## Current state

One Rust agent loop owns primary sessions. Every model request currently receives all built-in tools plus the current project MCP catalog. Sessions have no parent/child classification, and the SDK lists every active project session as a peer.

## Proposed design

- Add a Rust `BuiltinAgentRegistry` containing six immutable definitions.
- Add the core-owned `delegate_agent` conversation tool to primary model requests.
- Extend session persistence with kind, parent identity, agent identity, and definition version.
- Add `subagent_invocation` as the durable correlation between parent turn/tool call and child session.
- Snapshot the specialist identity and tool allowlist into the child continuation.
- Filter advertised tools and revalidate every returned tool call against the continuation allowlist.
- Run a delegated child turn through the same provider, policy, dispatcher, persistence, event, and recovery path as any other session.

## Boundaries and dependencies

- Rust remains the only catalog, orchestration, policy, and SQLite owner.
- `suncode-tool` continues to own neutral machine tools; it does not know agent roles.
- The client reads typed SDK DTOs and never infers agent definitions or opens SQLite.
- Agent definitions are not configuration rows and have no mutation API.

## Data and control flow

1. Primary model calls `delegate_agent(agent, task)`.
2. Core validates the built-in definition and delegation limit.
3. Core persists an invocation and creates a linked child session.
4. Core submits the task internally with inherited model settings and specialist instructions/tool allowlist.
5. The child uses normal model/tool orchestration under its restricted catalog.
6. Core terminally updates the invocation and returns a normalized result to the parent tool call. If the child suspends for approval, the parent receives an `awaiting_approval` result and may finish while the child remains durable.
7. The primary agent continues and produces the user-facing response. Resolving a child approval later resumes the child independently and updates the invocation plus child-session detail.

## Security and failure handling

- Tool allowlists reduce advertised capability but never replace policy authorization.
- Child sessions cannot use MCP, ask the user directly, or delegate again.
- Sensitive child operations use the existing durable approval path.
- Unknown or disallowed tools fail before policy or execution.
- Restart marks incomplete child execution interrupted and preserves its session and invocation.

## Compatibility and migration

The database initializer accepts the immediately preceding valid schema and transactionally adds the invocation table and nullable/defaulted session columns. Unexpected schemas remain rejected. Existing sessions become `primary` sessions.

## Risks and rollback

- Nested async agent execution can complicate cancellation and recovery; keep ordinary delegation synchronous in the parent tool call and depth one. Approval suspension is the explicit exception: the parent may finish while the durable child waits and resumes independently.
- Parent/child checkpoint grouping must be explicit before claiming unified undo.
- Rollback can stop advertising `delegate_agent`; durable child rows remain readable.

## Open questions

- Whether a later release should allow curated MCP prefixes per built-in agent.
