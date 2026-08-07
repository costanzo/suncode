# OpenCode Runtime Comparison

- Compared on: 2026-08-07
- OpenCode source: local checkout at commit `aefaf140c19e25494da27739ae979f31b8cfe474` on branch `dev`
- Scope: runtime orchestration, LLM abstraction, context, tools, permissions, skills, plugins, MCP, persistence, and client API

## Executive assessment

The Suncode draft and OpenCode converge on the important macro-boundaries: provider-neutral LLM calls, durable sessions, per-session serialization with cross-session concurrency, a canonical tool registry, scoped permissions, composable context, skills, plugins, MCP, streaming events, output bounds, and observability.

Suncode is stricter in two important areas:

- Machine-affecting work and final permission enforcement are isolated in Rust rather than trusted to the TypeScript tool implementation.
- Configuration and secrets are runtime-owned and interface-managed rather than centered on user-edited project configuration.

OpenCode is more mature in several mechanisms that the original Suncode draft described only generally. The highest-value additions are durable input admission, explicit input delivery modes, frozen execution snapshots, context epochs, provider prompt caching, managed oversized outputs, catalog-versus-execution authorization separation, and loop-stall detection.

## Capability matrix

| Area | OpenCode evidence | Existing Suncode draft | Recommendation |
| --- | --- | --- | --- |
| Durable prompt admission | Records `PromptAdmitted` before an advisory execution wake | Turn submission is durable but admission and scheduling are not sharply separated | Add a durable inbox and make execution wake advisory |
| Mid-run user input | Distinguishes `steer` from `queue` and promotes them at safe boundaries | Rejects or queues conflicting turns without delivery semantics | Add `steer`, `queue`, and an explicit replacement/cancel operation |
| Per-session scheduling | Process-local coordinator joins/resumes by session and coalesces wakeups | Sequential per session, concurrent independent sessions | Adopt coalesced wake semantics and define crash-recovery ownership separately |
| Context change tracking | Persists a context epoch and emits `ContextUpdated` when sources change | Tracks provenance and resolved versions | Add a context snapshot/epoch hash and safe reconciliation rules |
| Tool catalog stability | Materializes a registry for a provider turn and rejects stale calls | Has a canonical registry but does not freeze advertised definitions | Freeze a tool snapshot per provider turn and reject stale/mismatched calls |
| Catalog filtering vs authorization | Hides wholly denied tools but leaf execution still authorizes | Policy filters tools and authorizes calls | Explicitly document these as separate controls |
| Tool output management | Bounds model-visible output, saves full output, and cleans it by retention | Requires bounded output and artifact references | Specify opaque managed-output artifacts, retention, retrieval, and cleanup |
| Provider architecture | Separates portable request, provider definition/model, protocol, route/transport, one provider turn, and complete run | Canonical provider adapter and one runtime loop | Add provider-turn as an explicit primitive and separate serializable request data from executable model behavior |
| Provider prompt caching | Cache policy is explicit and usage is normalized across providers | Not specified | Add canonical cache policy and cache-read/write usage accounting |
| Provider extensibility | Typed provider options, staged hooks, HTTP escape hatch, protocol authoring | Provider adapter contract only | Add constrained hook stages and typed escape hatches; prevent hooks from changing orchestration |
| Agent profiles | Named primary/subagent/all profiles include model, prompt, permissions, steps, and visibility | One generic agent loop; skills select workflows | Add named execution profiles/modes, but keep subagent execution disabled in Phase 1 |
| Loop-stall protection | `doom_loop` permission detects repeated tool behavior | Only iteration/tool budgets | Add repeated-call/stall detection with visible approval or termination |
| Compaction | Anchored summaries retain objectives, work state, exact identifiers, and recent history | Deterministic compaction requirements | Define a structured compaction artifact and quality/integrity checks |
| MCP lifecycle | Local/remote config, startup/request timeouts, OAuth state, pagination guards, namespacing, schema tolerance | General MCP security and lifecycle requirements | Add bounded pagination, cursor-cycle detection, progress-aware timeouts, schema-drift handling, and OAuth state lifecycle |
| Plugin loading | Tracks plugin origin, resolves relative paths by declaring config, deduplicates by identity, and orders loading | Manifest, provenance, isolation, pinning, quarantine | Add deterministic precedence, origin-preserving resolution, dependency graph, readiness barrier, and atomic reload |
| Project-scoped services | Uses per-location service graphs with scoped cleanup | Workspace/session isolation stated generally | Add workspace service containers and lifecycle-scoped cleanup |
| Event evolution | Separates current versus compatibility event surfaces | Versioned events and compatibility ranges | Add explicit current/legacy event classification and avoid leaking compatibility events into new clients |

## Recommended Phase 1 additions

### 1. Durable session inbox

Record the user's input transactionally before scheduling execution. The durable input record should include:

- input ID and idempotency key
- session, workspace, and authorization scope
- admitted sequence and timestamp
- prompt and attachments
- delivery mode
- selected agent profile/model override where allowed
- promotion/execution sequence when consumed

The wake signal is only a hint. After restart, the runtime drains eligible admitted inputs from SQLite instead of trusting an in-memory queue.

### 2. Explicit delivery semantics

Support these client intents:

- `queue`: wait until the current turn reaches an idle boundary.
- `steer`: incorporate the new input at the next safe provider-turn boundary if the current run continues.
- `cancel-and-replace`: cancel the current run, then promote the replacement after cancellation settles.

The client must never guess which behavior occurred. Admission and promotion are separate events.

### 3. Frozen execution snapshots

At each provider turn, resolve and snapshot:

- provider/model and capabilities
- agent profile and budgets
- context epoch
- tool definitions and their implementation identities
- policy/grant snapshot
- skill/plugin/MCP versions
- provider cache policy

A model tool call is settled only against the tool snapshot it was shown. Configuration reloads apply at a later safe boundary.

### 4. Workspace service containers

Create scoped service containers keyed by stable workspace/runtime placement. Provider defaults, tool registrations, instruction sources, skills, plugins, MCP connections, watchers, and cleanup must be acquired and released with the workspace container. Process-global services should be limited to genuinely global concerns such as installation catalog metadata and runtime telemetry.

### 5. Managed output artifacts

Return a bounded head/tail preview to the model and client while retaining complete output behind an opaque artifact reference. Rust should own the file/artifact handle and access checks; TypeScript owns session metadata and retention policy. Never inject a raw storage path into model-visible content.

### 6. Provider call architecture

Define one provider turn separately from the agent run:

- A provider turn sends exactly one canonical request and yields normalized events.
- The Suncode kernel, not the provider library, owns durable tool dispatch and continuation.
- Serializable request/message/tool-definition values are separate from process-local models, transports, handlers, hooks, and credentials.
- Provider hooks may observe or transform named stages but cannot approve tools, synthesize durable events, retry side effects, or control the agent loop.

### 7. Prompt caching and usage accounting

Add a provider-neutral cache policy with `auto`, `none`, and explicit hints. The provider adapter converts it to native semantics and returns normalized cache-read/cache-write token usage. Cache identity must account for model, stable instructions, tool definitions, context epoch, and policy-visible content without incorporating secrets into logs or keys.

### 8. Context epochs

Persist the snapshot/hash of instruction and system-context sources used by a session. When workspace instructions, selected skills, plugins, environment facts, or policy-facing context changes:

- reconcile at a safe provider-turn boundary
- emit a durable context update event
- explain added, removed, or replaced sources in redacted diagnostics
- retain the prior epoch for historical explanation
- never retroactively reinterpret old events under new instructions

### 9. Loop-stall detection

Detect repeated equivalent tool calls, alternating no-progress calls, repeated recoverable errors, and budget consumption without new durable state. Policy may stop the turn or require a user decision. The detection must use structured tool name/input/result fingerprints and avoid treating legitimate paginated work as a loop.

### 10. Hardened MCP catalog behavior

Add explicit requirements for:

- maximum discovery pages and total items
- duplicate/non-advancing cursor detection
- sanitized, collision-resistant namespacing
- schema-size/depth limits and safe handling of invalid optional output schemas
- separate startup, discovery, request, and idle timeouts
- progress-aware timeout reset with an absolute deadline
- OAuth state, PKCE verifier, client registration, token refresh, revocation, and redirect validation
- capability-change events and per-turn frozen MCP tool snapshots

## Later milestones

OpenCode includes or is evolving capabilities that should not expand Suncode Phase 1:

- Parallel/general/explore subagents and child-session permission derivation.
- Worktree creation, session movement, and distributed control-plane placement.
- PTY persistence and interactive terminal ownership.
- LSP and formatter orchestration.
- Session sharing and public synchronization.
- Background agents and scheduled work.
- Provider-hosted tools that execute outside Suncode's normal tool path, unless a separate authority model is approved.

The Phase 1 schemas should reserve identifiers and event causation fields for future child runs, but no subagent scheduler or cross-machine execution should be implemented yet.

## OpenCode choices not to copy directly

- OpenCode uses Bun in the inspected checkout; Suncode's approved runtime is Node.js.
- OpenCode can execute machine-facing TypeScript tools directly; Suncode routes those operations through Rust.
- OpenCode supports user/project configuration files; Suncode settings remain API-backed and interface-managed.
- OpenCode's current plugin loading includes in-process dynamic modules; Suncode should keep third-party plugin code behind an isolated host.
- OpenCode's managed tool-output implementation exposes paths in some current surfaces; Suncode should expose opaque permission-checked artifact references.
- OpenCode contains current/V1 transition paths. Suncode should begin with one canonical contract and add compatibility layers only when real migration requires them.

## Source locations inspected

- `packages/core/src/session/input.ts`
- `packages/core/src/session/execution.ts`
- `packages/core/src/session/run-coordinator.ts`
- `packages/core/src/session/runner/`
- `packages/core/src/session/context-epoch.ts`
- `packages/core/src/session/history.ts`
- `packages/core/src/session/compaction.ts`
- `packages/core/src/system-context/`
- `packages/core/src/tool/`
- `packages/core/src/tool-output-store.ts`
- `packages/core/src/skill/`
- `packages/opencode/src/provider/`
- `packages/opencode/src/session/`
- `packages/opencode/src/permission/`
- `packages/opencode/src/plugin/`
- `packages/opencode/src/mcp/`
- `packages/llm/`
- `packages/schema/src/`

