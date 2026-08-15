# SunCode Architecture

**Status:** Approved

**Date:** 2026-08-08

**Scope:** Phase 1 process topology, ownership boundaries, authority, persistence, protocols, and repository layout

## 1. Purpose

SunCode is a local-first coding agent. Phase 1 embeds one Rust runtime SDK inside the .NET 10 Avalonia desktop process. Rust owns the complete runtime: provider integration, agent behavior, policy, durable state, the SDK API, and machine-affecting operations.

The architecture favors local ownership, reviewable authority, and one authoritative runtime. It does not claim that a process running as the user is an OS sandbox.

## 2. Process Topology

```text
.NET 10 Avalonia desktop
    | P/Invoke over C ABI
Embedded Rust SDK facade
    |
Rust SunCode runtime core
    |- typed SDK services and subscriptions
    |- built-in model providers and agent loop
    |- context, policy, approvals, and scheduling
    |- SQLite, settings, events, and credentials
    |- filesystem, search, process, and artifacts
    `- checkpoints and operation journal

Future TypeScript N-API and Python PyO3 bindings embed the same SDK.
```

There is no runtime-to-core process boundary and no client-facing server. Operations are Rust modules called in-process after policy authorization. The old TypeScript runtime, core client, runtime server, JSON-RPC stdio core, and loopback HTTP/SSE adapter are not production architecture. Provider adapters still make outbound HTTPS requests to configured model providers.

## 3. Ownership Boundaries

### 3.1 Avalonia desktop

Avalonia XAML and C# view models own presentation, navigation, and transient interaction state. They consume runtime DTOs and ordered events through the SDK facade. They never open SQLite, contact model providers, read project files directly, or invoke operation modules.

Phase 1 has no CLI, TUI, Web, mobile, or IDE client.

### 3.2 Rust runtime

The runtime core and SDK own:

- built-in model provider integrations and canonical provider messages
- context construction, turn scheduling, budgets, cancellation, and the agent loop
- tool registration, policy evaluation, durable approvals, and audit records
- typed SDK methods, snapshots, ordered event replay, and live subscription delivery
- SQLite migrations, transactions, projections, settings, and local event streams
- provider credentials through SQLite-owned local secret records
- project boundary checks and machine-affecting operations
- checkpoints, undo, managed artifacts, and operation reconciliation

Provider and orchestration modules cannot perform project operations directly. They construct typed operation requests which pass through policy and the runtime operation dispatcher.

### 3.3 Operations

Operations are narrow Rust modules inside the runtime. They own canonical path validation, bounded reads/searches, read-only Git repository inspection, mutations, process execution, checkpoint payloads, artifacts, and operation journal records. They do not own provider semantics, conversation state, UI DTOs, or policy grants.

This internal boundary is for auditability and testing. It is not a child-process security boundary.

## 4. Runtime Lifecycle

One runtime instance exists per data directory. Its host process acquires a single-instance lock, opens and migrates SQLite, reconciles interrupted local work, and retains the SDK handle until shutdown. It does not bind a client-facing socket, create a runtime credential, or publish an endpoint discovery record.

The Avalonia client embeds and opens the runtime, fetches a session snapshot, then resumes ordered events by content sequence through a direct subscription. Reconnect never treats client cache as authoritative. A second process cannot attach to an active runtime; replacement IPC requires a new architectural decision.

## 5. SDK Contract

Phase 1 keeps the embedded SDK contract in `contracts/runtime-sdk/`. C# calls named methods through the stable C ABI using P/Invoke. Future TypeScript and Python packages wrap the same Rust facade through native bindings. DTOs are hand-implemented in Rust and each host language and verified with shared vectors. Contract generation is prohibited.

Mutating calls carry idempotency keys where replay could duplicate work. Session events have a strictly increasing per-session content sequence. A subscription registers for live delivery and replays retained events after the supplied cursor without a replay-to-live loss window.

## 6. Provider Boundary

The built-in providers are DeepSeek, Zhipu GLM, OpenAI, Kimi, Claude, and Gemini. The static catalog currently exposes two models per provider: `deepseek-v4-flash` and `deepseek-v4-pro`; `glm-5.2` and `glm-5.3`; `gpt-5.5` and `gpt-5.6-sol`; `kimi-k2.7-code` and `kimi-k3`; `claude-sonnet-5` and `claude-opus-5`; and `gemini-3.5` and `gemini-3.6-flash`. One trusted adapter serves each provider, while each model route supplies its own vendor wire model. Kimi, Claude, and Gemini use their documented OpenAI-compatible chat-completions surfaces. Vendor request and streaming response shapes remain inside the provider adapter. Clients receive canonical messages, tool activity, usage, and redacted errors only.

The API key is read from the plaintext `secret_records` table in SQLite. Plaintext credentials never enter protocol responses, events, audit rows, or logs. An environment override is allowed only in an explicitly configured non-interactive execution mode. On macOS, the runtime may migrate credentials left by older releases from Keychain into SQLite.

## 7. Persistence

Rust is the only database owner. Avalonia, providers, and future extensions never open the database.

SQLite keeps separate durable concerns:

- immutable audit records for authority decisions and outcomes
- compactable, ordered session content as the rebuild source for durable message and lifecycle events
- query projections for projects, sessions, turns, tool calls, messages, approvals, and checkpoint manifests
- ephemeral live streaming deltas that are broadcast to connected clients but not retained
- disposable reconnect cursors
- durable turn admission and approval continuation
- scoped settings and plaintext-secret records

The Phase 1 schema preserves version 10 compatibility during the TypeScript-to-Rust migration, uses version 11 for plaintext provider secret records, and uses version 12 to backfill cumulative per-turn token usage projections. New migrations are append-only and transactional. JSON stores evolving payloads; identifiers, state, ordering, timestamps, scope, and foreign keys remain queryable columns.

## 8. Authority Model

Every tool call is validated, assigned a declared risk, evaluated by policy, and audited before execution. Read-only project inspection is allowed by the interactive default. Writes, process execution, network use outside the configured provider, secret access, destructive operations, and external paths require an explicit grant or user approval. Non-interactive execution fails closed without a matching profile grant.

Approval precedes execution. Approval requests and suspended continuations are durable and single-use. A restart may reconcile an operation with a durable idempotency record but must not blindly replay a provider call with unknown completion.

## 9. Reversibility and Recovery

Filesystem mutations capture pre-image checkpoints before changing disk. A turn-level manifest is the desktop undo unit and restores items in reverse operation order with post-image conflict checks. Process operations report the isolation actually enforced on the current platform; filtered environment or project-scoped working directory must never be described as network or OS sandboxing.

Startup marks non-recoverable in-memory turn execution interrupted, discovers admitted submissions and suspended approvals, and reconciles operation journal entries. Unknown completion remains visible and requires safe reconciliation.

## 10. Repository Layout

```text
apps/desktop-avalonia/    .NET 10 Avalonia desktop client
apps/desktop-qt/          retained Qt visual and interaction reference
contracts/                hand-written protocols and shared vectors
runtime/crates/core/      runtime core and embedded Rust SDK facade
runtime/crates/operations/ audited in-process machine operations
sdks/                     native language binding packaging surfaces
.agents/                  durable product and engineering knowledge
```

The old `typescript/` packages and retired `rust/` workspace were migration sources and are removed from the production tree. The complete `apps/desktop-qt/` source remains as a non-production parity fixture so Avalonia behavior can be checked against the original implementation. Language SDK directories may contain placeholder documentation before implementation starts, but they must not pretend to ship a working package until one exists.

## 11. Dependency Rules

- Avalonia depends only on .NET/Avalonia and the native SDK contract.
- Native binding functions call typed runtime services, never SQLite or provider wire types directly.
- Agent and provider modules call operations through the authorized dispatcher.
- Persistence does not depend on Avalonia, native bindings, or provider wire types.
- Operations do not depend on agent, provider, persistence projections, or client DTOs.
- No production TypeScript or Node.js process remains in Phase 1.

## 12. Deferred Scope

Phase 1 defers TypeScript and Python package implementation, CLI/TUI/Web clients, cross-process IPC, executable plugins, MCP servers, third-party provider adapters, PTY interaction, hosted execution, collaboration, telemetry, filesystem indexing/watchers, Git mutations and remote operations, other VCS-aware semantic operations, and cross-platform OS sandbox profiles. Adding executable third-party code requires a separate isolation design.
