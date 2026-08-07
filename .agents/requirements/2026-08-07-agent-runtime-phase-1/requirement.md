# Requirement

## Background

Suncode clients are presentation shells. The TypeScript runtime is the product's agent brain and the only layer that talks to model providers, constructs context, runs agent turns, persists sessions, and translates model intent into typed core operations. It runs on the user's machine.

Claude Code, Codex, and OpenCode provide useful reference patterns: a conversation-centered loop, provider/model selection, composable instructions, tool calls, approval gates, MCP integrations, skills, plugins, streaming events, and resumable sessions. Suncode should combine these ideas behind one coherent contract rather than expose product-specific internals.

## Goals

- Run one deterministic, cancellable agent loop per session, structured as a two-level turn and tool-call state machine.
- Run unattended under a declared policy profile so the agent is usable in scripts and CI.
- Support multiple model providers through a canonical provider-neutral interface.
- Build bounded, inspectable context from session history, project metadata, user input, skills, plugins, and tool results.
- Make filesystem, search, process, shell, write, and artifact tools go through Rust.
- Provide first-class skills, plugins, and MCP with explicit discovery, versioning, capability declaration, and trust policy.
- Enforce approvals and authorization as a layered policy flow, with the core validating every machine-affecting operation.
- Persist an immutable audit log, compactable session content, and disposable client sync state in runtime-owned SQLite.
- Make the agent's filesystem changes reversible through checkpoints.
- Stream ordered events to every authorized client and recover after runtime/core/provider failures.
- Expose model, tool, permission, session, settings, diagnostics, and extension state through a client-safe API.
- Make cost, token, latency, retry, and failure behavior observable and bounded.

## Non-goals

- Implementing this runtime in the current delivery.
- Implementing a model provider, tool operation, SQLite repository, encryption, or MCP server now.
- Allowing model providers, plugins, skills, or MCP servers to bypass the runtime policy engine or call Rust directly.
- Supporting parallel subagents inside one session in Phase 1.
- Executing a turn's tool calls concurrently in Phase 1, though the state model must express it.
- Cloud hosting, tenancy, remote identity, and hosted sandbox infrastructure.
- Generating protocol types or a client SDK from contract documents.
- Letting a committed project file grant authority on its own.
- Exposing hidden model chain-of-thought or requiring any provider to return it.

## Reference synthesis

| Reference pattern | Suncode decision |
| --- | --- |
| Codex-style session work with resumable activity | Adopt durable sessions, ordered events, snapshots, replay, and explicit turn lifecycle |
| Claude Code-style composable instructions, skills, MCP, and approval prompts | Adopt layered instruction sources, discoverable skills, MCP tools, and causal approval UX |
| Claude Code and Codex non-interactive modes | Adopt policy profiles and a single-process run mode as first-class requirements |
| OpenCode-style provider/model breadth and configurable tools | Adopt provider adapters, a capability-aware model catalog, and policy-filtered tool registries |
| Product-specific config or private implementation details | Do not copy; use Suncode's layered configuration and runtime-owned state |

## Core concepts

### Runtime

One Node.js process per OS user. It supervises one Rust child, one SQLite database writer, provider clients, extension hosts, and authorized client connections. It can also be hosted in-process by the CLI for a single non-interactive session.

### Session and turn

A session is durable conversation state scoped to one project. A turn is one user submission and its agent-loop execution. A turn has stable identifiers, an idempotency key, a lifecycle state, a cancellation token, budgets, and correlation IDs. Each tool call within a turn carries its own identifier and lifecycle state.

### Model provider and model

A provider adapter translates a canonical request into a vendor SDK/API call and translates responses, streaming deltas, usage, rate limits, and errors back into canonical types. A model is selected by stable Suncode identity plus provider-specific metadata and negotiated capabilities.

### Tool

A typed capability the model may request. Runtime tools include read/search/context operations and Rust-backed machine operations. Every tool has a schema, risk class, execution target, approval policy, timeout, output limit, and redaction policy.

### Skill

A declarative, versioned instruction and resource bundle that can be selected for a session or project. Skills shape context and available workflows; they do not gain ambient authority merely by being loaded.

### Plugin

A versioned extension package that adds runtime adapters, skills, provider integrations, commands, or UI metadata through a stable manifest. Plugins execute in a restricted host boundary and are disabled unless their trust and compatibility checks pass.

### MCP server

An external tool/resource/prompt provider connected through an MCP client owned by the runtime. MCP capabilities are normalized into the Suncode registry and pass through the same schema validation, policy, approval, timeout, output, and audit pipeline as built-in tools.

## Requirements

### Runtime lifecycle and isolation

- Use Node.js exclusively; Bun is prohibited.
- Start only after configuration, database migrations, Rust initialization, and built-in provider discovery reach a safe state. Project-declared third-party extensions may be discovered and validated as metadata, but must not be started or activated before a future trust-boundary delivery and explicit user authorization.
- Supervise one Rust child process and keep the Rust JSON-RPC transport private to the runtime.
- Enforce one database writer per runtime instance and a per-user single-instance policy.
- Track active clients, turns, core operations, provider calls, extension calls, and background jobs before allowing idle shutdown.
- Isolate sessions by project and authorization scope.
- Never accept a client request that names an unauthorized provider, project, session, tool, or extension.
- Structure the runtime as a library with a thin server wrapper so it can be hosted in-process for single-process and CI use. No orchestration logic may live in request handlers.
- Publish the runtime build version in the discovery file and refuse client attachment on incompatible version skew, supporting a drain-and-exit handoff rather than a forced kill.

### Non-interactive execution

- Support a single-process mode that runs one session without a resident runtime, against a private or explicitly named database, and exits.
- Support policy profiles: named, user-declared sets of pre-authorized capabilities and scopes as defined in `ARCHITECTURE.md` section 9.4.
- Fail an unauthorized operation immediately with a typed error in non-interactive mode instead of blocking on an approval no one can answer.
- Record profile-sourced authorization in the audit stream, marked as profile-originated rather than user-originated.
- Provide no mode that disables policy enforcement or audit recording.
- Emit machine-readable output suitable for scripts, and a non-zero exit status distinguishing agent failure, budget exhaustion, and authorization denial.

### Provider abstraction

- Define canonical operations for model discovery, chat/response generation, streaming, tool-call requests, cancellation, usage, and health.
- Support provider-specific authentication without exposing credentials to clients or storing plaintext secrets.
- Keep provider adapters stateless per request where possible; durable state belongs to the runtime session.
- Normalize roles, content parts, tool calls, refusal, finish reason, usage, rate limits, retry hints, and typed errors.
- Declare capabilities such as streaming, structured output, vision, tool use, reasoning summary, context limit, and cancellation rather than infer them from model names.
- Permit per-project defaults and per-turn overrides only from models advertised by the runtime.
- Route transient failures through bounded retries with jitter and idempotency awareness; never blindly replay a non-idempotent tool call.
- Record provider, model, request correlation ID, latency, token usage, and redacted error class in runtime telemetry.
- Make provider selection policy-aware: unavailable, disallowed, unconfigured, or over-budget models must fail before a turn starts.

### Agent loop

- Implement a two-level state machine as defined in `ARCHITECTURE.md` section 8.1. The turn level is `admitted -> queued -> preparing -> calling_model -> resolving_calls | compacting -> calling_model -> completed|failed|cancelled|interrupted`. Each tool call requested by the model gets an independently identified child machine: `requested -> validating -> policy_check -> denied | awaiting_approval | authorized -> executing -> succeeded|failed|timed_out|unknown_completion -> reconciling`.
- Treat several tool calls in one assistant message as the normal case, not an error. The turn leaves `resolving_calls` only when every child call is terminal.
- Place approval strictly before execution. No path executes a call and then requests authorization for it.
- Execute child calls sequentially in Phase 1 by scheduling policy. Do not encode that limit into the state model, event payloads, or identifiers.
- Persist each transition at both levels as session content and expose it to clients in order, with the tool-call identifier distinct from the turn identifier.
- On each iteration, supply the model with the canonical system instructions, scoped session history, selected skills, available tool schemas, user input, and bounded tool results.
- Validate model output before acting. Text is emitted as an assistant message; tool calls enter the tool policy pipeline; malformed output becomes a typed turn failure.
- Enforce maximum iterations, wall-clock duration, model-token budget, tool-call budget, and output-size budget.
- Support cooperative cancellation at provider, tool, MCP, and core-operation boundaries.
- Ensure exactly one sequential turn mutates a session at a time; reject or queue conflicting submissions explicitly.
- Preserve partial messages and completed tool results when a turn is interrupted or a provider fails.
- Never treat model text as an executable command or authorization decision.
- Allow future parallel subagents by keeping turn, tool, event, and correlation identifiers independent, while disabling that mode in Phase 1.

### Context construction and compaction

- Assemble context through the named, inspectable layers defined in `ARCHITECTURE.md` section 8.6, each carrying source, trust class, size, sensitivity, and compaction priority.
- Define precedence and conflict behavior. Higher-trust policy may restrict lower-trust instructions; lower-trust text cannot weaken permissions.
- Track provenance for every injected instruction and attachment without exposing secrets.
- Budget context before provider invocation. Use deterministic truncation, summarization, and compaction policies.
- Keep the latest user intent, unresolved approvals, active files, and recent failures ahead of low-value history during compaction.
- Do not silently discard required tool results or approval state; emit a compaction event and retain a recoverable summary.
- Support project-aware context providers such as repository metadata and user-selected files only through runtime/Rust APIs.
- Treat repository instruction files as untrusted project content unless a later trust policy explicitly promotes them.

### Tool registry and execution

- Register built-in, plugin, and MCP tools in one canonical registry.
- Require each tool to declare input/output JSON Schemas, side-effect class, resource scope, approval requirement, timeout, concurrency class, maximum output, and origin.
- Separate read-only tools, project mutation tools, process tools, network-capable tools, secret-consuming tools, and user-interaction tools.
- Apply allow/deny policy before invocation, then invoke only through an adapter owned by the runtime.
- Route all machine-affecting operations to typed Rust RPC calls; Rust canonicalizes paths and makes the final capability decision.
- Use bounded output, pagination, artifact references, and truncation markers for large results.
- Assign stable tool-call IDs and idempotency keys; persist request, approval, progress, result, and failure events.
- Support timeout, cancellation, crash, unknown completion, and retry states explicitly.
- Prevent a tool from invoking another tool by hidden side channel; nested calls must be declared and authorized.

### Permissions and approvals

- Use a policy evaluator in TypeScript for preflight explanation and grant lookup, with Rust as the authoritative OS capability boundary.
- Represent a grant by operation class, canonical resource scope, argument restrictions, project/session scope, lifetime, origin, and audit metadata.
- Support at minimum deny, allow once, allow for the session, allow for the project, and explicit persistent grants where policy permits.
- Treat network access, process execution, secret use, writes outside the safe project, destructive operations, and external MCP calls as separately classifiable capabilities.
- Return an approval requirement before execution when an applicable grant is absent or insufficient.
- Persist approval request, user decision, capability assertion, Rust result, expiry, revocation, and audit events.
- Never let a plugin, skill, model, client, or MCP server manufacture or broaden a grant.
- Make policy changes affect future calls only unless an explicit revocation/reconciliation rule exists.
- Resolve duplicate or concurrent approvals idempotently and notify all authorized clients.

### Skills

- Define a manifest containing stable ID, name, version, description, supported scopes, instruction entrypoint, resources, required capabilities, compatible runtime range, and provenance.
- Discover skills from approved built-in, project, user, and plugin locations through a deterministic precedence order.
- Require explicit enablement for skills that add capabilities, access secrets, contact external services, or change policy-relevant behavior.
- Load only the selected skill's bounded instructions and resources into context.
- Keep skill instructions separate from system policy and mark their provenance in diagnostics.
- Validate manifests and schemas before registration; quarantine invalid or incompatible skills.
- Version and cache skill metadata without caching secrets or unbounded generated content.

### Plugins

- Define a signed or otherwise trusted manifest with plugin ID, version, publisher/provenance, runtime compatibility, exported contributions, requested capabilities, and integrity metadata.
- Load plugins through a host API rather than allowing arbitrary imports into the runtime process.
- Prefer an isolated worker process for untrusted or third-party plugin code, with an explicit message protocol and resource limits.
- Expose narrow extension points: provider adapter, tool adapter, skill bundle, context provider, command, event observer, or client metadata.
- Require capability declarations and approval policy for every plugin contribution.
- Support install, update, disable, uninstall, health, and rollback as runtime operations; do not mutate plugin state during an active turn.
- Pin resolved plugin versions per project/session so a running or replayed turn is reproducible.
- A failed plugin must be quarantined without preventing unrelated sessions from recovering.

### MCP

- Implement MCP as a runtime-owned client integration, not a client or Rust concern.
- Support configured transports only after a separate security design defines local process launch, remote transport, authentication, origin, and network policy.
- Import MCP tools, resources, and prompts into namespaced registry entries with source metadata.
- Revalidate MCP schemas and tool descriptions at discovery and invocation time.
- Apply the same capability policy, approval UI, timeout, cancellation, output limit, secret redaction, audit, and failure handling as built-in tools.
- Treat MCP server instructions and prompt templates as lower-trust context; they cannot override Suncode policy or system instructions.
- Require explicit user configuration for servers that can access networks, secrets, or resources outside the selected project.
- Surface server health, version, capabilities, and disabled reason to clients.

### Sessions, persistence, and replay

- Own SQLite connections, migrations, transactions, encryption metadata, durable streams, projections, and database health in the runtime.
- Separate durable state into the three streams defined in `ARCHITECTURE.md` section 7.1: an immutable audit log, compactable session content, and disposable client sync state. Do not give them a shared sequence counter; cross-stream links use correlation identifiers.
- Record authority decisions in the audit stream only: capability requested, scope evaluated, decision and its source, grant lifetime, and operation outcome. Never compact or rewrite it.
- Record messages, tool calls and results, turn and tool-call transitions, context summaries, and artifact references in session content.
- Assign strictly increasing per-session content sequence numbers in one transaction with affected projections.
- Store provider/model selection and resolved extension versions needed to explain or replay a turn.
- Keep sensitive credentials encrypted and never include them in any stream.
- Rebuild disposable projections deterministically from session content.
- Support snapshot plus replay for clients and runtime recovery; return `resume_unavailable` when retention prevents replay.
- Make unknown provider/tool completion visible and reconcile it before retrying, using observed file hashes rather than assumption.
- Define per-stream retention, compaction, export, deletion, and backup behavior in a separate persistence design. A retention rule for one stream must not be applied to another.

### Client-facing runtime API

- Expose a presentation-safe authenticated HTTP/WebSocket API on loopback, defined by a written contract document with shared test vectors. No generated types or generated SDK.
- Authenticate every connection with the runtime credential. Never treat a loopback origin as authentication.
- Cover runtime health, capabilities, project, session, turn, events, approvals, settings, models, artifacts, checkpoints, extensions, diagnostics, and reconnect/resume.
- Stream ordered session events and bounded progress notifications.
- Require session scope and idempotency keys on mutating operations.
- Allow multiple authorized clients to observe one session while preventing ambiguous concurrent mutations.
- Never expose provider API keys, Rust transport, SQLite paths, raw filesystem handles, or privileged internal RPC methods.

### Observability and operations

- Emit structured runtime logs with severity, deployment, project/session correlation, provider/model, turn, tool, plugin, and MCP identifiers where safe.
- Record metrics for turn latency, time-to-first-token, token usage, cost estimate, retries, approval wait, tool duration, queue depth, failures, and compactions.
- Add distributed trace correlation across client request, runtime turn, provider call, tool call, MCP call, and Rust operation without recording secret payloads.
- Provide redacted diagnostics and health state to clients.
- Apply configurable log and telemetry sampling to high-volume token and progress events.
- Ensure logs do not share Rust protocol stdout and do not contain full prompts, secrets, or sensitive file contents by default.

### Configuration and secrets

- Resolve settings through the four layers defined in `ARCHITECTURE.md` section 11.1: built-in defaults, committed project file, interface-managed user settings, then environment and command-line overrides.
- Treat the project file as untrusted content. It may declare skills, MCP servers, model preferences, and policy profiles, but activating anything security-relevant requires user confirmation recorded in user settings.
- Keep secrets in user settings only. A project file may reference a credential, never contain one.
- Report effective values with their originating layer so a surprising setting is traceable.
- Encrypt classified secrets with a master key held in the OS credential store.
- Never place a secret value in a protocol message body. Operations needing a credential in a child process receive a scoped reference, and the core injects the value at launch.
- Store only references and redacted metadata in logs and durable streams.
- Validate settings before applying them and expose effective scope: global, project, session, or turn.
- Define an explicit secret rotation and invalidation path before enabling provider integrations.

### Checkpoints

- Request a core checkpoint before a turn's first filesystem mutation and anchor it to a session content event.
- Extend the checkpoint as a turn touches additional paths.
- Expose restore as an audited operation requiring authorization, never a silent action.
- Surface a conflict for user resolution when a file changed outside the agent since capture, instead of overwriting.
- State clearly that restore covers filesystem changes only and does not undo external side effects.
- Make checkpoint retention and expiry visible before a user relies on it.

## Edge cases

- Provider returns partial output then disconnects, rate-limits, or reports a malformed tool call.
- Context exceeds the selected model limit after a tool result or attachment arrives.
- A tool or MCP server changes its schema between discovery and invocation.
- Approval is denied, expires, is revoked, or resolves on another client.
- Rust completes an operation while the runtime is restarting and its result is initially unknown.
- Plugin crashes, hangs, requests undeclared capabilities, or is upgraded during a session.
- Two clients submit turns for one session simultaneously.
- A skill or repository instruction conflicts with system policy.
- Session projection is corrupt while the append-only journal remains available.
- Provider credentials are revoked while a turn is streaming.
- Network loss occurs during a non-idempotent provider or tool operation.

## Acceptance criteria

1. A reviewed architecture diagram shows client API, runtime kernel, provider adapters, context engine, agent loop, tool registry, policy engine, extension hosts, SQLite, and core RPC boundaries.
2. The two-level state machine defines turn lifecycle, per-tool-call lifecycle, multiple calls per assistant message, cancellation, retry, unknown completion, approval ordering, and recovery transitions.
3. Provider, tool, skill, plugin, and MCP manifests/interfaces define capabilities, provenance, schemas, compatibility, and requested authority.
4. A threat model covers provider credentials, prompt/context injection, tool abuse, plugin/MCP compromise, approval bypass, and log/telemetry leakage. It states plainly that the runtime process is trusted and not contained by the core boundary.
5. A separate client-runtime contract document covers all operations and event types the first two surfaces need.
6. Persistence design specifies the three durable streams, their independent retention, projections, secret classification, and replay semantics before functional implementation.
7. Shared test vectors cover valid and invalid provider responses, tool calls, approvals, event ordering, cancellation, limits, and extension failures, and both implementations agree on every vector.
8. No proposed subsystem creates a direct client-to-provider, client-to-core, plugin-to-core, or extension-to-SQLite path.
9. Phase 1 executes tool calls sequentially by policy while the state model, identifiers, and events already express concurrent calls.
10. A non-interactive path exists: a policy profile authorizes a scripted run, unauthorized operations fail with typed errors, and audit records mark profile-sourced authorization.
11. Runtime implementation begins only after this requirement and its blocking decisions are approved.
12. Phase 1 contains no executable plugin, MCP server, or third-party provider adapter; any such component fails closed as unavailable.

## Open questions

Blocking:

- Which provider and authentication method is first, and is it an API key or an OAuth subscription login? This decides credential storage, first-run experience, and how cost is presented, and several other answers depend on it.
- Should Suncode standardize on one canonical message format based on Responses-style content parts, Chat Completions-style messages, or its own schema?
- Which operations require approval by default, and what does the default policy profile authorize?
- What are default per-turn token, time, cost, iteration, tool-call, and output budgets?

Non-blocking:

- Which local MCP transports are allowed initially?
- What plugin trust model is acceptable for a local developer tool, and are third-party plugins in scope at all before the first release?
- What are the per-stream retention defaults, and how are exported sessions redacted?
- How is cost presented when a provider omits or disagrees on usage?
- Do background jobs and scheduled tasks belong in this milestone or a later automation subsystem?
- What is the initial fixed task suite for behavioral evaluation, and what pass rate gates a release?

