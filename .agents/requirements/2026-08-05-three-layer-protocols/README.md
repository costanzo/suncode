# Three-Layer Protocol Brainstorm

- Date: 2026-08-05
- Status: Draft
- Scope: Architecture exploration only; no implementation is authorized or claimed.

## Problem

Suncode needs a clear three-layer design:

```text
Qt desktop | web | mobile
             |
 JSON-RPC 2.0 over authenticated WebSocket
             |
 TypeScript agent runtime
             |
 JSON-RPC 2.0 over newline-delimited stdio
             |
 Rust OS core
```

The existing foundation already places AI orchestration in TypeScript and trusted machine operations in Rust, but it defers the UI/runtime protocol and names a different set of clients.

## Proposed responsibilities

### UI layer

- Own presentation, navigation, local interaction state, and platform integration.
- Support Qt desktop, web, and mobile clients.
- Require Qt for the desktop application; Electron is prohibited.
- Communicate only with the agent runtime.
- Never access Rust, SQLite, model providers, secrets, or OS capabilities directly.

### Agent runtime layer

- Use TypeScript for model integrations, context construction, agent loops, and orchestration.
- Expose a presentation-safe WebSocket RPC API.
- Translate user intent into typed OS-core operations rather than proxying raw core methods.
- Supervise the Rust child and fan out ordered session events to connected UIs.

### OS layer

- Use Rust for high-performance and security-sensitive work.
- Own filesystem and process operations, canonical paths, permissions, durable sessions, SQLite, secrets, and artifacts.
- Remain the authoritative enforcement boundary even when upper layers validate requests.

## Proposed protocols

Both boundaries use JSON-RPC 2.0, OpenRPC method descriptions, and JSON Schema payloads. They remain separate contracts and version independently.

### UI to agent runtime

- Transport: authenticated WebSocket using one JSON-RPC message per text frame.
- Initialization: negotiate versions, capabilities, limits, client kind, and resume cursors.
- Main domains: runtime, workspace, session, turn, approval, settings, model, artifact, and events.
- Streaming state: ordered `session.event` notifications with per-session sequence numbers.
- Recovery: reconnect after the last applied sequence or fetch a fresh snapshot when replay is unavailable.
- Multiple clients: allow observation of the same session while keeping mutations explicit, authorized, and idempotent.

### Agent runtime to OS core

- Transport: JSON-RPC 2.0 as newline-delimited UTF-8 JSON over child-process stdin/stdout.
- Initialization: negotiate versions, capabilities, limits, platform information, database schema, and recovery support.
- Main domains: workspace, sessions, operations, permissions, settings, secrets, artifacts, and recovery.
- Long-running work: stable operation IDs, progress notifications, explicit cancellation, and idempotency keys.
- Security: Rust canonicalizes paths, evaluates policy, persists approvals, and never trusts UI authorization directly.
- Recovery: restart the Rust child with bounded backoff and reconcile unknown outcomes without blind retries.

## Contract organization

```text
contracts/
├── openrpc/
│   ├── ui-runtime.json
│   └── runtime-os.json
├── schemas/
│   ├── common/
│   ├── ui-runtime/
│   └── runtime-os/
├── examples/
└── fixtures/
```

Common schemas should contain only transport-neutral values. Privileged core request types must not become UI API types automatically.

## Constraints and assumptions

- The initial topology remains local and same-device.
- Mobile does not imply LAN, hosted, or cross-device runtime access.
- Large or binary values use bounded pages or permission-checked artifact references rather than unbounded JSON.
- Authentication token discovery and browser-safe handoff need a separate security design.
- Qt binding, mobile framework, complete method catalogs, and event-retention policy remain open.

## Before implementation

1. Review and approve or revise this architecture proposal.
2. Resolve endpoint discovery, credential handoff, origin validation, and mobile lifecycle.
3. Define canonical OpenRPC documents and JSON Schemas.
4. Add examples, invalid fixtures, compatibility tests, and deterministic type generation.
5. Create an implementation requirement only when the contract scope is accepted.
