# Three-Layer Protocol Brainstorm

- Date: 2026-08-05
- Status: **Superseded.** Its conclusions were absorbed into `../../ARCHITECTURE.md`; read that instead.
- Scope: Architecture exploration only; no implementation was authorized or claimed.

## Why superseded

This exploration settled the three-layer split and the two protocol boundaries, and `ARCHITECTURE.md` now carries both in current form. Later decisions changed several details recorded below, so the text is kept only as reasoning history:

- Cloud-hosted deployment is out of scope (`ADR-20260807-local-first-scope`). The cloud topology sketch and its tenancy constraints no longer apply.
- Contracts are hand-implemented per language (`ADR-20260807-hand-written-protocol-contracts`). The `contracts/` tree below assumed generated types; there is no `generated/` directory.
- The runtime-to-core transport uses two channels, not one, so heartbeats and cancellation cannot queue behind bulk output.
- "Workspace" is retired in favor of "project" (`ADR-20260807-domain-vocabulary`).
- The committed surfaces are CLI/TUI then Qt desktop, not the six listed here.

This package also predates the eight-file requirement template and was never brought into line with it. That is expected for a superseded record and should not be fixed.

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

## Deployment topologies

### Local

    Qt desktop or web UI
            | authenticated loopback WebSocket
    TypeScript agent runtime
            | private newline-delimited JSON-RPC
    Rust OS core -> local workspace

### Cloud hosted

    Qt desktop | web | mobile
            | authenticated WSS over TLS
    Cloud TypeScript agent runtime
            | private newline-delimited JSON-RPC
    Cloud Rust OS core -> isolated cloud workspace

The Node.js runtime and OS core stay colocated in both modes. Remote clients never connect directly to the Rust core, and the cloud Rust core operates on its authorized cloud workspace rather than the mobile or desktop device.

## Proposed responsibilities

### UI layer

- Own presentation, navigation, local interaction state, and platform integration.
- Support Qt desktop, web, mobile, CLI/TUI, and IDE-plugin clients.
- Require Qt for the desktop application; Electron is prohibited.
- Communicate only with the agent runtime.
- Never access the runtime database, Rust, model providers, secrets, or OS capabilities directly.

### Agent runtime layer

- Use TypeScript for model integrations, context construction, agent loops, and orchestration.
- Expose a presentation-safe WebSocket RPC API.
- Translate user intent into typed OS-core operations rather than proxying raw core methods.
- Use Node.js exclusively; Bun is prohibited.
- Own SQLite connections, migrations, settings, session history, projections, runtime-level approvals, and runtime logs.
- Supervise the Rust child and fan out ordered session events to connected UIs.

### OS layer

- Use Rust for high-performance and security-sensitive work.
- Own filesystem, process, sandbox, canonical-path, and OS-capability operations.
- Return operation results and signals to the runtime; do not own SQLite, session history, settings, or runtime logs.
- Remain the authoritative enforcement boundary even when upper layers validate requests.

## Layered logs

Each layer writes to a separate file or cloud log stream:

- UI layer: client interaction, connection, rendering, and adapter logs.
- Agent runtime layer: Node.js lifecycle, model, orchestration, SQLite, session, and protocol logs.
- OS layer: Rust operation, sandbox, capability, process, and filesystem logs.

All three streams use correlation IDs where safe and redact credentials, tokens, model secrets, and sensitive file contents. Local paths follow platform application-data conventions; cloud deployments use tenant-scoped durable log storage or an approved collector. Logs never use Rust protocol stdout and are never committed to the repository.

## Proposed protocols

Both boundaries use JSON-RPC 2.0, OpenRPC method descriptions, and JSON Schema payloads. They remain separate contracts and version independently.

### UI to agent runtime

- Transport: authenticated WebSocket using one JSON-RPC message per text frame; local connections use loopback, while remote connections require WSS over TLS.
- Initialization: negotiate versions, capabilities, limits, client kind, and resume cursors.
- Main domains: runtime, workspace, session, turn, approval, settings, model, artifact, and events.
- Streaming state: ordered `session.event` notifications with per-session sequence numbers.
- Recovery: reconnect after the last applied sequence or fetch a fresh snapshot when replay is unavailable.
- Multiple clients: allow observation of the same session while keeping mutations explicit, authorized, and idempotent.

### Agent runtime to OS core

- Transport: JSON-RPC 2.0 as newline-delimited UTF-8 JSON over child-process stdin/stdout.
- Initialization: negotiate versions, capabilities, limits, platform information, database schema, and recovery support.
- Main domains: workspace, operations, OS-capabilities, artifacts, and recovery signals; session persistence belongs to the runtime.
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

- Local and cloud-hosted deployments are both supported target topologies.
- Cloud access requires authenticated WSS, short-lived revocable credentials, and authorization scoped to user, tenant, workspace, and session.
- Each cloud runtime/core environment must isolate processes, workspaces, storage, secrets, network access, and resource limits from other tenants.
- The cloud Rust core acts only on its provisioned cloud workspace; remote access does not grant it control of the client device.
- Large or binary values use bounded pages or permission-checked artifact references rather than unbounded JSON.
- Authentication token discovery and browser-safe handoff need a separate security design.
- Qt binding, mobile framework, complete method catalogs, event-retention policy, and the layered log-file format remain open.

## Before implementation

1. Review and approve or revise this architecture proposal.
2. Resolve local discovery and credential handoff plus remote identity, WSS/TLS termination, origin validation, tenant authorization, workspace provisioning, and isolation.
3. Define canonical OpenRPC documents and JSON Schemas.
4. Add examples, invalid fixtures, compatibility tests, and deterministic type generation.
5. Create an implementation requirement only when the contract scope is accepted.
