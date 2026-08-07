# Architecture

## Current state

The approved architecture defines TypeScript on Node.js as the owner of model integration, context construction, agent loops, orchestration, approvals, settings, SQLite, session state, and the client API. Rust is the trusted child process for filesystem, process, sandbox, project-boundary, artifact, and capability enforcement. The foundational milestone intentionally excludes their functional implementation.

This document proposes the runtime internals and identifies contracts that must be specified separately. It follows `ARCHITECTURE.md` for the layer split, the two-level state machine, the three durable streams, authority, and configuration layering; where this document and that one disagree, that one governs.

Suncode is local-first, so nothing here assumes tenancy, remote identity, or hosted infrastructure. Protocol contracts are hand-implemented per language, so nothing here assumes generated types.

## Proposed topology

```text
        CLI / TUI            Qt desktop
              \                 /
        authenticated client API (loopback)
                      |
              runtime server + auth scope
                      |
   session service: audit / content / sync + projections
                      |
              agent runtime kernel
       /              |              \
 context          policy          turn scheduler
 engine           engine               |
       \              |              /
          tool / extension registry
          /           |           \
     built-in     plugins      MCP client
          \           |           /
              provider gateway
                      |
              model provider APIs

 agent runtime kernel -- private JSON-RPC (2 channels) -- Rust OS core
```

The runtime kernel is the only component allowed to compose a turn. Other modules provide typed services to it; none may start an untracked agent loop.

The kernel and everything below the client API is a library. The runtime server is a thin wrapper, so the CLI can host the same kernel in-process for a single non-interactive session.

## Module responsibilities

### Runtime server

Authenticates clients against the runtime credential, establishes project and session scope, validates request envelopes, applies rate limits, and maps runtime events to client subscriptions. It does not call providers or Rust directly outside service interfaces, and it never treats a loopback origin as authentication.

### Session service

Owns SQLite, migrations, and the three durable streams: the immutable audit log, compactable session content, and disposable client sync state. It handles appends, projection updates, snapshot queries, replay, per-stream retention, and idempotency records. It is the source for durable session state and the recovery coordinator for the kernel.

Each stream has its own retention policy. Audit is never rewritten. Content is the rebuild source for projections. Sync state is recreatable and never authoritative.

### Turn scheduler

Serializes turns per session, permits independent sessions to run concurrently, applies queue and resource policy, tracks cancellation, and prevents duplicate idempotency keys from causing duplicate execution. It drains durably admitted input rather than trusting an in-memory queue, and it coalesces repeated wake signals.

Within a turn it schedules the tool calls of one assistant message — sequentially in Phase 1, though it receives them as a set.

### Agent runtime kernel

Coordinates the two-level state machine from `ARCHITECTURE.md` section 8.1. It asks the context engine for a bounded prompt, freezes a turn snapshot, calls the provider gateway, validates model output, opens a child machine per requested tool call, drives each through policy and execution, appends results, and repeats until completion, failure, cancellation, or budget exhaustion.

It owns tool dispatch and continuation across provider calls. A provider adapter or vendor SDK never drives the loop.

### Provider gateway

Owns provider adapters, model catalog, credential references, capability negotiation, usage normalization, retry classification, and provider health. Adapters may use vendor SDKs but expose only canonical runtime types.

### Context engine

Resolves instruction layers, selected skills, plugin context providers, project metadata, session summaries, attachments, and tool results. It records provenance and applies deterministic token budgeting and compaction.

### Policy engine

Evaluates runtime-level grants and produces a typed authorization or approval requirement. It cannot override the core's capability decision. It also resolves policy profiles for non-interactive runs, where a pre-authorized capability substitutes for a prompt but not for enforcement or audit.

Separately, it filters advertised tools so the model sees only capabilities usable in the current scope. Filtering shapes context; preflight decides calls. Neither replaces the other.

### Tool registry and executor

Registers canonical tool definitions and dispatches to built-in adapters, Rust operation adapters, plugin hosts, or MCP servers. It handles schema validation, timeout, cancellation, output bounds, idempotency, approval, redaction, and audit events uniformly.

### Extension manager

Discovers, validates, resolves, pins, starts, health-checks, disables, and upgrades skills, plugins, and MCP servers. It keeps extension provenance and compatibility state available to diagnostics and replay.

### Observability service

Emits structured logs, metrics, traces, and redacted diagnostic snapshots. It is deliberately downstream of domain services so telemetry failure cannot silently change authorization or turn behavior.

## Trust and extension boundaries

```text
trusted runtime kernel/policy/session code
        |
        +-- provider adapters (network + secret reference)
        +-- built-in tools (typed Rust/client interfaces)
        +-- plugin worker (restricted IPC + declared capabilities)
        +-- MCP client (declared external server boundary)
        +-- skill loader (data/instructions, no ambient code authority)
        |
        +-- Rust core (authoritative OS enforcement)
```

- A skill is data and instructions by default, never executable authority.
- A plugin is code and therefore requires a host boundary, manifest, compatibility check, and capability grant.
- An MCP server is an external principal; its tools are untrusted until policy permits invocation.
- Provider responses and all extension content are untrusted input.
- No extension receives the runtime database connection, master key, Rust stdin/stdout, unrestricted environment variables, or arbitrary client sockets.

## Canonical turn flow

```text
client.submitInput
        |
validate scope + idempotency + delivery mode
        |
admit durably -> persist input.admitted        (advisory wake)
        |
promote -> persist turn.admitted / turn.queued
        |
resolve model + policy-filtered tools + context
        |
freeze turn snapshot -> persist turn.prepared
        |
provider call (one provider turn)
   | text deltas / usage / N tool calls
   v
validate output against the frozen snapshot
   |
   +-- text ------> persist assistant.message
   |
   +-- N tool calls -> open a child machine per call
                         |
                    (per call, independently)
                         |
                    validate args
                         |
                    policy preflight
                      | deny -----> persist tool.denied            + audit
                      | approval -> persist approval.requested
                      |             await decision                 + audit
                      | allow ----> persist tool.authorized        + audit
                                       |
                                    invoke adapter (core / MCP / plugin)
                                       |
                                    persist tool.progress / result
                                       |
                                    on unknown -> reconcile by hash
        |
all child machines terminal
        |
append bounded results to context and loop
        |
completed | failed | cancelled | interrupted
```

Every arrow is an explicit service call or a durable append. Authority decisions go to the audit stream; conversation goes to the content stream. A process restart can recover the last durable state and determine whether the next transition is safe.

Approval always precedes invocation. Note that policy preflight and catalog filtering are separate: filtering decides what the model was shown in the frozen snapshot, preflight decides whether an actual call proceeds, and neither substitutes for the other.

## Provider adapter contract

The canonical provider interface should include:

- `listModels()` with stable model ID, provider ID, capabilities, limits, pricing metadata, availability, and deprecation state.
- `startResponse(request, signal)` yielding content deltas, tool-call deltas, usage, warnings, and a terminal outcome.
- `cancel(requestId)` where supported, otherwise cooperative stream closure.
- `classifyError(error)` returning authentication, invalid request, context limit, rate limit, transient, policy, provider outage, or unknown.
- `estimate(request)` for preflight token/cost checks where the provider supports it.

The gateway owns provider-specific message conversion and does not expose vendor response objects to the kernel or clients. Provider adapters must support a deterministic request transcript for diagnostics with secrets and sensitive content redacted.

## Context architecture

Context is represented as a list of typed, provenance-bearing parts rather than one opaque prompt string:

1. Runtime policy and safety instructions.
2. Suncode product/system instructions.
3. Project and session policy.
4. Selected skill instructions and resources.
5. Approved plugin/MCP context contributions.
6. Stable project metadata and user-selected file references.
7. Compacted session summary and recent durable events.
8. Current user input and attachments.
9. Tool results and unresolved approval state.

Each part has source, trust class, size, sensitivity class, and compaction priority. A higher-trust part may constrain or exclude a lower-trust part, never the reverse. The context engine must be able to explain why a part was included or omitted without revealing secret values.

## Policy architecture

Policy evaluation is two-stage:

1. **Runtime preflight:** determine whether the operation is known, in scope, declared by the tool/extension, covered by a grant, or requires user approval.
2. **Rust enforcement:** validate the capability assertion, canonicalize paths and arguments, enforce project and OS limits, and return authorization, denial, approval-required, or execution result.

The runtime policy engine should be pure and testable for a request plus policy snapshot. It must not make decisions from model wording, UI labels, or unverified path strings. Policy outcomes and their reason codes are durable audit events.

## Event model

Events are written to the stream that matches their consumer. Authority decisions go to the audit log — capability requested, scope evaluated, decision and source, grant lifetime, outcome — and are never compacted. Everything below is session content unless noted.

At minimum, the streams should cover:

- runtime/session initialization and recovery
- input admission and promotion, recorded separately so delivery is never inferred
- user input and assistant output
- turn state transitions and budgets
- per-tool-call state transitions, identified independently of the turn
- checkpoint capture and restore, with restore also in the audit stream
- provider request, delta summary, usage, retry, and terminal outcome
- context build and compaction summary
- tool requested, approval requested, decision, progress, result, timeout, cancellation, and unknown completion
- skill/plugin/MCP discovery, enablement, health, disablement, and version resolution
- artifact metadata and retention references
- settings and policy changes
- diagnostic and security-relevant decisions

Payloads are versioned, bounded, redacted, and correlated. High-volume token deltas may be streamed to clients without persisting every delta; session content stores reconstructable message checkpoints and usage summaries.

## Failure and recovery

- Provider streams are resumable only when the provider contract makes it safe; otherwise the turn is interrupted with a visible recovery state.
- A Rust child restart fails outstanding calls as `core_unavailable` and reconciles unknown operation outcomes before retry.
- An extension worker restart does not replay a non-idempotent call automatically.
- Context compaction is transactional with its summary event and cannot erase unresolved approvals.
- A corrupt projection is rebuilt from session content; corrupt or unavailable content leaves the runtime diagnostic and mutation-disabled. A corrupt audit log also disables mutation, because operating without an authority record is not acceptable.
- Backoff, circuit breakers, and per-provider concurrency limits prevent one provider or extension from starving all sessions.
- Shutdown waits for active turns, core operations, and durable writes up to a configured grace period, then records an interrupted recovery marker.

## Compatibility and migration

- Runtime API, runtime-to-Rust protocol, provider adapter interface, tool schemas, extension manifests, and event payloads version independently but declare compatibility ranges.
- A session records resolved model, provider, skill, plugin, MCP, tool, and policy versions necessary to explain its behavior.
- Unknown compatible fields are tolerated; unknown required capabilities fail with typed errors.
- Plugin and skill upgrades are not applied to an active turn and require a new resolution for future turns.
- Context summaries and projections carry schema versions and migration/rebuild paths.

## Risks and rollback

- **Provider abstraction leakage:** vendor-specific semantics may be lost. Keep escape hatches as typed optional capabilities, not untyped payloads.
- **Prompt/context injection:** repository, skill, MCP, and provider content may attempt to override policy. Preserve trust classes and enforce policy outside the model prompt.
- **Extension supply-chain risk:** require integrity/provenance, least privilege, isolation, and quarantine.
- **Event volume:** persist checkpoints and bounded summaries rather than every token delta.
- **Retry duplication:** classify idempotency and unknown completion before retrying any external side effect.
- **Budget surprises:** preflight and continuously enforce token, time, cost, tool, and output budgets.
- **Cross-client races:** use session sequence and mutation idempotency, not UI locking alone.

This is documentation only; rollback means revising or superseding the requirement before implementation.

## Open questions

- Decide the canonical content/message schema and whether multimodal parts are Phase 1.
- Define the independent child-process extension host and platform sandbox contract in the deferred extension trust-boundary delivery.
- Decide the initial MCP transport policy.
- Define exact secret classes and rotation behavior against the OS credential store.
- Define the initial tool catalog and default risk classifications.
- Define per-stream retention, compaction thresholds, and session export semantics.
- Define the scoped-secret-reference mechanism for operations needing a credential in a child process.
- Decide whether the audit log needs tamper-evidence such as a hash chain, or whether file permissions suffice for a local single-user tool.

