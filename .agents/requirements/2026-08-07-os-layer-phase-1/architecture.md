# Architecture

## Current state

The approved architecture places filesystem, process, project-boundary, sandbox, and OS-capability operations in the Rust core. The runtime requirement adds a canonical TypeScript tool registry, policy preflight, provider turns, context epochs, plugins, skills, and MCP. This document defines the core side of that boundary for Phase 1 design.

No OS-layer implementation exists yet. This package is a requirements and design record only. It follows `ARCHITECTURE.md` for the boundary rationale, the two protocol channels, the operation journal, artifact collection, and checkpoints; where the two disagree, that document governs.

## Core split

```text
Model-facing tool or service
        |
TypeScript runtime: registry, policy preflight, session content, audit
        |
typed private JSON-RPC operation request (stdio + control channel)
        |
Rust OS capability broker
        |
filesystem / search / process / sandbox / artifact / checkpoint / journal
```

TypeScript owns what the model sees. The core owns what the machine does.

This keeps provider-specific tool schemas, context, approval UX, session persistence, and extension orchestration in the runtime while making every privileged operation pass through one audited path.

Note what the boundary is *not*: an OS-enforced sandbox around the runtime. The runtime spawns the core as a child under the same user and could bypass it. The value is containment of third-party code, a single auditable choke point, and a home for performance-sensitive filesystem work — which is exactly why the surface must stay narrow rather than absorbing everything that touches the machine.

## Operation categories

Phase 1:

| Category | Core responsibility | Runtime responsibility |
| --- | --- | --- |
| Path resolution | Canonicalize, validate, enforce project scope | Request operation with declared project/session scope |
| Read | Read bounded file content, metadata, hashes, encodings | Decide what enters context and session content |
| Glob/list | Walk directories, apply ignore/hidden/binary policy, bound results | Present as tool result and store summaries |
| Search | Scan files, bound previews, produce artifact for full result | Decide when to search and how to rank results |
| Write/edit/patch | Apply mutation, check preconditions, detect conflicts, journal it | Build intended edit, request approval, persist outcome |
| Process/shell | Spawn, stream, limit, cancel, clean process trees | Model-facing command tool, approval UX, session events |
| Sandbox | Materialize execution envelopes, restrict filesystem/process/network/env/secrets | Request profile, explain approval, persist outcome |
| Artifact | Store, read, and sweep named artifacts with access checks | Mark garbage, link to sessions, expose client-safe handles |
| Checkpoint | Capture snapshots, verify hashes, restore, report conflicts | Anchor to session content, request restore, present conflicts |
| Journal | Record operation intent and outcome, answer duplicate queries | Reconcile unknown outcomes, persist audit records |

Deferred, each to its own package:

| Category | Core responsibility | Runtime responsibility |
| --- | --- | --- |
| Watchers | Subscribe to filesystem events, recover from overflow | Invalidate stale context, notify clients |
| File catalog | Ignore-aware path/hash catalog tied to watchers | Query and rank for context |
| Content-search index | Text-search structures | Query intent and ranking |
| Diff engine | Compute hunks from snapshots | Request, persist, and present diffs |
| VCS | Execute VCS commands under project authority and limits | Decide intent, summarize status and history |
| PTY | Interactive process handles | Terminal presentation |
| Toolchain inspection | Bounded, redacted environment and executable facts | Decide which facts enter context |

Not core at any phase:

| Category | Owner | Why |
| --- | --- | --- |
| Language-server orchestration | Runtime, over core-launched processes | Churns per language; the core supplies process lifetime and file reads only |
| Symbol and dependency indexes | Runtime, over core reads and searches | Semantic and ecosystem-specific |
| Embedding and vector indexes | Runtime | Requires provider calls, which the core never makes |

## Rust process modules

The later implementation should keep internal Rust crates or modules aligned around boundaries, not around model-facing tool names:

- `rpc-server`: JSON-RPC framing on both channels, validation, initialization, cancellation, and progress notifications.
- `capabilities`: grant assertion validation, policy result types, resource scopes, and denial reasons.
- `project`: project identity, canonical paths, filesystem metadata, hashes, and ignore policy.
- `fs-ops`: read, write, edit, patch, copy, move, delete, and metadata operations.
- `search`: glob, directory walking, content search, and search artifacts.
- `process`: command execution, process lifecycle, environment filtering, limits, and cleanup.
- `sandbox`: profile resolution, filesystem views, process isolation, network egress policy, temporary directories, and credential injection.
- `artifacts`: opaque artifact storage, access enforcement, and sweep execution.
- `checkpoint`: file snapshot capture, hash verification, and restore.
- `journal`: operation intent and outcome records, duplicate resolution, and recovery reporting.
- `limits`: shared time, memory, byte, concurrency, and cancellation enforcement.
- `diagnostics`: redacted OS-layer logs and safe health snapshots.

Deferred modules that would arrive with their packages: `watchers`, `repo-index`, `diff`, `vcs`, `environment`. There is no `lsp` module — language-server processes are ordinary `process` operations under a sandbox profile, and the protocol conversation belongs to the runtime.

These are implementation suggestions for later work; the current delivery does not create crate skeletons.

## Operation lifecycle

```text
runtime request
        |
schema validation
        |
canonicalize resources
        |
evaluate capability assertion and OS policy
        |
admit operation or return denial/approval-required
        |
execute with limits, progress, cancellation
        |
return bounded result plus artifact refs
        |
runtime persists events and reconciles state
```

Every mutating operation has an idempotency key, and the core journals its intent before executing. See the operation journal below.

## Operation journal

Because execution happens in the core and recording happens in the runtime with no transaction spanning both, a mutation whose response is lost leaves the runtime unable to tell whether it happened. That is the normal third outcome of every mutating operation, not an edge case.

The core therefore keeps a private append-only journal, written before the mutation and completed after:

```text
idempotency key | operation class | canonical target
pre-image reference | start time | terminal outcome (once known)
```

This makes the core authoritative for "did the operation with this key complete?" A duplicate request returns the recorded outcome rather than executing again.

On restart the core reports entries with no terminal outcome. The runtime resolves each by observation:

```text
read current hash
  == post-image -> completed, record result
  == pre-image  -> never happened, safe to retry
  neither       -> conflict, surface it, never retry
```

The journal is bookkeeping, not history. It is bounded, prunable, holds no conversation content, and is the only durable state the core owns. It never contains session events; the runtime decides how outcomes are recorded.

## Tool mapping

Built-in agent tools are named and described in the runtime; each machine-affecting implementation is a typed core operation.

Phase 1:

- `read_file` -> core bounded file read
- `list_files` / `glob` -> core directory walk and pattern match
- `search` -> core content search
- `write_file` -> core atomic or validated write
- `edit_file` -> core preconditioned edit
- `apply_patch` -> core patch validation and application
- `run_command` -> core process execution inside an approved sandbox profile
- `open_artifact` -> core artifact read with runtime session authorization

Arriving with their deferred packages: `git_status` and related VCS tools, `get_diff`, `detect_toolchain`, and index queries.

Runtime-owned throughout: `diagnostics`, `symbols`, and `references` are runtime LSP tools that talk to a language server the core launched as a sandboxed process. The core exposes no LSP methods.

Pure runtime tools — selecting a model, summarizing session state, changing settings, listing sessions — need no OS capability and stay in the runtime.

## Index architecture

Phase 1 has no index. Search runs directly against the filesystem under limits, which is sufficient for a local project and avoids building invalidation machinery before there is anything to invalidate.

When indexing arrives it should be tiered, and the tiers split across the boundary rather than all landing in the core:

- `file_catalog` — canonical paths, ignore policy, size, mtime, hashes, binary flags. **Core**, because it is filesystem bookkeeping tied to watchers.
- `content_search` — trigram or equivalent text structures. **Core**, because it is a bulk transformation of file bytes.
- `symbol_index` — definitions, references, document symbols, imports. **Runtime**, because extraction is per-language and churns constantly.
- `dependency_index` — manifests, module relationships, build and test entrypoints. **Runtime**, for the same reason.
- Embedding and vector indexes — **runtime**, since they require provider calls the core never makes.

An earlier draft placed all four tiers in the core on the grounds that an index is a privileged view of the repository. It is, but confidentiality is not what the boundary provides: the runtime can already read every file through core reads. What the boundary provides is auditability of machine effects, and a symbol index has none. Placing per-language extraction behind a hand-written protocol would make every new language a change in Rust and in two protocol implementations.

Index freshness must be explicit regardless of owner. Every result carries a source hash, document version, index epoch, freshness state, and invalidation reason when stale. An index result is advisory until reconciled through a core read or hash.

## Sandbox architecture

Sandboxing should be a first-class Rust service. A sandbox profile describes the filesystem view, writable paths, temp directories, environment variables, inherited credentials, network egress, process tree limits, and cleanup policy for an operation.

Initial profiles should be conservative and composable:

- `read_only_project`: read project files, no writes, no network, bounded process execution only when required.
- `project_write`: read and write approved project paths, no broad deletes, network denied by default.
- `command_no_network`: run approved commands in project with filtered environment and no network egress.
- `command_with_network`: run approved commands with explicit network approval and stronger audit.
- `language_server`: long-lived language server with project read access, controlled temp/cache paths, and bounded process/network policy.
- `untrusted_extension_host`: future profile for plugin or MCP host isolation, with minimal filesystem and network access.

Platform primitives differ across Windows, macOS, and Linux. The contract names desired capability classes and reports observed enforcement, rather than assuming one OS sandbox technology. An operation result states which profile was applied and what it enforced, so the runtime can explain why a command had the environment it received.

## Secret injection

Some processes need a credential — a token for a push, a registry credential for an install. The value must not travel in a protocol message body, because messages may be captured in diagnostic transcripts.

```text
runtime: request carries a scoped secret reference, not a value
core:    resolves the reference over a channel separate from the request body
core:    injects the value into the child environment at launch
core:    never logs it, never returns it, never echoes it in results
```

A sandbox profile declares which credentials a process may inherit; the default is none. The resolved value is confined to the launched process and never reaches the model.

## Checkpoint architecture

```text
before a turn's first mutation
        |
core captures pre-images of target paths -> artifact storage
        |
runtime anchors the checkpoint to a session content event
        |
turn proceeds, checkpoint extends as new paths are touched
        |
restore request -> authority check -> hash verification
        |
    hash matches capture -> restore
    hash differs         -> conflict for the user, never overwrite
```

Restore is an audited mutation like any other, never silent. It covers filesystem state only; the core makes no claim about external side effects such as a push or a published package, and the UI must say so.

## Diff architecture

Diff is a shared feature but not shared authority. Phase 1 scope is snapshot capture; hunk computation arrives with the diff package.

- The core captures trusted base and current content with hashes, binary detection, and line-ending metadata.
- Until the diff engine exists, the runtime computes presentation diffs from those snapshots.
- Whichever layer computes them, diffs for approval, for the client, and for model context all derive from the same core snapshots.
- Clients render from runtime API data only.

This avoids three layers each inventing their own view of what changed on disk.

## Language servers

There is no core LSP module. A language server is an ordinary process the core launches under the `language_server` sandbox profile; the runtime speaks LSP to it over the process handle.

```text
runtime LSP service
  document sync / diagnostics / definitions / refs / symbols / code actions
        |
core process operation (sandbox profile, limits, cleanup)
        |
language server process
```

The core owns process lifetime, environment filtering, resource limits, and cleanup — the parts that are machine effects. The runtime owns the protocol conversation and the agent-facing vocabulary, because that is where per-language churn lives.

An earlier draft made the core an LSP broker with `lsp.startServer`, `lsp.stopServer`, and `lsp.request` methods. That put the highest-churn subsystem behind the most rigid boundary: adding a language would mean changing Rust plus both hand-written protocol implementations. The runtime gains no filesystem access from holding the conversation, since the server's own access is bounded by the profile the core applied.

Formatting and code actions are the exception that proves the rule: if a server proposes a file mutation, the runtime applies it through the normal core mutation path with the same authority checks as any other write. The server never writes directly.

## Runtime restrictions

The runtime must not use Node filesystem or process APIs for privileged project operations. Allowed exceptions are narrow runtime-internal files outside the project authority path: its own SQLite database, runtime logs, the discovery file, and plugin package metadata. Even those need separate path and secret handling rules.

One deliberate exception is the language-server transport. The runtime holds the LSP conversation over a process handle the core created, which is not a filesystem or process API call and grants no additional access — the server's own reach is bounded by the sandbox profile the core applied.

Plugins and MCP servers must not receive core transport handles, project filesystem paths as authority, database handles, or unrestricted process privileges.

## Recovery model

Core recovery reports:

- Protocol and core version, plus supported capabilities.
- Journal entries with no terminal outcome, each with its pre-image reference so the runtime can reconcile by hash.
- Artifact inventory, so orphaned bytes and dangling references are both detectable.
- Checkpoint inventory and retention status.
- Process handles that survived or were cleaned up.

The runtime reconciles those signals against session content and the audit stream. The core does not decide how session history is written.

## Contract implications

The Phase 1 runtime-to-core method backlog:

```text
project.resolvePath
fs.readFile          fs.writeFile      fs.editFile
fs.applyPatch        fs.metadata       fs.hash
search.glob          search.find
sandbox.describeProfiles
process.start        process.cancel    process.getStatus
artifact.read        artifact.sweep
checkpoint.capture   checkpoint.restore
operation.cancel     operation.status  operation.recover
```

Roughly eighteen methods, down from about thirty in the earlier draft. The reduction came from removing the index, watch, diff, VCS, environment, and LSP families, and from collapsing `search.grep` and a separate glob-content search into one `search.find` with options.

Kept deliberately absent: `process.writeStdin`, because Phase 1 is non-interactive; and anything resembling `lsp.request`, per the language-server section.

Two channels, per `ARCHITECTURE.md` section 5.2. Requests, responses, and bulk results use stdio; heartbeats, `operation.cancel`, and progress notifications use the control channel so cancelling a runaway operation does not queue behind that operation's own output.

Names are illustrative. Canonical names are set during contract design, then hand-implemented on both sides against shared test vectors. Nothing is generated, so the vector suite is the only protection against the two implementations drifting.

## Risks

- **Boundary drift:** the runtime may reach for Node APIs for convenience. Add dependency and code-search checks before implementation.
- **Surface creep:** every added method costs audit coverage and two hand-written implementations. New operations need justification against the primitives-not-semantics rule, and the deferred packages must not quietly merge back into Phase 1.
- **Protocol drift:** with no code generation, only test vectors keep the two implementations aligned. An uncovered field is a silent divergence.
- **Language-server trust:** language servers are powerful external processes that read the project and sometimes spawn children. Treat them as process tools under a profile, not passive analyzers.
- **Diff inconsistency:** diffs for approvals, clients, and model context must come from the same core snapshots.
- **Unknown completion:** writes and process launches can complete while the runtime is down. The journal plus hash reconciliation replaces guessing; never retry automatically.
- **Artifact leakage:** bytes and references live on opposite sides of the boundary. Without disciplined mark-and-sweep, storage grows or references dangle.
- **Secret exposure:** a credential reaching a protocol message body would land in diagnostic transcripts. Keep resolution off the request path.
- **Performance pressure:** search and process streaming can be expensive. Limits and cancellation belong in the first contract, not as afterthoughts.
- **Sandbox portability:** Windows, macOS, and Linux expose different enforcement primitives. Specify capability outcomes and profile semantics before choosing platform implementations.

## Open questions

- What is the final Phase 1 method list, kept as small as the capability list allows?
- How durable must journal idempotency state be across core restart, and how long are pre-image references retained?
- Should the journal be a file or an embedded store, given the core must not open the runtime database?
- What is the mechanism for the scoped secret reference and its resolution channel?
- Which sandbox mechanisms are required per supported OS for command execution?
- What triggers each deferred package, and which arrives first?

