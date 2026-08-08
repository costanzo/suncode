# Suncode Architecture

**Status:** Approved

**Date:** 2026-08-08

**Scope:** Phase 1 process topology, ownership boundaries, authority, persistence, protocols, and repository layout

## 1. Purpose

Suncode is a local-first coding agent. Phase 1 is one Rust runtime core serving one Qt desktop client through a reusable local SDK facade. Rust owns the complete runtime: provider integration, agent behavior, policy, durable state, client API, and machine-affecting operations.

The architecture favors local ownership, reviewable authority, and one authoritative runtime. It does not claim that a process running as the user is an OS sandbox.

## 2. Process Topology

```text
Qt desktop
    |
Rust SDK facade
    |
Rust Suncode runtime core
    |- client API and authentication
    |- DeepSeek provider and agent loop
    |- context, policy, approvals, and scheduling
    |- SQLite, settings, events, and credentials
    |- filesystem, search, process, and artifacts
    `- checkpoints and operation journal

HTTP + SSE adapter
    |
    `- same Rust runtime core
```

There is no runtime-to-core process boundary. Operations are Rust modules called in-process after policy authorization. The old TypeScript runtime, core client, runtime server, and JSON-RPC stdio core have been removed from the production tree after parity verification. HTTP/SSE remains as an adapter surface for future non-Qt clients and compatibility use.

## 3. Ownership Boundaries

### 3.1 Qt desktop

Qt owns presentation, navigation, and transient interaction state. It consumes runtime DTOs and ordered events through the SDK facade. It never opens SQLite, contacts DeepSeek, reads project files directly, or invokes operation modules.

Phase 1 has no CLI, TUI, Web, mobile, or IDE client.

### 3.2 Rust runtime

The runtime core owns:

- DeepSeek V4 model integration and canonical provider messages
- context construction, turn scheduling, budgets, cancellation, and the agent loop
- tool registration, policy evaluation, durable approvals, and audit records
- authenticated HTTP requests, snapshots, SSE replay, live event delivery, and SDK dispatch
- SQLite migrations, transactions, projections, settings, and local event streams
- provider credentials through the OS credential store
- project boundary checks and machine-affecting operations
- checkpoints, undo, managed artifacts, and operation reconciliation

Provider and orchestration modules cannot perform project operations directly. They construct typed operation requests which pass through policy and the runtime operation dispatcher.

### 3.3 Operations

Operations are narrow Rust modules inside the runtime. They own canonical path validation, bounded reads/searches, mutations, process execution, checkpoint payloads, artifacts, and operation journal records. They do not own provider semantics, conversation state, UI DTOs, or policy grants.

This internal boundary is for auditability and testing. It is not a child-process security boundary.

## 4. Runtime Lifecycle

One runtime instance exists per OS user. It acquires a single-instance lock, opens and migrates SQLite, reconciles interrupted local work, creates a random runtime credential, binds loopback, and writes a user-readable discovery record with restrictive permissions. The discovery record contains only endpoint, runtime credential, version, and process metadata needed by the launcher.

The Qt client discovers or launches the runtime, authenticates every request, fetches a session snapshot, then resumes ordered events by content sequence. Reconnect never treats client cache as authoritative.

## 5. Client Protocol

Phase 1 keeps the documented authenticated HTTP and SSE contract in `contracts/client-runtime/` for the adapter surface. The Qt desktop client uses the Rust SDK facade directly. DTOs are hand-implemented in Rust and Qt and verified with shared vectors. Contract generation is prohibited.

Mutating requests carry idempotency keys. Session events have a strictly increasing per-session content sequence. SSE first replays retained events after the supplied cursor and then delivers live events without reordering.

## 6. Provider Boundary

The first built-in provider is DeepSeek and the stable Suncode model identity is `deepseek-v4-flash`. Vendor request and streaming response shapes remain inside the provider adapter. Clients receive canonical messages, tool activity, usage, and redacted errors only.

The API key is read from the OS credential store. Plaintext credentials never enter SQLite, protocol responses, events, audit rows, or logs. An environment override is allowed only in an explicitly configured non-interactive execution mode.

## 7. Persistence

Rust is the only database owner. Qt, providers, and future extensions never open the database.

SQLite keeps separate durable concerns:

- immutable audit records for authority decisions and outcomes
- compactable, ordered session content as the rebuild source for durable message and lifecycle events
- query projections for projects, sessions, turns, tool calls, messages, approvals, and checkpoint manifests
- ephemeral live streaming deltas that are broadcast to connected clients but not retained
- disposable reconnect cursors
- durable turn admission and approval continuation
- scoped settings and encrypted-secret metadata

The Phase 1 schema preserves version 10 compatibility during the TypeScript-to-Rust migration. New migrations are append-only and transactional. JSON stores evolving payloads; identifiers, state, ordering, timestamps, scope, and foreign keys remain queryable columns.

## 8. Authority Model

Every tool call is validated, assigned a declared risk, evaluated by policy, and audited before execution. Read-only project inspection is allowed by the interactive default. Writes, process execution, network use outside the configured provider, secret access, destructive operations, and external paths require an explicit grant or user approval. Non-interactive execution fails closed without a matching profile grant.

Approval precedes execution. Approval requests and suspended continuations are durable and single-use. A restart may reconcile an operation with a durable idempotency record but must not blindly replay a provider call with unknown completion.

## 9. Reversibility and Recovery

Filesystem mutations capture pre-image checkpoints before changing disk. A turn-level manifest is the Qt undo unit and restores items in reverse operation order with post-image conflict checks. Process operations report the isolation actually enforced on the current platform; filtered environment or project-scoped working directory must never be described as network or OS sandboxing.

Startup marks non-recoverable in-memory turn execution interrupted, discovers admitted submissions and suspended approvals, and reconciles operation journal entries. Unknown completion remains visible and requires safe reconciliation.

## 10. Repository Layout

```text
apps/desktop-qt/          Qt desktop client
contracts/                hand-written protocols and shared vectors
runtime/crates/core/      runtime core, SDK facade, and HTTP adapter binary
runtime/crates/operations/ audited in-process machine operations
sdks/                     language SDK packaging surfaces
.agents/                  durable product and engineering knowledge
```

The old `typescript/` packages and retired `rust/` workspace were migration sources and are removed from the production tree. Language SDK directories may contain placeholder documentation before implementation starts, but they must not pretend to ship a working package until one exists.

## 11. Dependency Rules

- Qt depends only on Qt and the client protocol.
- Client API handlers call runtime services, never SQLite or provider wire types directly.
- Agent and provider modules call operations through the authorized dispatcher.
- Persistence does not depend on Qt, HTTP, or provider wire types.
- Operations do not depend on agent, provider, persistence projections, or client DTOs.
- No production TypeScript or Node.js process remains in Phase 1.

## 12. Deferred Scope

Phase 1 defers CLI/TUI/Web clients, executable plugins, MCP servers, third-party provider adapters, PTY interaction, hosted execution, collaboration, telemetry, filesystem indexing/watchers, VCS-aware semantic operations, and cross-platform OS sandbox profiles. Adding executable third-party code requires a separate isolation design.
