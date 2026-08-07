# OpenCode-Informed Runtime Amendments

- Date: 2026-08-07
- Status: **Consolidated — no longer normative**
- Applies to: `requirement.md` and `architecture.md` in this directory
- Evidence: `opencode-comparison.md`

These proposals were accepted and folded into `requirement.md` and `architecture.md` on 2026-08-07. Read those two documents for current requirements; this file is retained only as the reasoning behind them, so it should not be cited as a separate source of truth.

Note where consolidation changed a proposal: the frozen-snapshot and durable-admission items were adopted as written, but the turn state machine they assumed was replaced by the two-level turn and tool-call machine in `ARCHITECTURE.md` section 8.1, and the single event journal was replaced by three durable streams.

## Domain additions

### Admitted input

An admitted input is a user request durably recorded before execution scheduling. Its delivery mode is `queue`, `steer`, or `cancel-and-replace`. Admission acknowledges durability; promotion into the conversation records when the agent loop actually consumes it.

### Agent profile

A named, versioned set of system instructions, provider/model defaults, tool visibility, permissions, budgets, and interaction behavior. Phase 1 supports primary profiles only; schemas may reserve subagent eligibility without enabling subagent execution.

### Provider turn

Exactly one canonical request to a model provider and its normalized streamed response. The Suncode kernel, not the provider adapter or provider library, owns durable tool dispatch and continuation across provider turns.

### Context epoch

A durable identity for the resolved set of instruction and context sources used at a safe execution boundary. Context changes create or reconcile a new epoch; they do not retroactively reinterpret earlier events.

## Required changes

### Durable input admission and delivery

- Admit input transactionally before issuing an advisory execution wake.
- Persist input ID, idempotency key, scope, prompt, attachments, delivery mode, selected profile/model override, admission sequence, and promotion sequence.
- Drain eligible admitted inputs from SQLite after restart; an in-memory queue is never authoritative.
- Support `queue`, `steer`, and `cancel-and-replace` as explicit client intents.
- Promote `steer` only at safe provider-turn boundaries.
- Promote queued inputs in durable admission order when the session would otherwise become idle.
- Persist separate admission and promotion events so clients never infer delivery behavior.
- Coalesce repeated wake signals without dropping durable inputs.

### Workspace-scoped services

- Create lifecycle-scoped service containers keyed by stable workspace/runtime placement.
- Scope provider defaults, tool registrations, context sources, skills, plugins, MCP connections, watchers, and cleanup to the workspace container.
- Restrict process-global services to genuinely global concerns such as installation metadata, runtime coordination, and telemetry.
- Release processes, network connections, subscriptions, and caches deterministically when the workspace container closes.

### Frozen provider-turn snapshot

Before each provider turn, resolve and freeze:

- provider, model, protocol capabilities, and model limits
- agent profile and budgets
- context epoch
- effective tool definitions and implementation identities
- policy and grant snapshot
- skill, plugin, and MCP versions
- provider prompt-cache policy

Model tool calls must settle against the exact materialized tool snapshot advertised in that provider turn. A tool that was replaced, removed, or reconfigured must fail as stale rather than run a different implementation under the old name. Configuration and extension reloads take effect only at safe boundaries.

### Provider architecture and caching

- Separate serializable request, message, tool-definition, event, usage, and error values from executable models, transports, hooks, credentials, and handlers.
- Keep provider-specific protocol/wire behavior behind adapters while preserving typed provider metadata needed for continuation.
- Add canonical prompt-cache modes: `auto`, `none`, and explicit cache hints.
- Normalize cache-read and cache-write token usage and include it in per-turn accounting.
- Include model, stable instruction identity, tool snapshot, context epoch, and policy-visible content in cache identity where applicable.
- Permit typed provider options and named-stage hooks for canonical request, native body, prepared transport, normalized event, and error handling.
- Hooks may observe or transform their declared stage but cannot grant permission, dispatch tools, synthesize durable events, retry side effects, or control the agent loop.

### Context epochs and compaction

- Persist the resolved context source snapshot or hash for each context epoch.
- Reconcile workspace instructions, skills, plugin contributions, environment facts, and other dynamic sources at safe provider-turn boundaries.
- Emit durable events for added, removed, updated, rejected, or replacement-blocked context sources.
- Keep redacted provenance diagnostics for every included context source.
- Define compaction as a structured artifact containing at least objective, important constraints, completed work, active work, blockers, next action, and relevant exact identifiers or files.
- Validate compaction output structure before replacing history and retain the recent uncompacted tail.
- Never discard unresolved approvals, active tool state, exact error identifiers, or current user intent during compaction.

### Tool visibility, authorization, and managed output

- Treat catalog visibility and execution authorization as independent controls.
- The policy engine may hide wholly denied tools to reduce attack surface and prompt size, but every visible invocation still performs runtime preflight and Rust enforcement.
- Detect stale tool calls by materialized implementation identity, not name alone.
- Return bounded head/tail previews and explicit truncation metadata for oversized results.
- Retain complete output behind an opaque managed artifact reference with expiry, sensitivity class, owner session/tool call, content metadata, and access policy.
- Rust owns the underlying artifact/file handle and access checks; TypeScript owns session linkage and retention metadata.
- Never expose a raw storage path as a model-visible managed-output reference.

### Loop-stall detection

- Detect repeated equivalent tool calls, alternating no-progress patterns, repeated recoverable failures, and budget consumption without new durable state.
- Fingerprint normalized tool name, relevant input, resource scope, and outcome while excluding volatile fields.
- Do not flag legitimate pagination, polling with declared progress, or changing resource state as a loop.
- Policy may terminate the turn or request explicit user confirmation before continuing.
- Emit the triggering evidence and decision as redacted audit events.

### Plugin lifecycle

- Resolve plugin origin and declaring scope deterministically.
- Validate dependency graphs and reject cycles, missing dependencies, incompatible versions, duplicate identities, and ambiguous precedence.
- Require a readiness barrier before contributions become visible.
- Apply enable, disable, update, and reload atomically at safe turn boundaries.
- Pin the resolved plugin set for a provider turn and record it for session explanation.
- Continue to require isolated hosting for third-party plugin code; OpenCode's in-process dynamic import approach is not adopted.

### MCP hardening

- Bound discovery page count, total item count, schema byte size, schema depth, and total catalog size.
- Reject duplicate or non-advancing cursors.
- Use sanitized, collision-resistant namespaced identities while retaining the original server and capability names for diagnostics.
- Handle invalid optional output schemas without weakening input validation; record the compatibility workaround.
- Separate startup, discovery, request-inactivity, idle-connection, and absolute deadlines.
- Progress may reset an inactivity timeout but never the absolute deadline.
- Freeze the effective MCP tool snapshot for each provider turn and emit capability-change events for later turns.
- Persist OAuth state, PKCE verifier, dynamic client metadata, access/refresh tokens, expiry, refresh, revocation, and reauthentication state as encrypted secrets or secret references.
- Validate redirect URI, callback binding, OAuth state, server origin, and credential-to-server association.

### Contract evolution

- Classify runtime events as current, transitional compatibility, or legacy-only before including them in client protocol manifests.
- Maintain one canonical schema identity for each current event and type.
- Do not expose legacy events to new clients merely because an older execution path still emits them.
- Add compatibility layers only for real migrations; do not pre-create parallel current/V1 APIs.

## Additional acceptance criteria

1. Input admission, promotion, wake coalescing, and crash recovery are separately specified and tested.
2. Provider-turn snapshots freeze context, tools, policy, provider/model, cache policy, and extension versions.
3. Context epochs make instruction changes durable, explainable, and non-retroactive.
4. Stale tool calls cannot execute a replacement implementation under the same name.
5. Catalog hiding cannot substitute for execution authorization.
6. Oversized output is available only through opaque permission-checked artifacts with retention policy.
7. Loop-stall detection distinguishes repeated no-progress work from legitimate pagination and polling.
8. MCP catalog discovery and OAuth flows have explicit resource and security bounds.
9. Plugin dependency resolution and reload are deterministic and atomic at safe boundaries.
10. Provider caching usage is normalized and included in budget and telemetry accounting.

## Deferred after comparison

The following OpenCode capabilities remain outside Suncode Phase 1:

- parallel or specialized subagent execution
- worktree and distributed session placement
- persistent PTY and interactive terminal ownership
- LSP and formatter orchestration
- session sharing and public synchronization
- background agents and scheduled automation
- provider-hosted tools that bypass Suncode's normal tool authority path

