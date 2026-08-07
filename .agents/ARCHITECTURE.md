# Suncode Architecture

**Status:** Approved

**Date:** 2026-08-07

**Scope:** Architectural boundaries, process topology, protocol foundation, authority model, durable state, and repository conventions

## 1. Purpose

Suncode is a local-first coding agent. This document defines its structure and the invariants every subsystem design must preserve. It does not describe implemented behavior; nothing here exists in source yet.

The architecture separates three concerns: presentation, agent orchestration, and machine-affecting execution. The split exists to keep the surface that can touch the user's filesystem small and auditable, and to keep the agent loop testable without a machine attached.

Design tradeoffs are resolved in favor of the product thesis in `PRODUCT.md`: local-first with no service dependency, reviewable authority, and one runtime behind several surfaces.

## 2. Architectural Principles

1. **Rust is the audited execution point for machine-affecting work.** Filesystem, process, and OS-capability operations execute there and nowhere else. It is a choke point for auditing and for containing third-party code — not an OS-enforced sandbox around the runtime. See section 3.4 for what it does and does not defend against.
2. **TypeScript owns agent behavior and durable state.** Model integration, context construction, the agent loop, tool orchestration, policy, SQLite, session state, and settings. Node.js only; Bun is prohibited.
3. **Clients are views, not applications.** Every surface consumes the same client API and holds no authoritative state. The desktop application must use Qt; Electron is prohibited.
4. **Contracts are written, not generated.** Protocol documents define messages, ordering, and errors. Each language implements its own types by hand. Conformance is proven by shared test vectors.
5. **Authority is explicit and reviewable.** Every machine-affecting operation carries a declared capability and a scope, and produces an audit record. No component can widen its own authority.
6. **Filesystem changes are reversible.** The agent's writes are checkpointed so a user can undo a turn's effect on disk.
7. **Non-interactive execution is a first-class mode.** Any operation requiring approval must be expressible as a pre-authorized policy, so the agent can run in a script or CI job.
8. **The local machine is not a trust boundary.** A client is authenticated because it presents a credential, never because it connected from localhost.

## 3. System Boundaries

```text
        CLI / TUI          Qt desktop
              \               /
               \             /
        authenticated local client API
              (HTTP + WebSocket, loopback)
                      |
            TypeScript agent runtime  ── SQLite (sessions, settings, secrets)
                      |
        private JSON-RPC over stdio (two channels)
                      |
              Rust OS core  ── operation journal
                      |
        filesystem / processes / project tree
```

### 3.1 Clients

The committed surfaces are CLI/TUI first, Qt desktop second. Clients own presentation, navigation, and local interaction state. They never open SQLite, never speak to a model provider, and never reach the Rust core.

Clients are not privileged by locality. Each connection authenticates with a runtime credential and is bound to the project and session scopes it is authorized for.

A client may hold a cached view of session content for responsiveness, but the runtime is authoritative. On reconnect the client resumes from its last applied sequence or refetches a snapshot.

### 3.2 TypeScript runtime

The runtime is the product's brain. It owns:

- Model-provider integrations and the model catalog
- Context construction, provenance, and compaction
- The agent loop and turn scheduling
- The tool registry and tool dispatch
- Policy evaluation, approval lifecycle, and audit records
- SQLite: schema, migrations, transactions, session content, settings, secrets
- Extension hosting: skills, plugins, MCP clients
- The client-facing API and event fan-out
- Supervision of the Rust child process

It does not perform privileged filesystem or process work directly. Narrow exceptions, all outside the project tree: its own database, its own logs, its discovery file, and plugin package metadata.

### 3.3 Rust OS core

The Rust core owns:

- Path canonicalization and project-boundary enforcement
- Bounded file reads, metadata, and hashes
- File mutations: write, edit, patch, move, delete
- Directory walking, glob, and content search
- Process and shell execution, including PTY handling
- Sandbox profile materialization for launched processes
- Checkpoint snapshots and restore
- Managed artifacts for large operation output
- A private operation journal for idempotency and crash reconciliation

Rust runs as one long-lived child of the runtime. Its interface is deliberately narrow and low-level; see section 9.2 for what is deliberately excluded from it and why.

### 3.4 What the Rust boundary defends against

Stating this precisely matters, because later designs will lean on it.

The runtime process spawns Rust as a child under the same OS user. It can call `fs.writeFileSync` at any time. **Rust is therefore not an OS-enforced sandbox around the runtime, and a compromised runtime is not contained by it.**

What the boundary does provide:

- **Containment of explicitly isolated third-party code.** Future plugins, MCP servers, and third-party provider adapters run outside the runtime in independent child processes and receive OS access only through a Rust-mediated sandbox. The TypeScript runtime and its built-in provider adapters remain trusted in Phase 1; Rust does not contain their npm dependency tree.
- **One auditable choke point.** Every machine-affecting operation is one code path, so audit coverage and policy enforcement can be verified by inspecting a small surface.
- **Enforced discipline.** The runtime cannot casually reach for Node filesystem APIs; the layer split makes bypasses visible in review and in dependency checks.
- **Performance headroom** for search, walking, and hashing.

What it does not provide: protection from a compromised or malicious runtime process. Confining the runtime itself is an open question recorded in section 13, not a solved problem.

Two consequences follow. First, principle 5 is about reviewability, not about a hard isolation guarantee. Second, because the boundary's value is auditability rather than isolation, the Rust surface stays narrow — putting high-churn semantic logic behind a rigid process boundary costs stability without buying containment.

### 3.5 Dependency rules

- Clients depend on the client protocol and presentation libraries only.
- The runtime depends on its protocol implementations and provider SDKs.
- Rust crates depend on internal abstractions and its protocol implementation.
- No client imports runtime or Rust internals.
- Only the runtime opens the database.
- No Rust crate contains provider or presentation logic.
- Phase 1 has no executable third-party extensions. Future extensions receive no database connection, master key, Rust transport handles, or unrestricted environment; their requests carry extension identity and are re-authorized at both runtime and core boundaries.

These are checked by automated dependency and import rules, not by convention.

## 4. Process Topology and Lifecycle

### 4.1 Runtime instance

One runtime per OS user, started on demand:

1. A launcher acquires a per-user single-instance lock, or attaches to the healthy runtime already holding it.
2. The runtime generates a random credential valid for its lifetime and binds an authenticated loopback endpoint.
3. It writes a discovery file readable only by the current user, containing the endpoint, credential, and its build version.
4. It opens SQLite, runs migrations, and starts the Rust child.
5. It completes the protocol handshake and reconciles any interrupted operations before accepting client requests.
6. It exits after a configurable idle period, once no client, turn, operation, or background job is active.

### 4.2 Version skew

A shared long-lived runtime creates a failure mode worth naming: the on-disk CLI is upgraded while an older runtime is still resident, so the user runs a new client against an old brain.

The rule: a client refuses to attach to a runtime whose build version it does not recognize as compatible. On mismatch the client asks the resident runtime to drain — finish active turns, reject new ones, exit — then starts the current version. If draining exceeds a grace period, the client reports the conflict rather than forcing a kill, because active turns may hold uncommitted work.

The discovery file carries the version so a client detects skew before opening a session, not after.

### 4.3 Single-process mode

The CLI must be able to run without a resident runtime. In single-process mode it hosts the runtime in-process, uses a private temporary database or an explicitly named one, runs one session, and exits.

This is required for CI and scripted use, and it also gives the daemon path a fallback when the lock is unavailable. The runtime is therefore structured as a library with a server wrapper around it, never as a server with logic embedded in its request handlers.

### 4.4 Concurrency

Independent sessions run concurrently. Within one session, one turn executes at a time. Within one turn, tool calls are scheduled sequentially in Phase 1 — a policy choice, not a structural limit, since the state model in section 8 already expresses concurrent calls.

Rust serializes or rejects conflicting mutations to the same resource regardless of what the runtime schedules.

### 4.5 Client authentication

Loopback binding is not authentication. Every connection presents the runtime credential, obtained from the user-readable discovery file. The credential never appears in a URL or a log. Browser-originated connections, if a web surface is ever built, require origin validation; that is out of current scope.

Authentication establishes that a caller is this user. Authorization separately binds each connection to permitted project and session scopes.

### 4.6 Layered logs

Each layer writes its own file under the platform's application-data directory:

- Client: interaction, connection, and rendering.
- Runtime: lifecycle, provider calls, orchestration, database, protocol.
- OS core: operations, sandbox, process, filesystem.

Every record carries a timestamp, severity, and correlation identifiers where safe. Logs redact credentials, tokens, and file contents. Rust protocol stdout is never a log target. Logs are never committed.

## 5. Protocols

Two boundaries, two independent contracts: client-to-runtime, and runtime-to-core.

### 5.1 Contracts without code generation

Each contract is a written document defining methods, payload shapes, ordering guarantees, and error codes. Each side implements its own types and validation by hand.

This is a deliberate reversal of the earlier contract-first-with-generation plan (ADR-20260807-hand-written-protocol-contracts). The consequence is that nothing mechanically prevents the two implementations from drifting, so drift must be caught by tests:

- **Shared test vectors.** A directory of recorded messages — valid and invalid — that both implementations run against. Each vector states whether it must be accepted or rejected and, for valid messages, the fields that must be extracted. This is the enforcement mechanism, so it is mandatory for every contract change, not optional coverage.
- **Round-trip tests.** Each side encodes and decodes its own types; sample messages survive both directions unchanged.
- **A contract change checklist.** Adding a field means: update the document, add vectors, hand-edit both implementations, note it in the compatibility section.

The client-to-runtime contract carries the same rule and the same vector suite. Because there is no generated SDK, each surface hand-writes its transport adapter against the document.

### 5.2 Runtime-to-core transport

JSON-RPC 2.0 over the Rust child's stdio, UTF-8, newline-delimited, one message per line.

**Two channels.** A single pipe head-of-line blocks: a large search result or diff serializing ahead of a heartbeat delays the heartbeat, which makes health detection unreliable exactly when the core is busy. So:

- **stdio** carries requests, responses, and bulk results.
- **A separate control channel** carries heartbeats, cancellation, and progress notifications. On Unix this is an extra file descriptor; on Windows a named pipe. The handshake establishes it.

Cancellation on the control channel matters most: cancelling a runaway operation must not queue behind that operation's own output.

Rules on both channels:

- Rust stdout is reserved for protocol messages; diagnostics go to its log file and stderr.
- Both sides enforce a negotiated maximum message size.
- Protocol corruption terminates the child. Neither side attempts stream resynchronization.
- **Large payloads never travel inline.** Results exceeding the inline threshold return an artifact reference or a pagination cursor. This is a hard requirement, not a preference — it is what keeps the size limit from being hit in normal operation.

### 5.3 Handshake

Before normal traffic the peers exchange protocol version and supported range, build versions, available methods and optional capabilities, OS and architecture, maximum message size, control-channel details, and recovery state.

Incompatible versions produce a typed initialization failure. Optional behavior is capability-negotiated, never inferred from build version.

### 5.4 Message conventions

- Request identifiers are unique within a runtime lifetime.
- Session-scoped requests carry a session identifier; tool-originated requests also carry a tool-call identifier.
- Every mutating operation carries an idempotency key.
- Long-running requests support cancellation and report whether it was confirmed, best-effort, or unknown.
- Unknown fields are tolerated within a compatible version; missing required fields fail validation.
- Errors use stable machine-readable codes, a safe human-readable message, and optional typed data.

### 5.5 Health and recovery

Heartbeats on the control channel detect an unresponsive child without waiting on bulk traffic.

If Rust exits:

1. Outstanding calls fail with `core_unavailable`.
2. The runtime restarts it with bounded exponential backoff.
3. Both sides reconcile operation outcomes per section 6.3.
4. Operations with unknown completion are not retried unless their contract declares retry safe.
5. Repeated startup failure leaves the runtime in a diagnostic state with execution disabled.

Clients reconnect with their last applied sequence. The runtime resumes after it, or returns `resume_unavailable` when retention no longer covers it.

## 6. Crossing the Process Boundary

Three problems arise from executing in one process and recording in another. Each needs a protocol, not a case-by-case rule.

### 6.1 The atomicity gap

Rust performs a write; the runtime records it. There is no transaction spanning both. If the pipe drops between the two, the write happened and the record does not exist.

This is not an edge case — it is the inherent third outcome of every mutating operation. Rather than making every caller handle three branches, the boundary resolves it:

**Rust keeps an operation journal.** A small append-only record, private to the core, holding for each mutating operation: idempotency key, operation class, canonical target, a pre-image reference where one was captured, start time, and terminal outcome once known. It is written before the mutation and completed after.

This makes Rust authoritative for "did the operation with this key complete?" — replacing a guess with a lookup. A duplicate request returns the original outcome instead of re-executing.

The journal is bookkeeping, not history. It is bounded, prunable, and holds no conversation content. It is the one piece of durable state Rust owns, which is why current documents do not say "Rust owns no persistence."

### 6.2 Reconciliation

When Rust restarts, it reports every journal entry lacking a terminal outcome. For each, the runtime resolves the truth by observation rather than assumption:

1. Ask Rust to read the target's current hash.
2. Compare against the pre-image and the intended post-image.
3. Matches post-image: the operation completed. Record the result.
4. Matches pre-image: it did not happen. Safe to retry.
5. Matches neither: something else changed the file. Surface it as a conflict; never retry silently.

The outcome becomes a durable audit record either way, including "unresolvable."

### 6.3 Ordering

For a mutating operation the sequence is: runtime records intent → Rust journals and executes → Rust returns outcome → runtime records result. Intent is durable before execution, so a crash always leaves a recoverable trace.

## 7. Durable State

### 7.1 Three streams, not one

A single journal previously served audit, conversation, client sync, and recovery. Those consumers want opposite things: audit wants immutability and long life, conversation wants compaction, sync wants only the recent tail, recovery wants bounded size. Any retention rule for one damaged another — and "projections are deterministically rebuildable from the journal" is false the moment the journal is compacted.

So durable state is three streams with independent lifetimes:

**Audit log — immutable, long-lived, never rewritten.** Authority decisions only: capability requested, scope evaluated, decision and its source, grant lifetime, operation outcome. Small, because it holds no content. Retention is a user setting with a long default. Compaction never touches it. This is what makes "reviewable authority" verifiable after the fact.

**Session content — compactable.** Messages, tool calls and results, turn transitions, context-build summaries, artifact references. This is the agent's memory and the rebuild source for projections. Compaction rewrites it under the rules in section 7.3.

**Client sync state — disposable.** Per-client cursors and delivery bookkeeping. Recreatable from session content; never a source of truth; discardable at any time.

Cross-stream references use explicit correlation identifiers. There is no sequence number shared across streams.

### 7.2 Projections

The runtime maintains relational projections over session content for current-state queries. Appending content and updating affected projections happens in one transaction.

Projections are disposable and rebuildable from session content — the claim now holds, because the audit log is not their source and compaction is defined as a content rewrite that preserves rebuildability. Clients query the runtime API; they never read SQLite.

### 7.3 Compaction

Compaction replaces older session content with a structured summary. It must preserve: the current objective, unresolved approvals, active tool state, checkpoint anchors, exact error identifiers and file paths, and a recent uncompacted tail.

Compaction is transactional with the event recording it, validated for structure before it replaces anything, and never applied to the audit log.

### 7.4 Database and location

The runtime exclusively owns the SQLite connection, schema, migrations, and integrity checks. The database lives in the platform per-user application-data directory. Two runtimes must never write the same database; the single-instance lock enforces this and single-process mode uses a separate database.

### 7.5 Secrets at rest

The runtime owns key generation and encryption. The master key lives in the OS credential store — Keychain, DPAPI, or Secret Service — never in SQLite. Rust does not open the database.

Classified secrets are encrypted: provider API keys, access tokens, refresh tokens. Session messages and ordinary settings are plaintext protected by file permissions unless a later data-classification design widens coverage.

Encrypted values use authenticated encryption with a unique nonce per value; SQLite stores ciphertext, nonce, algorithm identifier, and key version. Associated data binds ciphertext to record type, record identity, and field identity. Decryption failure is a typed integrity error and never falls back to plaintext.

Rotation re-encrypts in resumable batches with durable progress. Backups hold ciphertext and are not portable to another machine without an explicit key transfer, which is deferred.

### 7.6 Secret delivery to operations

Some operations need a credential in a child process — a token for `git push`, a registry credential for a package install. The runtime holds decrypted secrets; Rust launches the process. How the value crosses matters.

The rule: **secret values never appear in protocol message bodies.** Protocol messages carry only opaque secret handles. Payloads are logged, included in diagnostic transcripts, and retained in test vectors; a value placed there leaks by design.

Redemption of a handle uses a channel outside the JSON-RPC message stream, and Rust injects the value directly into the child process environment without materializing it in its own logs, its journal, or any artifact. A handle names one secret, one operation, and expires with it.

Consequence for diagnostics: the deterministic request transcript required for provider debugging records handles, never values.

### 7.7 Artifact ownership and collection

Artifact bytes live in Rust; references live in the runtime's database. Neither side can safely reclaim alone: Rust deleting on its own retention leaves dangling references, and the runtime deleting a session leaves orphaned bytes.

The protocol is mark-and-sweep with split roles:

- **The runtime is the mark authority.** It knows which references are still reachable from session content, and it alone decides what is garbage.
- **Rust is the sweep executor.** It deletes only what the runtime names, and it reports what it deleted.
- Rust may refuse to delete an artifact pinned by an in-flight operation, reporting the refusal rather than failing silently.
- Rust never deletes on its own schedule. Its own limits are enforced by refusing new artifacts, not by reclaiming old ones.
- Startup reconciliation compares both inventories: bytes with no reference are sweep candidates; references with no bytes are marked unavailable so a client gets a typed error rather than a hang.

Artifact references are opaque identifiers carrying owning session, sensitivity class, content type, size, hash, and retention hint. Model-visible references never expose filesystem paths.

## 8. The Agent Loop

### 8.1 Two levels, not one sequence

A turn is not a linear path. Every current major provider can return several tool calls in one assistant message, and those calls resolve independently — one may need approval, one may time out, one may succeed. A single sequence through `awaiting_tool → executing_tool` cannot represent that state.

So a turn is a two-level machine.

**Turn level:**

```text
admitted -> queued -> preparing -> calling_model
                                        |
                    +-------------------+-------------------+
                    |                   |                   |
              resolving_calls      compacting        (terminal)
                    |                   |
                    +--------> calling_model <-------+

terminal: completed | failed | cancelled | interrupted
```

**Tool-call level.** Each call requested by the model gets its own machine, identified independently of the turn:

```text
requested -> validating -> policy_check
                              |
        +---------------------+----------------+
        |                     |                |
      denied          awaiting_approval     authorized
                              |                |
                    approved / refused / expired
                                               |
                                          executing
                                               |
        +--------------+--------------+---------------+
        |              |              |               |
    succeeded       failed        timed_out    unknown_completion
                                                     |
                                              reconciling -> resolved | unresolvable
```

The turn leaves `resolving_calls` when every child reaches a terminal state. Results are appended to context and the loop returns to `calling_model`.

This is what makes the identifier requirement real: a tool-call identifier is distinct from the turn identifier from the start. Phase 1 still executes children one at a time by scheduling policy, so enabling concurrency later is a policy change rather than a redesign.

### 8.2 Approval precedes execution

`policy_check` and `awaiting_approval` both sit strictly before `executing`. There is no path that executes and then asks.

An earlier draft listed `executing_tool -> awaiting_approval`, which contradicted the authority model. That ordering is void; if any document still implies it, this section governs.

### 8.3 Input admission and delivery

User input is recorded durably before any scheduling. Admission and promotion are separate events, so a client never has to infer what happened to its input.

Delivery modes:

- **`queue`** — wait for the current turn to reach an idle boundary.
- **`steer`** — incorporate at the next safe boundary, which is between provider calls, never mid-tool-call.
- **`cancel-and-replace`** — cancel the current turn, then promote once cancellation settles.

The wake signal is advisory. After a restart the runtime drains admitted input from SQLite; an in-memory queue is never authoritative. Repeated wakes coalesce without dropping durable input.

### 8.4 Frozen turn snapshot

Before each provider call the runtime freezes: provider and model with capabilities and limits, agent profile and budgets, context epoch, the exact tool definitions advertised with their implementation identities, the policy and grant snapshot, resolved skill/plugin/MCP versions, and prompt-cache policy.

A tool call settles against the snapshot it was shown. A tool that was replaced or reconfigured since fails as stale rather than running a different implementation under a familiar name. Configuration and extension reloads apply at the next safe boundary.

### 8.5 Budgets and stalls

Enforced per turn: iteration count, wall-clock duration, model tokens, tool calls, and total output bytes. Exceeding one is a typed terminal failure with partial work preserved.

Repeated near-identical tool calls are detected as a stall and either surfaced for user decision or terminated. Budgets alone do not catch a loop that stays under them.

### 8.6 Context layers

Context is an ordered list of typed parts, each carrying source, trust class, size, sensitivity, and compaction priority — not one opaque string:

1. Runtime safety and policy instructions
2. Suncode system instructions
3. Project and session policy
4. Selected skill instructions
5. Approved plugin and MCP contributions
6. Project metadata and user-selected files
7. Compacted summary and recent session content
8. Current user input and attachments
9. Tool results and unresolved approval state

A higher-trust part may constrain a lower-trust part; never the reverse. Model text is never an authorization decision, and project instruction files are untrusted content until a trust policy says otherwise. The engine can explain every inclusion and omission without revealing secrets.

A context epoch identifies the resolved source set at a boundary. Source changes create a new epoch and emit an event; they never retroactively reinterpret earlier turns.

## 9. Authority

### 9.1 Two stages

**Runtime preflight** decides whether the operation is known, in scope, declared by its tool, covered by a grant, or needs approval. It is a pure function of request plus policy snapshot — no decisions from model wording or unverified path strings.

**Core enforcement** independently validates the capability assertion, canonicalizes paths without trusting the caller's normalization, checks project scope and limits, and returns authorization, denial, or a typed approval requirement.

The runtime cannot manufacture a grant; it submits a user decision through a dedicated authorization call. Rust validates the resulting assertion and applies it only to the named operation. Both stages emit audit records.

Preflight also filters which tools the model sees. Catalog filtering and execution authorization are separate controls — hiding a tool is a context decision, authorizing a call is a security decision, and neither substitutes for the other.

### 9.2 Keeping the core surface narrow

The core's value is auditability, and auditability degrades as its surface grows. Two rules follow.

**Rust owns primitives, not semantics.** It executes canonical path resolution, bounded reads, mutations, directory walks, content search, process launch, sandbox materialization, checkpoints, and artifacts. It does not own language-server orchestration, index ranking, symbol resolution, or dependency analysis.

The reason is churn against rigidity. Process boundaries are rigid; language and ecosystem support changes constantly. Putting language-server orchestration behind the core means every new language touches Rust and both protocol implementations. So the semantic work lives in TypeScript over core primitives: the runtime hosts and speaks to language servers as *processes launched through the core under a sandbox profile*, and reads files through core reads. It gains no filesystem access by doing so, and adding a language is a runtime change.

The same applies to indexing. Rust may hold a file catalog — canonical paths, sizes, hashes, ignore state — because that is filesystem bookkeeping tied to its watchers. Symbol extraction, dependency graphs, and ranking are runtime concerns built on core reads and searches.

**Prefer fewer, more general operations.** One bounded search with options beats a family of near-duplicate search methods. A smaller surface is easier to audit and cheaper to keep consistent between two hand-written implementations.

### 9.3 Grants

Scopes: one operation, session, project, or explicitly persistent. Each grant is constrained by operation class, canonical resource scope, argument restrictions, lifetime, and origin.

Separately classified capabilities: network access, process execution, secret use, writes outside the project tree, destructive operations, and external MCP calls. These are independent, so approving a write never implies approving execution.

Grants are stored by the runtime and enforced by Rust at execution. Policy changes affect future calls only, unless an explicit revocation rule applies. Concurrent or duplicate approvals resolve idempotently and notify all connected clients.

### 9.4 Non-interactive authorization

Approval-gated operations must be expressible as policy, or the agent cannot run unattended.

A **policy profile** is a named, declared set of pre-authorized capabilities with their scopes. Running non-interactively selects a profile; anything outside it fails immediately with a typed error rather than blocking on a prompt that no one will answer.

Profiles are ordinary policy, subject to the same enforcement and audit as interactive grants — the audit record notes that authorization came from a profile rather than a person. There is no mode that disables enforcement, and no profile can grant a capability the user has not declared.

### 9.5 Third-party code

- A **skill** is data and instructions. It gains no authority by being loaded.
- A **plugin** is future third-party code. It does not run in a worker thread as a security boundary; it runs in an independent child process under a Rust-mediated platform sandbox with declared capabilities.
- An **MCP server** is an external principal and is deferred from Phase 1. When enabled, it follows the same child-process and sandbox rule unless a separately reviewed transport has equivalent enforcement.
- A **third-party provider adapter** follows the extension rule. Built-in provider adapters remain trusted runtime code in Phase 1.

Phase 1 executes no plugins, MCP servers, or third-party provider adapters. Future extension-originated tool calls carry extension identity and pass through schema validation, policy, approval, timeout, output bounds, secret handling, and audit at both runtime and core boundaries.

## 10. Reversibility

Undoing what the agent did to disk is a product commitment, and it cannot be added later — it requires a snapshot authority spanning session state and the filesystem, plus interaction with diffs, artifacts, and watchers.

A **checkpoint** is a Rust-captured snapshot of the files a turn is about to touch, anchored to a session content event. It is created before the turn's first mutation and extended as new paths are touched.

Rules:

- Checkpoint capture is a core operation; the runtime records the anchor.
- Restore is a core operation subject to the same authority and audit as any mutation. It is never silent.
- Restore verifies current hashes first. If a file changed outside the agent, restoring it is a conflict the user resolves, not an overwrite.
- Restore covers agent-caused filesystem changes only. It does not undo external side effects — a pushed commit, a published package, a sent request. The UI must say so.
- Checkpoints have bounded retention, and expiry is visible before a user relies on it.
- Snapshot storage uses the artifact mechanism and its collection protocol.

Checkpoints are not a version-control replacement, and Suncode does not create commits on the user's behalf.

## 11. Configuration

### 11.1 Layered resolution

An earlier principle said users do not edit configuration files and all settings are interface-managed. That was inconsistent — skills and plugins were still discovered from project directories — and it broke two real needs: sharing agent setup with a team through the repository, and overriding settings in a script.

Settings resolve in precedence order, lowest first:

1. **Built-in defaults.**
2. **Project file** — committed to the repository, shared by the team. Declares skills, MCP servers, model preferences, and policy profiles. **Treated as untrusted content**: it can express intent but cannot grant authority, so a policy profile it declares still requires the user's activation.
3. **User settings** — interface-managed, stored in SQLite. Where secrets live.
4. **Environment variables and command-line flags** — highest precedence, for scripted and CI use.

Secrets appear in layer 3 only. A project file naming a credential names a reference, never a value.

The runtime exposes effective values with their originating layer, so a surprising setting is traceable.

### 11.2 Trust in configuration

Reading a project file must never itself confer authority — a repository is often untrusted code. Layer 2 supplies preferences and declarations; anything security-relevant needs confirmation recorded in layer 3. Opening a hostile repository cannot silently widen what the agent may do.

## 12. Repository Layout

```text
suncode/
├── contracts/
│   ├── client-runtime/      # protocol documents
│   ├── runtime-core/        # protocol documents
│   └── vectors/             # shared conformance test vectors
├── rust/
│   ├── Cargo.toml
│   └── crates/
├── typescript/
│   └── packages/
│       ├── agent-runtime/   # library: loop, providers, context, policy, storage
│       ├── runtime-server/  # thin server wrapper
│       └── core-client/     # runtime-side protocol implementation
├── apps/
│   ├── cli/                 # first surface, includes TUI
│   └── desktop-qt/          # second surface
├── tooling/
├── docs/
└── .github/workflows/
```

There is no `generated/` directory — nothing is generated. Client surfaces hand-write their transport adapters, so there is no shared SDK package either.

This expresses intended ownership. Directories are created when a milestone needs real buildable files; empty modules mirroring the diagram are not.

Toolchains: `rust-toolchain.toml` pins Rust; the root Node manifest declares the Node range and pins pnpm via `packageManager`. Cargo and pnpm stay separate native workspaces. Root commands are cross-platform, not Make or Unix shell.

## 13. Verification

### 13.1 Contract conformance

Because contracts are not generated, test vectors carry the whole burden:

- Both implementations run every vector and agree on accept or reject.
- Valid vectors assert the fields each side must extract.
- Coverage includes handshake and version negotiation, requests and responses, notifications, progress, cancellation, typed errors, malformed JSON, invalid envelopes, missing and unknown fields, oversized messages, unexpected EOF, and incompatible versions.
- Every contract change adds vectors in the same change.

### 13.2 Transport

Complete frames, partial reads, several frames per read, escaped newlines, malformed UTF-8, size limits, closed streams, control-channel independence under bulk load, stderr separation, and child termination.

### 13.3 Repository

Cross-layer import rules, formatting and linting for Rust and TypeScript, clean builds, documentation links, and a clean worktree after full verification.

### 13.4 Behavioral quality

Correctness of the harness does not make the agent useful. What determines that is instruction design, tool descriptions, and edit reliability — none of which the checks above measure.

A fixed task suite with tracked pass rate and token cost is therefore part of the architecture, not a later addition. It is the regression net for prompt and tool-description changes, which are otherwise unverifiable and easy to regress. It should exist as soon as one tool call works end to end.

### 13.5 Platforms

CI runs Linux, Windows, and macOS. Process creation, stdio behavior, path handling, application-data locations, filesystem semantics, and credential stores differ materially, and path canonicalization is where that surfaces first.

## 14. Scope

### 14.1 Foundation milestone

Included: architecture documentation and decision records, repository and build-workspace foundations, toolchain manifests, cross-platform root commands, formatting and lint configuration, CI, protocol documents with initial vectors, and minimal compile-only packages with protocol adapters.

Excluded: provider integrations, functional agent loop, functional operations, SQLite repositories, encryption, product surfaces, packaging, parallel tool execution, and executable plugins/MCP/third-party provider adapters. Enabling third-party extensions requires a separate approved extension trust-boundary delivery covering child-process IPC, platform sandbox enforcement, extension identity propagation, secret handoff, lifecycle, and failure recovery.

Complete when a fresh checkout bootstraps from documented prerequisites, one root command verifies formatting, linting, contract vectors, builds, and tests, vectors pass against both implementations, CI passes on three platforms, dependency rules are automatically enforced, and no excluded behavior exists.

### 14.2 Deferred designs

1. Client-runtime protocol document and credential handoff
2. Core supervision and protocol implementation
3. SQLite schema, migrations, credential store, encryption algorithm, rotation
4. Session content taxonomy, retention, projections, replay
5. Policy model, capability classes, sandbox profiles, conflict control
6. Core operation catalog
7. Agent loop, provider abstraction, context management, approvals
8. Checkpoint capture and restore
9. CLI/TUI surface
10. Qt desktop surface
11. Behavioral evaluation suite
12. Packaging, updates, signing

Each must preserve the boundaries and invariants here unless a superseding decision record changes them.
