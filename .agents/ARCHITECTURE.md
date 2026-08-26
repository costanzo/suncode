# SunCode Architecture

**Status:** Approved

**Date:** 2026-08-08

**Scope:** Phase 1 process topology, ownership boundaries, authority, persistence, protocols, and repository layout

## 1. Purpose

SunCode is a general-purpose coding agent. Phase 1 embeds one Rust agent SDK inside the .NET 10 Avalonia desktop process. Rust owns the complete agent: provider integration, agent behavior, policy, durable state, the SDK API, and machine-affecting operations.

The architecture favors explicit ownership, reviewable authority, and one authoritative agent. It does not claim that a process running as the user is an OS sandbox.

## 2. Process Topology

```text
.NET 10 Avalonia desktop
    | P/Invoke over C ABI
Embedded Rust SDK facade
    |
Rust SunCode agent core
    |- typed SDK services and subscriptions
    |- agent loop using the suncode-llm provider layer
    |- context, policy, approvals, and scheduling
    |- SQLite, settings, events, and credentials
    |- filesystem, search, process, and artifacts
    `- checkpoints and operation journal

Future TypeScript N-API and Python PyO3 bindings embed the same SDK.
```

There is no agent-to-core process boundary and no client-facing server. Operations are Rust modules called in-process after policy authorization. The old TypeScript runtime, core client, runtime server, JSON-RPC stdio core, and loopback HTTP/SSE adapter are not production architecture. Provider adapters still make outbound HTTPS requests to configured model providers.

## 3. Ownership Boundaries

### 3.1 Avalonia desktop

Avalonia XAML and C# view models own presentation, navigation, and transient interaction state. They consume agent DTOs and live events through the SDK facade. They never open SQLite, contact model providers, read project files directly, or invoke operation modules.

Phase 1 has no CLI, TUI, Web, mobile, or IDE client.

### 3.2 Rust agent

The Rust agent packages own:

- built-in model provider integrations and canonical provider messages
- context construction, turn scheduling, budgets, cancellation, and the agent loop
- tool registration, policy evaluation, durable approvals, and audit records
- typed SDK methods, normalized snapshots, and live subscription delivery
- SQLite initialization, transactions, projections, settings, and local event streams through the database package
- provider credentials and model catalog through SQLite-owned LLM provider/model records
- global, project, and session configuration through the unified `configuration` table
- project boundary checks and machine-affecting operations
- checkpoints, undo, managed artifacts, and operation reconciliation

Provider and orchestration modules cannot perform project operations directly. They construct typed operation requests which pass through policy and the agent operation dispatcher.

### 3.3 LLM providers

The `suncode-llm` package owns provider-neutral messages, tool schemas, completion results, provider errors, model metadata, model routing, and OpenAI-compatible HTTP/SSE behavior. It has no database, agent-core, SDK, desktop, or machine-operation dependency. Core loads the seeded or custom database catalog, converts its rows into LLM descriptors, and supplies credentials through the `ApiKeyResolver` trait at the agent boundary.

The registry accepts trusted in-process Rust implementations of `LlmProvider` with owned provider and model identifiers. Enterprise OpenAI-compatible gateways can use the built-in adapter with a custom endpoint; other trusted integrations can implement the trait. Rust hosts can extend the built-in registry during `AgentSdk` construction. This is library composition inside the host process, not dynamic plugin loading or an isolation boundary.

### 3.4 Tools

Tools are narrow Rust modules in the `suncode-tool` package inside the agent. The package owns the built-in model-facing tool catalog and canonical path validation, bounded reads/searches, read-only Git repository inspection, mutations, process execution, checkpoint payloads, artifacts, and operation journal records. It does not own provider semantics, conversation state, UI DTOs, or policy grants. Agent core converts the package's neutral tool definitions to provider request DTOs and remains responsible for policy, approval, orchestration, and conversation-only tool handling.

This internal boundary is for auditability and testing. It is not a child-process security boundary.

## 4. Agent Lifecycle

One agent instance exists per data directory. Its host process acquires a single-instance lock, opens and initializes the current SQLite schema, reconciles interrupted local work, and retains the SDK handle until shutdown. It does not bind a client-facing socket, create an agent credential, or publish an endpoint discovery record.

The Avalonia client embeds and opens the agent, fetches a session snapshot, then receives live events through a direct subscription. A lagged subscription or reconnect reloads the normalized snapshot and never treats client cache as authoritative. A second process cannot attach to an active agent; replacement IPC requires a new architectural decision.

## 5. SDK Contract

Phase 1 keeps the embedded SDK contract in `contracts/agent-sdk/`. C# calls named methods through the stable C ABI using P/Invoke. Future TypeScript and Python packages wrap the same Rust facade through native bindings. DTOs are hand-implemented in Rust and each host language and verified by focused contract tests. Contract generation is prohibited.

Mutating calls carry idempotency keys where replay could duplicate work. Session snapshots read normalized tables directly. Subscriptions deliver live in-memory events only; if a subscriber lags, it receives `resync.required` and reloads a snapshot.

## 6. Provider Boundary

The seeded providers are DeepSeek, Zhipu GLM, OpenAI, Kimi, Claude, and Gemini. The seeded database catalog currently exposes two models per provider: `deepseek-v4-flash` and `deepseek-v4-pro`; `glm-5.2` and `glm-5.3`; `gpt-5.5` and `gpt-5.6-sol`; `kimi-k2.7-code` and `kimi-k3`; `claude-sonnet-5` and `claude-opus-5`; and `gemini-3.5` and `gemini-3.6-flash`. Users may add provider and model rows for custom OpenAI-compatible gateways. One trusted adapter serves each provider, while each model route supplies its own vendor wire model. Kimi, Claude, and Gemini use their documented OpenAI-compatible chat-completions surfaces. Vendor request and streaming response shapes remain inside `suncode-llm`. Clients receive canonical messages, tool activity, usage, and redacted errors only.

The API key is read from the plaintext `llm_model_provider.api_key` column in SQLite. Provider endpoints and required `adapter_type` values are read from `llm_model_provider`; model request codes, context lengths, auto-compaction thresholds, output limits, capability flags, and enabled/order state are read from `llm_model`. A custom provider must select an adapter implemented by `suncode-llm`; the current persisted adapter is `openai` for OpenAI-compatible endpoints. Plaintext credentials never enter protocol responses, events, audit rows, or logs. An environment override is allowed only in an explicitly configured non-interactive execution mode.

## 7. Persistence

Rust is the only database owner. Avalonia, providers, and future extensions never open the database.

SQLite keeps separate durable concerns:

- immutable audit records for authority decisions and outcomes
- normalized rows in `project`, `session`, turn, model-call, tool-use, message, approval, and checkpoint tables
- ephemeral live streaming deltas that are broadcast to connected clients but not retained
- durable turn admission and approval continuation
- scoped settings and plaintext provider-key records

The Phase 1 database has one current 15-table schema and no schema versions or general migration runner. The `suncode-database` package owns backend resources, with SQLite scripts and file setup under `suncode-database::sqlite`; the `suncode-data` package owns Diesel connections, ORM declarations, persistence DTOs, and operations. Initialization applies the database package's ordered schema/data manifests in one transaction. Initialization may add the current empty `project_dependency` table to an otherwise-current 13-table database; unexpected or structurally incompatible databases remain rejected without conversion. `project` is the project identity table, and `project_dependency` stores its registered read-only source roots. `session` is the conversation root; `session_turn` is the single turn/submission/recovery record, `session_turn_todo` is the authoritative current todo projection keyed by turn and ordinal, `session_call` stores each LLM request plus independently nullable provider HTTP request and response-object identifiers, `session_tool_use` exclusively stores tool requests/results and state, and `session_message` stores user, assistant, and thinking messages. Provider context derives transient tool-role messages from succeeded tool-use rows. `configuration` owns global/project/session key-value overlays, including global logging policy. Human-readable messages are ordered by timestamp. Agent event payloads are not duplicated in SQLite; SDK snapshots read normalized rows and live subscribers resync after lag.

## 8. Authority Model

Every tool call is validated, assigned a declared risk, evaluated by policy, and audited before execution. Read-only project inspection is allowed by the interactive default. Writes, process execution, network use outside the configured provider, secret access, destructive operations, and external paths require an explicit grant or user approval. Non-interactive execution fails closed without a matching profile grant.

Approval precedes execution. Approval requests and suspended continuations are durable and single-use. A restart may reconcile an operation with a durable idempotency record but must not blindly replay a provider call with unknown completion.

## 9. Reversibility and Recovery

Filesystem mutations capture pre-image checkpoints before changing disk. A turn-level manifest is the desktop undo unit and restores items in reverse operation order with post-image conflict checks. Process operations report the isolation actually enforced on the current platform; filtered environment or project-scoped working directory must never be described as network or OS sandboxing.

Process execution has two explicit semantics: structured program-plus-argv execution never invokes a shell implicitly, while shell-script execution selects the documented host dialect (Windows PowerShell on Windows and POSIX `sh` on macOS/Linux). Both pass through the same policy and audited dispatcher. Shell syntax is platform-specific and is never translated between dialects.

Registered project dependencies extend only read authority. Rust stores and canonicalizes their roots, exposes stable opaque IDs instead of absolute paths, and routes model `dependency:<id>/...` aliases only through bounded read, glob, and grep operations. They do not expand write, process, Git, checkpoint, undo, or project authority.

Startup marks non-recoverable in-memory turn execution interrupted, discovers admitted submissions and suspended approvals, and reconciles operation journal entries. Unknown completion remains visible and requires safe reconciliation.

## 10. Repository Layout

```text
apps/desktop-avalonia/    .NET 10 Avalonia desktop client
contracts/                hand-written protocols and contract documentation
agent/crates/core/      agent core and embedded Rust SDK facade
agent/crates/common/    shared Rust business errors and cross-crate contracts
agent/crates/database/  backend-specific SQL resources and database setup
agent/crates/data/      Diesel ORM, persistence DTOs, and data operations
agent/crates/llm/       provider-neutral LLM contracts, catalog, registry, and adapters
agent/crates/tools/      `suncode-tool` package for built-in definitions and audited in-process machine operations
sdks/                     native language binding packaging surfaces
.agents/                  durable product and engineering knowledge
```

The old `typescript/` packages and retired `rust/` workspace were migration sources and are removed from the production tree. Language SDK directories may contain placeholder documentation before implementation starts, but they must not pretend to ship a working package until one exists.

## 11. Dependency Rules

- Avalonia depends only on .NET/Avalonia and the native SDK contract.
- Native binding functions call typed agent services, never SQLite or provider wire types directly.
- Agent and provider modules call operations through the authorized dispatcher.
- The database crate does not depend on the agent core, Avalonia, native bindings, operations, or provider wire types.
- Cross-crate business failures use `suncode-common::BusinessError`; lower-level Diesel, HTTP, Git, and OS errors are converted before crossing their owning crate boundary.
- The agent core depends on the database crate for durable state and persistence DTOs.
- The LLM crate does not depend on the database, agent core, SDK, desktop, or tools crates.
- The agent core supplies credentials and tool schemas to the LLM crate through provider-neutral interfaces and request DTOs.
- Tools do not depend on agent, provider, persistence projections, or client DTOs.
- No production TypeScript or Node.js process remains in Phase 1.

## 12. Deferred Scope

Phase 1 defers TypeScript and Python package implementation, CLI/TUI/Web clients, cross-process IPC, executable or dynamically loaded provider plugins, MCP servers, persisted client configuration of custom providers, PTY interaction, hosted execution, collaboration, telemetry, filesystem indexing/watchers, Git mutations and remote operations, other VCS-aware semantic operations, and cross-platform OS sandbox profiles. Adding executable third-party code requires a separate isolation design.
