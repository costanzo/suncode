# Changes

## Source

- None. This delivery defines the runtime requirements and architecture only.

## Contracts

- None. The package identifies the required future contract domains but does not write contract documents or test vectors. Nothing is generated; each language will implement its own types.

## Configuration and persistence

- None. The package defines ownership and behavior that a later persistence/security design must specify.

## Tests

- Added a requirements review, conformance, security, fault-injection, and performance test plan.

## Documentation

- Added the Phase 1 TypeScript/Node.js agent runtime requirement package.
- Defined provider gateway, agent loop, context engine, tools, skills, plugins, MCP, policy, persistence, recovery, and observability.
- Recorded open decisions and implementation sequencing.

### Revision, 2026-08-07

- Replaced the linear turn state machine with a two-level turn and tool-call machine so multiple tool calls per assistant message are representable.
- Corrected approval-before-execution ordering in both the requirement and the turn-flow diagram.
- Split durable state into audit, session content, and client sync streams with independent retention.
- Added non-interactive execution: single-process mode, policy profiles, typed denials, script-friendly output and exit codes.
- Added layered configuration with an untrusted committed project file.
- Added checkpoint requirements for reversible filesystem changes.
- Removed cloud, tenancy, and generated-artifact assumptions per the local-first and hand-written-contract decisions.
- Renamed "workspace" to "project" throughout and split turn from tool-call identifiers.
- Marked `opencode-amendments.md` consolidated and non-normative.
- Reordered `plan.md` around a vertical slice, moving the evaluation suite ahead of tool-catalog breadth.

