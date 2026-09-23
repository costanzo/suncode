# SunCode Architecture

**Status:** Approved

**Date:** 2026-08-08

**Scope:** Embedded agent process topology, approved native clients, ownership boundaries, authority, persistence, protocols, and repository layout

## 1. Purpose

SunCode is a general-purpose coding agent. Phase 1 embeds one Rust agent SDK inside the .NET 10 Avalonia desktop process. A native Rust CLI is approved as the next production client and will embed the async SDK facade directly. Rust owns the complete agent: provider integration, agent behavior, policy, durable state, the SDK API, and machine-affecting operations.

The architecture favors explicit ownership, reviewable authority, and one authoritative agent. It does not claim that a process running as the user is an OS sandbox.

## 2. Process Topology

```text
.NET 10 Avalonia desktop
    | ProjectReference to the managed C# SDK
`sdks/csharp` typed managed binding
    | P/Invoke over C ABI
`sdks/c` native binding
    |
`sdks/rust` async-first typed facade plus explicit blocking host adapter
    |
Rust SunCode agent core
    |- agent loop using the suncode-llm provider layer
    |- context, policy, approvals, and scheduling
    |- SQLite, settings, events, and credentials
    |- filesystem, search, process, and artifacts
    |- project-scoped language-server protocol clients
    |- project-scoped Browser Use lifecycle and audited browser tools
    |- first-party Computer Use lifecycle and audited desktop input/capture
    `- checkpoints and operation journal

Bundled Browser Use worker (lazy, one per active browser project)
    |- fixed Node.js runtime
    |- fixed Playwright package
    `- fixed Chromium build and project-isolated persistent profile

Future TypeScript N-API and Python PyO3 bindings embed the same SDK.

Approved Rust CLI (administration and one-shot new/resumed turns implemented)
`apps/cli`
    | direct async Rust calls
`sdks/rust::AsyncAgentSdk`
    `- same Rust SunCode agent core and ownership boundaries
```

There is no agent-to-core process boundary and no client-facing server. Operations are Rust modules called in-process after policy authorization. The old TypeScript runtime, core client, runtime server, JSON-RPC stdio core, and loopback HTTP/SSE adapter are not production architecture. Provider adapters still make outbound HTTPS requests to configured model providers. Computer Use remains in-process and Rust-owned through a pinned Enigo backend. Browser Use is the narrow exception to the otherwise Rust-only process topology: Rust may launch the exact bundled Node.js worker over a private framed stdio protocol, but that worker is a browser driver rather than an agent, provider, database owner, or extension host.

## 3. Ownership Boundaries

### 3.1 Product clients

Avalonia XAML and C# view models own presentation, navigation, and transient interaction state. They consume agent DTOs and live events through the SDK facade. They never open SQLite, contact model providers, read project files directly, or invoke operation modules.

The CLI is a line-oriented native Rust client under `apps/cli`. It owns administrative, session-administration, and one-shot-turn argument parsing, text/JSONL adaptation, typed event rendering, credential terminal input, interrupt handling, exit status, and explicit SDK lifecycle. `run` opens a project and creates a primary session; `session resume` reopens an existing primary session and adds one turn using durable history. Both establish atomic watch before submission, consume the typed event stream, drain queued terminal events, and request SDK cancellation on interrupt. Resume queries typed pending approval/question state and fails closed rather than bypassing it. Session list/archive call the typed SDK facade and do not interpret or mutate persistence directly. The CLI calls `AsyncAgentSdk` with a typed host capability ceiling that disables Browser and Computer Use without changing shared settings. It never calls the C ABI, opens SQLite, contacts providers, invokes operation modules, or implements policy. Interactive chat and interactive approval/question prompts remain unimplemented. Its normative design is `contracts/cli.md`.

The Avalonia desktop remains the implemented Phase 1 reference client. TUI, Web, mobile, and IDE clients remain deferred.

### 3.2 Rust agent

The Rust agent packages own:

- built-in model provider integrations and canonical provider messages
- context construction, turn scheduling, budgets, cancellation, and the agent loop
- tool registration, policy evaluation, and durable approvals
- harness services used by the typed SDK facade
- SQLite initialization, transactions, projections, settings, and local event streams through the database package
- provider credentials and model catalog through SQLite-owned LLM provider/model records
- global, project, and session configuration through the unified `configuration` table
- project boundary checks and machine-affecting operations
- project-scoped language-server definitions, lifecycle, document synchronization, and bounded semantic queries
- bundled-browser validation, project-scoped Browser Use lifecycle, control handoff, policy enforcement, and artifact promotion
- first-party Computer Use enablement, display-frame lifetime, approval, emergency stop, and transient screenshot handling
- checkpoints, undo, managed artifacts, and operation reconciliation

Provider and orchestration modules cannot perform project operations directly. They construct typed operation requests which pass through policy and the agent operation dispatcher.

### 3.3 LLM providers

The `suncode-llm` package owns provider-neutral messages, tool schemas, completion results, provider errors, model metadata, model routing, and OpenAI-compatible HTTP/SSE behavior. It has no database, agent-core, SDK, desktop, or machine-operation dependency. The Rust SDK facade loads the seeded or custom database catalog, converts its rows into LLM descriptors, and supplies credentials through a private SQLite-backed implementation of the `ApiKeyResolver` trait at the agent composition boundary. There is no separate credential store alongside SQLite.

The registry accepts trusted in-process Rust implementations of `LlmProvider` with owned provider and model identifiers. Enterprise OpenAI-compatible gateways can use the built-in adapter with a custom endpoint; other trusted integrations can implement the trait. Rust hosts can extend the built-in registry during `AgentSdk` construction. This is library composition inside the host process, not dynamic plugin loading or an isolation boundary.

### 3.4 Tools

Tools are narrow Rust modules in the `suncode-tool` package inside the agent. The package owns the built-in model-facing tool catalog and canonical path validation, bounded reads/searches, read-only Git repository inspection, mutations, process execution, checkpoint payloads, artifacts, and operation journal records. It does not own provider semantics, conversation state, UI DTOs, or policy grants. Agent core converts the package's neutral tool definitions to provider request DTOs and remains responsible for policy, approval, orchestration, and conversation-only tool handling.

This internal boundary is for auditability and testing. It is not a child-process security boundary.

### 3.5 Browser Use worker

The Browser Use worker is first-party, version-locked JavaScript running on the bundled Node.js executable. It owns Playwright objects, Chromium pages, accessibility snapshots, semantic locator resolution, and the private worker protocol. It receives dedicated profile, temporary, and staging directories plus a filtered environment. It cannot open SQLite, call model providers, choose policy, issue approvals, inspect project files, load plugins or browser extensions, or execute arbitrary model-authored JavaScript.

Rust owns the worker and Chromium process tree, validates the runtime handshake against the packaged manifest, serializes project operations, and fails closed on a version, target, integrity, protocol, path, or generation mismatch. This containment is an auditable ownership boundary, not an OS sandbox.

## 4. Agent Lifecycle

One agent instance exists per data directory. Its host process acquires a single-instance lock, opens and initializes the current SQLite schema, reconciles interrupted local work, and retains the SDK handle until shutdown. Explicit shutdown consumes the Rust facade, rejects new turn and continuation admission, cancels active turns, clears queued input, releases Computer Use input, drains Browser workers, MCP connections, and language-server clients, waits a bounded five seconds for active turns, closes event subscribers, and finally releases SQLite ownership and the data-directory lock. The blocking adapter keeps its executor alive while awaiting the same cleanup; native close invokes that adapter. Ordinary Rust drop remains a non-graceful fallback. The agent does not bind a client-facing socket, create an agent credential, or publish an endpoint discovery record.

The Rust core contains one immutable catalog of six built-in specialist definitions. A primary session may invoke the core-owned `delegate_agent` conversation tool, which creates one linked child session and a durable invocation correlation. Child sessions inherit project, model, and reasoning effort, advertise only their role allowlist, revalidate every tool call, cannot use MCP or `question`, and cannot delegate again. They are inspection and authority surfaces for SDK clients, not independent user conversations. Parent cancellation propagates to an active child; pending child approvals resume through the ordinary durable approval path.

The Avalonia client embeds and opens the agent through the explicit runtime-owning blocking adapter, atomically obtains a session snapshot plus dormant stream, applies the snapshot and auxiliary presentation state, then explicitly starts callback delivery. Native Rust hosts instead use the runtime-free `AsyncAgentSdk` on their own Tokio executor. Core routes non-exhaustive typed events through bounded session-scoped streams; the Rust SDK implements the standard `Stream` and `FusedStream` contracts in addition to direct receive methods, while the C binding owns dormant handle activation, callback-thread, and JSON-envelope adaptation for Avalonia. Normal Rust stream closure returns `None`; lag returns one typed error and terminates so the host can establish a fresh atomic watch. Rust `watch_session` registers a stream and reads its snapshot under the same per-session in-memory gate used by event projection and publication, so a notification cannot fall between the two views; clients still apply events idempotently because some normalized lifecycle writes precede notification projection. Stale or failed desktop loads dispose their dormant handle before callback activation. A lagged subscription or reconnect repeats the atomic watch flow and never treats client cache as authoritative. A second process cannot attach to an active agent. `ADR-20260923-desktop-notification-activation-ipc` adds a separate desktop-only local activation channel for notification clicks and secondary-process handoff. It uses a current-user Named Pipe on Windows or a user-only Unix Domain Socket on macOS/Linux, carries bounded navigation intent only, and never proxies the SDK.

## 5. SDK Contract

Phase 1 keeps the embedded SDK contract in `contracts/agent-sdk/`. C# calls named methods through the stable C ABI using P/Invoke. The approved CLI calls the async Rust facade directly and adapts typed DTOs/events to the terminal contract in `contracts/cli.md`. Future TypeScript and Python packages wrap the same Rust facade through native bindings. DTOs are hand-implemented in Rust and each host language and verified by focused contract tests. Contract generation is prohibited.

Mutating calls carry idempotency keys where replay could duplicate work. Session snapshots read normalized tables directly. Subscriptions deliver live in-memory events only; if a subscriber lags, it receives `resync.required` and reloads a snapshot.

## 6. Provider Boundary

The seeded providers are DeepSeek, Zhipu GLM, OpenAI, Kimi, Claude, and Gemini. The seeded database catalog currently exposes two models per provider: `deepseek-v4-flash` and `deepseek-v4-pro`; `glm-5.2` and `glm-5.3`; `gpt-5.5` and `gpt-5.6-sol`; `kimi-k2.7-code` and `kimi-k3`; `claude-sonnet-5` and `claude-opus-5`; and `gemini-3.5` and `gemini-3.6-flash`. Users may add provider and model rows for custom OpenAI-compatible gateways. One trusted adapter serves each provider, while each model route supplies its own vendor wire model. Claude's built-in route uses Anthropic Messages so it can advertise native client toolsets; custom Claude-compatible gateways remain unchanged unless explicitly configured for a supported adapter. Other seeded compatible routes use their documented chat-completions surfaces. Vendor request and streaming response shapes remain inside `suncode-llm`. Clients receive canonical messages, tool activity, usage, and redacted errors only.

The API key is read exclusively from the plaintext `llm_model_provider.api_key` column in SQLite. Provider endpoints and required `adapter_type` values are read from `llm_model_provider`; model request codes, context lengths, auto-compaction thresholds, output limits, capability flags, and enabled/order state are read from `llm_model`. A custom provider must select an adapter implemented by `suncode-llm`; the current persisted adapter is `openai` for OpenAI-compatible endpoints. Plaintext credentials never enter protocol responses, events, or logs. Provider API-key environment variables are not read in either interactive or non-interactive mode. Global `verify_https_certificates` defaults to `true` and controls server certificate-chain and hostname verification for built-in provider and WebFetch HTTPS requests. Disabling it is an explicit insecure mode equivalent to `curl -k`; it does not weaken other authority or URL controls.

Global proxy configuration is stored in the unified `configuration` table and applies to every SunCode-owned HTTP client: built-in and persisted OpenAI-compatible providers, WebFetch, and remote Streamable HTTP MCP connections. Modes are no proxy, supported system proxy discovery, and custom HTTP/HTTPS proxy with Basic credentials and bypass rules. PAC, SOCKS, local MCP child-process traffic, and trusted third-party provider internals are outside this guarantee. `proxy_password` follows the current plaintext SQLite policy but is removed from settings read projections, which expose only `proxy_password_configured`.

Bundled Chromium receives the effective no-proxy, system-proxy, or custom-proxy configuration through the Rust-owned Browser Use launch boundary. Chromium retains its normal certificate verification and never inherits `verify_https_certificates=false`; SunCode does not bypass browser HTTPS interstitials. Custom trust-file support for Chromium is outside the first Browser Use delivery.

## 7. Persistence

Rust is the only database owner. Avalonia, providers, and future extensions never open the database.

SQLite keeps separate durable concerns:

- normalized authority and operation outcomes in the owning session/tool rows
- normalized rows in `project`, `session`, turn, model-call, tool-use, message, approval, and checkpoint tables
- durable `subagent_invocation` rows correlating parent turn/tool calls with child sessions and their terminal or approval-suspended outcome
- ephemeral live streaming deltas that are broadcast to connected clients but not retained
- durable turn admission and approval continuation
- scoped settings and plaintext provider-key records

The Phase 1 database has one current 18-table schema and no schema versions or general migration runner. The `suncode-database` package owns backend resources, with SQLite scripts and file setup under `suncode-database::sqlite`; the `suncode-data` package owns Diesel connections, ORM declarations, persistence DTOs, and operations. Initialization applies the database package's ordered schema/data manifests in one transaction. Narrow additive compatibility steps cover the current `language_server` table and the earlier MCP and built-in-agent additions; unexpected or structurally incompatible databases remain rejected without conversion. `project` is the project identity table, `project_dependency` stores registered read-only source roots, and `language_server` stores global desired local-stdio definitions while project runtime state remains memory-only. `session` is the conversation root; `subagent_invocation` correlates primary turns with delegated child sessions; `session_turn` is the single turn/submission/recovery record; `session_turn_todo` is the authoritative current todo projection keyed by turn and ordinal; `session_call` stores each LLM request plus independently nullable provider HTTP request and response-object identifiers; `session_tool_use` exclusively stores tool requests/results and state; and `session_message` stores user, assistant, and thinking messages. Provider context derives transient tool-role messages from succeeded tool-use rows. `configuration` owns global/project/session key-value overlays, including global logging policy. Human-readable messages are ordered by timestamp. Agent event payloads are not duplicated in SQLite; SDK snapshots read normalized rows and live subscribers resync after lag.

## 8. Authority Model

Every tool call is validated, assigned a declared risk, evaluated by policy, and audited before execution. Read-only project inspection is allowed by the interactive default. Writes, process execution, network use outside the configured provider, secret access, destructive operations, and external paths require an explicit grant or user approval. Non-interactive execution fails closed without a matching profile grant.

Built-in agent tool lists are capability ceilings, not authority grants. Child calls still pass the same validation, project scope, policy, approval, audit, checkpoint, and operation dispatcher as primary calls. Unknown or disallowed tools fail before policy evaluation.

Approval precedes execution. Approval requests and suspended continuations are durable and single-use. A restart may reconcile an operation with a durable idempotency record but must not blindly replay a provider call with unknown completion.

Browser capability enablement is not browser authority. In the initial implementation, every Browser Use tool call requires interactive approval and is not bypassed by general Full Control; non-interactive calls fail closed. Page content is untrusted and cannot authorize an operation. Fine-grained origin/action classification and preflight descriptors remain delivery work before lower-risk observations can use narrower policy. Browser changes to external systems and browser profiles are not covered by filesystem undo.

Computer capability enablement is not desktop authority. Observation-only Computer Use actions may follow the interactive default, while any input-producing batch requires one interactive approval and is not bypassed by general Full Control. Non-interactive Computer Use fails closed. On-screen content is untrusted and cannot authorize an operation. Emergency stop disables the capability, cooperatively cancels active work, releases held input, and requires explicit re-enablement. Screenshots are transient provider context; external application changes are not covered by filesystem undo.

## 9. Reversibility and Recovery

Filesystem mutations capture pre-image checkpoints before changing disk. A turn-level manifest is the desktop undo unit and restores items in reverse operation order with post-image conflict checks. Process operations report the isolation actually enforced on the current platform; filtered environment or project-scoped working directory must never be described as network or OS sandboxing.

Delegated child mutations currently produce child-session checkpoint manifests. The parent turn's undo manifest does not aggregate them, and clients must not claim that parent-turn undo covers delegated changes.

Process execution has two explicit semantics: structured program-plus-argv execution never invokes a shell implicitly, while shell-script execution selects the documented host dialect (Windows PowerShell on Windows and POSIX `sh` on macOS/Linux). Both pass through the same policy and audited dispatcher. Shell syntax is platform-specific and is never translated between dialects.

Registered project dependencies extend only read authority. Rust stores and canonicalizes their roots, exposes stable opaque IDs instead of absolute paths, and routes model `dependency:<id>/...` aliases only through bounded read, glob, and grep operations. They do not expand write, process, Git, checkpoint, undo, or project authority.

Startup marks non-recoverable in-memory turn execution interrupted, discovers admitted submissions and suspended approvals, and reconciles operation journal entries. Unknown completion remains visible and requires safe reconciliation.

## 10. Repository Layout

```text
apps/desktop-avalonia/    .NET 10 Avalonia desktop client
apps/cli/                 native Rust CLI administration and one-shot turn client
contracts/                hand-written protocols and contract documentation
agent/crates/core/      agent harness and core services
agent/crates/config/    Rust-owned bootstrap configuration crate
agent/crates/common/    shared Rust business errors and cross-crate contracts
agent/crates/database/  backend-specific SQL resources and database setup
agent/crates/data/      Diesel ORM, persistence DTOs, and data operations
agent/crates/llm/       provider-neutral LLM contracts, catalog, registry, and adapters
agent/crates/tools/      `suncode-tool` package for built-in definitions and audited in-process machine operations
agent/crates/mcp/        bounded MCP client transports, discovery, invocation, and result normalization
agent/crates/lsp/        bounded local-stdio LSP framing, lifecycle, document sync, and semantic requests
agent/crates/browser/    bounded Browser Use worker protocol, runtime validation, and process lifecycle
browser-runtime/        fixed JavaScript worker plus target-specific Node.js, Playwright, and Chromium packaging inputs
sdks/rust/                typed Rust SDK facade over the agent harness
    sdks/c/                   stable C ABI/native library
    sdks/csharp/              typed managed SDK and native integration for Avalonia
sdks/                     native language binding packaging surfaces
.agents/                  durable product and engineering knowledge
```

The old `typescript/` packages and retired `rust/` workspace were migration sources and are removed from the production tree. Language SDK directories may contain placeholder documentation before implementation starts, but they must not pretend to ship a working package until one exists.

## 11. Dependency Rules

- Avalonia depends only on .NET/Avalonia and the native SDK contract.
- The CLI depends on `sdks/rust` plus terminal/argument/serialization libraries; it does not depend on C, C#, SQLite/data, provider, or operation crates.
- Native binding functions call typed agent services, never SQLite or provider wire types directly.
- Agent and provider modules call operations through the authorized dispatcher.
- The database crate does not depend on the agent core, Avalonia, native bindings, operations, or provider wire types.
- Cross-crate business failures use `suncode-common::BusinessError`; lower-level Diesel, HTTP, Git, and OS errors are converted before crossing their owning crate boundary.
- The agent core depends on the database crate for durable state and persistence DTOs.
- The LLM crate does not depend on the database, agent core, SDK, desktop, or tools crates.
- The Rust SDK composition supplies credentials to the LLM crate through its provider-neutral resolver interface; the agent core supplies tool schemas through provider-neutral request DTOs.
- Tools do not depend on agent, provider, persistence projections, or client DTOs.
- No production TypeScript agent path remains in Phase 1. The only production Node.js process is the fixed bundled Playwright Browser Use worker; no other package may depend on it or use it as a general execution or extension host.

## 12. Deferred Scope

The CLI still defers interactive `chat`, interactive approval/question continuation, TUI, PTY, daemon/attach, Browser Use, Computer Use, shell-parent mutation, auto-update, and hosted modes as specified in `contracts/cli.md`. The broader product still defers TypeScript and Python package implementation, TUI/Web/mobile/IDE clients, general client-facing cross-process IPC and live-agent attach, executable or dynamically loaded provider plugins, MCP prompts/resources/OAuth, client creation/removal of custom provider and model catalog entries, hosted execution, collaboration, telemetry, filesystem indexing/watchers, Git mutations and remote operations, other VCS-aware semantic operations, arbitrary browser script execution, external browser profiles, non-Chromium browsers, and cross-platform OS sandbox profiles. The narrow desktop-only notification activation IPC is implemented under `requirements/2026-09-23-session-attention-notifications/`; it carries navigation intent and never proxies the agent SDK. Installed notification display and click conformance remains release verification on macOS, Windows, GNOME, and KDE. Settings may manage tools-only MCP servers over local stdio and remote Streamable HTTP through the Rust-owned SDK. Local MCP, LSP, Browser Use worker, and Chromium processes are lifecycle-contained and policy-mediated but are not OS-sandboxed; the client and approval surfaces state their authority and undo limitations explicitly.
