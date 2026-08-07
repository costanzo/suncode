# Requirement

## Background

Suncode's approved architecture makes Rust the audited execution point for machine-affecting work and TypeScript the agent runtime. The UI and runtime requirements depend on a clear definition of what belongs in the core.

The OS layer is not a second agent brain. It is the capability substrate: where project paths, filesystem state, process handles, sandbox rules, resource limits, and machine-affecting operations are validated and executed. TypeScript owns model-facing tool descriptions, context construction, session events, approval UX, provider calls, and orchestration.

## What the boundary is for

The core's value is not OS-enforced isolation. The runtime spawns it as a child under the same OS user and can bypass it with a direct filesystem call at any time. Per `ADR-20260807-rust-boundary-rationale`, the boundary provides three things instead: third-party code (plugins, MCP servers, npm dependencies) never touches the OS, every machine-affecting operation is one auditable path, and performance-sensitive filesystem work has a home.

This matters for scope. If the justification were isolation, everything touching the machine would belong here. Because it is auditability, **the surface must stay small** — audit coverage degrades as it grows, and every method is hand-implemented on both sides of the protocol.

So the core owns primitives, not semantics. High-churn semantic work — language servers, symbol extraction, dependency graphs, ranking — lives in TypeScript over core primitives. A language server is a *process launched through the core under a sandbox profile*; the runtime speaks LSP to it directly and gains no filesystem access by doing so. Adding language support is then a runtime change rather than a change to Rust plus two protocol implementations.

## Goals

- Define a deliberately small Phase 1 core operation catalog: path resolution, bounded reads, mutations, directory walk, content search, process execution, sandbox profiles, checkpoints, and artifacts.
- Keep all privileged project and OS operations behind typed core RPC methods.
- Prevent TypeScript, clients, plugins, MCP servers, and model providers from bypassing the core for machine-affecting operations.
- Specify the operation journal, idempotency, reconciliation, cancellation, resource limits, artifact collection, and secret delivery before implementation.
- Provide enough detail to write the runtime-to-core contract document and its test vectors.

## Non-goals

- Implementing core operations in this delivery.
- Moving provider adapters, context construction, agent-loop scheduling, SQLite, approval UX, or session history into the core.
- Exposing the core directly to any client, plugin, or MCP server.
- Making the core understand provider-specific tool-call formats or prompt semantics.
- Unrestricted filesystem or process access outside the authorized project.
- Cloud sandbox implementation and host isolation technology.
- Executing third-party plugins, MCP servers, or third-party provider adapters in Phase 1. Their future execution requires independent child processes, extension identity propagation, and a platform sandbox whose observed enforcement is reported by the core.
- Hosting language servers as a core-owned semantic service. The core launches them as sandboxed processes; the runtime owns the protocol conversation.
- Owning symbol, dependency, or embedding indexes.
- Generating protocol types. Each side implements the contract by hand.

## Ownership rule

When an operation reads, mutates, executes, or observes machine resources, the core executes it. TypeScript owns the model-facing tool, policy preflight, approval explanation, session event, and orchestration, and calls the core for the privileged step.

`read`, `write`, `edit`, `apply_patch`, `glob`, `grep`, `shell`, process management, file watching, artifacts, checkpoints, diff snapshots, and sandbox enforcement are core operations. Language-server orchestration, index ranking, and symbol resolution are not — they are runtime services built on core primitives.

## Phase 1 catalog

Phase 1 implements only:

- Path canonicalization and project-boundary enforcement
- Bounded file read, metadata, and hash
- Write, edit, patch, move, delete
- Directory walk and glob
- Content search
- Non-interactive process execution with output streaming
- Sandbox profiles for launched processes
- Checkpoint capture and restore
- Artifact storage, read, and swept deletion
- Operation cancellation, status, and recovery reporting

Deferred to their own requirement packages, each with an explicit trigger: filesystem watchers, a file catalog, content-search indexes, diff computation beyond snapshot capture, VCS-aware operations, PTY and interactive process support, and toolchain inspection. Every one of these was previously inside "Phase 1," which made the package impossible to complete.

Prefer fewer general operations over families of near-duplicates: one bounded search with options rather than several search methods.

## Requirements

### Layer boundary

- The core is a long-lived child process supervised by the Node.js runtime.
- The core is reachable only through private JSON-RPC over newline-delimited stdio.
- Every OS operation request must include runtime identity, project identity, session or operation correlation, idempotency key when mutating, declared capability, and bounded arguments.
- The core validates operation schemas independently of TypeScript validation.
- The core never receives provider credentials unless an OS operation explicitly requires a scoped secret reference.
- The core never opens the runtime SQLite database and never emits session events directly; it returns operation events/results for TypeScript to persist.
- The core logs to OS-layer logs or stderr, never to protocol stdout.

### Filesystem and path operations

- The core owns path canonicalization, project-boundary checks, symlink resolution policy, case-sensitivity handling, path normalization, file metadata, file hashing, and file content reads.
- The core rejects paths outside the authorized project unless a separately granted capability explicitly allows the access.
- The core handles platform-specific edge cases such as Windows drive letters, UNC paths, reserved names, junctions, macOS normalization, Linux permissions, and case-insensitive filesystems.
- The core exposes bounded read operations with size limits, binary detection, encoding detection, line ranges, and artifact fallback for large files.
- The core exposes write/edit operations through explicit mutation APIs with idempotency keys, expected preconditions, atomic-write behavior where available, and recoverable failure reporting.
- The core owns file locking or conflict detection required to prevent unsafe concurrent mutations.

### Search and glob

- The core owns glob expansion, directory walking, ignore-file application, binary-file filtering, hidden-file policy, search limits, and content search.
- Search results use canonical project-relative identifiers plus bounded preview snippets. A returned path is data, never authority for a later operation.
- The core enforces maximum directories visited, files scanned, bytes scanned, matches returned, preview size, and wall-clock duration.
- Results exceeding inline limits return an artifact reference or a pagination cursor. Large results never travel inline.
- TypeScript decides when search results enter agent context; the core decides what search may execute.

### Indexes and language services

- Phase 1 has no core-owned index. Search operates directly on the filesystem under limits.
- When indexing arrives, the core may own a file catalog — canonical paths, sizes, hashes, ignore state — because that is filesystem bookkeeping tied to its watchers. Symbol extraction, dependency graphs, and ranking stay in the runtime.
- Any index entry must carry provenance, source hash or document version, freshness state, sensitivity class, and invalidation reason when stale. An index result is advisory until reconciled by a core read or hash.
- Embedding and vector indexes are runtime provider features. The core does not store them without an explicit future contract.
- Language servers are launched as sandboxed core processes. The runtime speaks LSP to them over the process handle and owns document sync, request routing, and semantics. The core owns process lifetime, environment filtering, resource limits, and cleanup.
- Any file mutation a language server proposes — a code action, a formatting result — is applied through the normal core mutation path with the same authority checks as any other write. The runtime never lets the server write directly.

### Process and shell operations

- The core owns process spawning, shell execution, environment filtering, working-directory validation, stdin/stdout/stderr streaming, exit status collection, cancellation, and process-tree cleanup. PTY and interactive process support are deferred; Phase 1 is non-interactive execution only.
- The core enforces command allow/deny policy, project restrictions, resource limits, timeout, network/process capability class, and approval assertions.
- TypeScript may present an approval prompt and persist the decision, but the core validates the capability assertion before launching anything.
- Long-running processes must have stable operation handles, progress notifications, cancellation, heartbeat/health state, and cleanup behavior after runtime restart.
- Shell output is bounded; full output must move to a managed artifact when it exceeds inline limits.

### Sandbox service

- The core owns sandbox profile selection, sandbox materialization, filesystem view restriction, process namespace or container setup, environment filtering, network egress policy, temporary directory scope, credential exposure rules, and child-process inheritance rules.
- A sandbox is the execution envelope for machine-affecting work, not just a command flag.
- Sandbox policy applies to process execution and LSP hosting in Phase 1. Any future untrusted extension host must be an independent child process; worker threads are not an isolation boundary.
- TypeScript may request a sandbox class or profile, but the core validates whether that profile is available and whether the request fits the current capability grant.
- Sandbox decisions must be explicit in operation results so the runtime can explain why a command, server, or tool had the environment it received.

### Write, edit, and patch tools

- The core owns the final filesystem mutation for create, overwrite, append, edit, rename, delete, chmod-like metadata changes, and patch application.
- TypeScript may compute or request an intended edit, but the core validates target paths, base file hashes, expected versions, patch shape, line endings, encoding, and conflict state.
- Mutations must be idempotency-aware. Retrying an operation with the same key should return the original known result or an explicit unknown-completion state.
- Destructive operations require stricter capability classes and must never operate on broad roots, unresolved globs, or ambiguous targets.
- The core returns structured change summaries and artifact references; TypeScript records user-visible events and audit history.

### Artifact service

- The core owns artifact storage for large operation outputs, file snapshots, binary blobs, search captures, and command logs originating from OS operations.
- Artifact references are opaque IDs carrying owning session and tool call, sensitivity class, content type, size, hash, retention hint, and access policy. Model-visible references never expose filesystem paths.
- Bytes live in the core; references live in the runtime database. Because no transaction spans both, collection follows an explicit mark-and-sweep split:
  - The runtime is the mark authority. It alone determines what is unreachable, since only it can see session content.
  - The core is the sweep executor. It deletes only what the runtime names and reports what it deleted.
  - The core may refuse to delete an artifact pinned by an in-flight operation, reporting the refusal rather than failing silently.
  - The core never deletes on its own schedule. It enforces its storage limits by refusing new artifacts, not by reclaiming existing ones.
- On startup both inventories are compared. Bytes with no reference are sweep candidates; references with no bytes are marked unavailable so a client receives a typed error rather than hanging.

### Diff inputs

- Phase 1 scope is snapshot capture only: the core captures base and current content with hashes, binary detection, and line-ending metadata.
- Diff computation is deferred to its own requirement package. Until then the runtime computes presentation diffs from core-provided snapshots.
- Whichever layer computes it, diffs shown for approval, rendered in a client, and given to the model must derive from the same core snapshots. Three layers must not each form their own view of what changed.
- Diff requests are tied to stable file hashes or snapshot IDs so a stale view is detectable.
- Clients never compute a project diff from direct filesystem access.

### Process capability classes

- Package managers, compilers, formatters, test runners, build tools, VCS commands, and language servers are all process operations. They receive no dedicated methods in Phase 1; they are ordinary process execution under a sandbox profile.
- The core owns working-directory scope, environment filtering, cache and temp directory policy, network class, output limits, and cancellation for every launched process.
- The core enforces network policy for processes it launches: egress disabled, allowlisted, or permitted by explicit capability. Runtime and provider network calls are a separate concern.
- Destructive VCS operations such as reset, clean, checkout overwrite, rebase, and force push require a high-risk capability class before any tool may request them.
- MCP servers launched as local processes receive explicit network and filesystem capability classes like any other process.

### Operation journal and reconciliation

- The core maintains a private append-only operation journal, written before a mutation and completed after it, holding idempotency key, operation class, canonical target, pre-image reference, start time, and terminal outcome.
- This makes the core authoritative for "did the operation with this key complete?" A duplicate request returns the recorded outcome instead of re-executing.
- The journal is bookkeeping, not history. It is bounded and prunable, holds no conversation content, and is the only durable state the core owns.
- On startup the core reports every entry lacking a terminal outcome. The runtime resolves each by comparing the target's current hash against the pre-image and intended post-image, and treats "matches neither" as a conflict rather than retrying.
- The core never rewrites session history and never decides how the runtime records an outcome.

### Secret delivery

- The core never receives the master key and never opens the runtime database.
- Secret values must not appear in protocol message bodies, since messages may be captured in diagnostic transcripts.
- An operation needing a credential in a child process receives a scoped reference. The core resolves it through a channel separate from the request body and injects the value into the process environment at launch.
- A resolved secret is confined to the launched process. It is never logged, never echoed in results, and never returned to the model.
- Sandbox profiles declare which credentials a process may inherit; the default is none.

### Checkpoints

- The core captures a snapshot of files a turn is about to modify, before its first mutation, and extends it as new paths are touched.
- Restore is an audited operation subject to the same authority checks as any mutation. It is never silent.
- Restore verifies current hashes first. A file changed outside the agent is reported as a conflict for the user to resolve, never overwritten.
- Snapshots use artifact storage and its collection protocol, with bounded retention.
- Restore covers filesystem state only. The core makes no claim about external side effects.

### Capability enforcement and policy

- The core is the final authority for OS capability decisions.
- TypeScript policy preflight may return deny, allow, or approval required, but the core independently validates the asserted grant.
- The core evaluates canonical paths, operation class, resource scope, arguments, current project state, and OS limits before execution.
- Grants are scoped by operation class, canonical resource, argument restrictions, project/session, lifetime, and origin.
- The core returns typed denial, approval-required, conflict, invalid-scope, and resource-limit errors.
- Every security-sensitive operation must produce enough structured result data for TypeScript to persist an audit event without leaking secrets or sensitive content.

### Operation lifecycle

- Operations have stable IDs, idempotency keys for mutations, correlation IDs, start/end timestamps, status, progress, cancellation state, and safe error codes.
- Long-running operations support cancellation and must report whether cancellation is confirmed, best-effort, or unknown.
- The core must distinguish success, denial, validation failure, conflict, timeout, cancelled, crashed, core unavailable, and unknown completion.
- Unknown completion is never silently retried unless the operation contract declares retry safe.
- Core startup recovery must report open handles, interrupted operations, managed artifacts, and project state needed for TypeScript reconciliation.

### Resource limits and safety

- The core enforces limits for CPU time where practical, wall-clock time, memory, output bytes, filesystem bytes scanned, process count, child process lifetime, open files, artifact size, and concurrent operations.
- Limits are configurable by runtime policy but enforced by the core at execution.
- The core refuses unbounded recursive operations unless an explicit bounded plan is provided.
- The core redacts secrets from operation diagnostics where it can classify them and avoids logging file contents or command output by default.
- The core must not trust model text, UI labels, TypeScript-normalized paths, plugin metadata, MCP schemas, or provider responses as authority.

## Edge cases

- Symlink or junction points outside the project appear during a search or write.
- A file changes between context construction and patch application.
- A case-insensitive filesystem maps two user-provided paths to the same canonical resource.
- A watcher misses events and the runtime has stale context.
- A command produces unbounded output or spawns children after cancellation.
- The core exits after a write reaches the OS but before TypeScript receives the response.
- A language server hangs, indexes too much data, returns malformed JSON-RPC, or proposes a code action that edits outside the project.
- A diff request targets a binary file or a file with mixed encodings.
- A plugin or MCP tool attempts to invoke an OS operation through TypeScript with misleading metadata.
- A sandbox profile removes network, process, or filesystem access needed by an operation.
- A package manager needs network access while the current sandbox profile denies egress.
- The project directory is renamed, deleted, or unmounted while a session remains open.
- A duplicate request arrives with a known idempotency key whose journal entry has no terminal outcome.
- An artifact is named for sweeping while an in-flight operation still holds it.
- A restore is requested for a file that was modified outside the agent after checkpoint capture.
- An operation requests a scoped secret reference that has been rotated or revoked since it was issued.

## Acceptance criteria

1. The Phase 1 catalog is small enough to implement and audit, and every deferred capability has its own package with an explicit trigger.
2. Built-in OS-related tools are classified into core operations and runtime-owned model-facing tool definitions.
3. Path resolution, read, write/edit/patch, glob, search, process execution, sandbox, artifact, checkpoint, and journal responsibilities are explicitly assigned.
4. Language-server orchestration and index ranking are assigned to the runtime, with the core owning only process lifetime and file primitives.
5. The design prohibits direct Node.js filesystem and process APIs for privileged project operations in the runtime.
6. Capability enforcement requires core validation even after runtime policy preflight and user approval.
7. Mutating operations define idempotency, conflict detection, unknown completion, and hash-based reconciliation.
8. The operation journal makes duplicate-request and post-restart outcome queries answerable rather than assumed.
9. Artifact collection is a defined mark-and-sweep protocol with the runtime marking and the core sweeping.
10. Secret delivery to child processes is defined without placing secret values in protocol message bodies.
11. Checkpoint capture and restore are specified, including hash verification and conflict reporting.
12. Large outputs and snapshots use opaque artifact references rather than raw paths or unbounded payloads.
13. The runtime-to-core contract document and its test vectors can be written from the operation lifecycle, error classes, and capability scopes described here.
14. No requirement describes OS-layer product behavior as implemented.
15. Documentation validation passes for the requirement package.

## Open questions

Blocking:

- What is the exact method list for the Phase 1 catalog, kept as small as the capability list allows?
- How durable must journal idempotency state be across core restart, and how long are pre-image references retained?
- Which sandbox profiles are required at minimum for command execution on each supported OS, and what does each enforce where platform primitives differ?
- What is the mechanism for the scoped secret reference and its separate resolution channel?

Non-blocking:

- What triggers the watcher, file-catalog, content-search-index, diff-engine, VCS, PTY, and toolchain-inspection packages?
- Should the eventual diff engine use a Rust crate or a custom implementation?
- What are the default resource limits per operation class, and are they user-configurable?
- Should the operation journal be a file or an embedded store, given the core must not open the runtime database?
- What default allow/approval policy applies to read-only project operations?
- What artifact and checkpoint retention defaults should apply to command output, file snapshots, and large search results?

