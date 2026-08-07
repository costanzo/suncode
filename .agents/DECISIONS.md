# Decision Index

Newest first. A superseded decision keeps its entry so the reasoning stays readable.
## ADR-20260807-trusted-runtime-extension-isolation

- Date: 2026-08-07
- Status: Accepted
- Context: Treating all npm dependencies and extensions as isolated while they run in the TypeScript runtime overstates the security boundary. A Node worker thread is not an OS isolation boundary, and a child process without an OS sandbox still runs with the user's authority. The product needs a small, implementable Phase 1 before adding the substantially more complex extension isolation and secret handoff machinery.
- Decision: Phase 1 trusts the TypeScript runtime and its built-in provider adapters as one runtime component. Phase 1 excludes plugins, MCP servers, and third-party provider adapters from execution. Skills are data and instructions only. When third-party extensions are introduced, they must run in independent child processes and through a Rust-mediated, platform-specific OS sandbox with explicitly reported enforcement; worker threads alone are never considered isolation. Extension-originated requests carry an extension identity and are re-authorized at the runtime and Rust boundaries.
- Consequences: The architecture no longer claims that the runtime's npm dependency tree is contained by Rust. Extension IPC, sandbox profiles, scoped secret delivery, lifecycle, and failure recovery become a separate prerequisite before third-party extensions are enabled. Built-in providers may remain in-process, while third-party providers follow the extension rule. The trusted runtime remains an explicit threat-model assumption, not an OS-enforced sandbox.
- Details: `ARCHITECTURE.md` sections 3.4, 9.5, and 14.1; `requirements/2026-08-07-extension-trust-boundary/`

## ADR-20260807-durable-stream-separation

- Date: 2026-08-07
- Status: Accepted
- Context: A single append-only journal was serving audit, conversation content, client synchronization, and crash recovery. Those consumers have contradictory retention needs: audit wants immutability and long life, conversation wants compaction, client sync wants only the recent tail, recovery wants bounded size. Any retention rule written for one damaged another, and the requirement that projections be deterministically rebuildable from the journal conflicted with the requirement that the journal be compactable.
- Decision: Split durable runtime state into three streams with independent lifetimes: an immutable audit log of authority decisions, a compactable session content store holding messages and tool results, and a disposable client synchronization cursor. Session content is the rebuild source for projections; audit is never rewritten; sync state is recreatable from content and never a source of truth.
- Consequences: Retention, compaction, and export are specified per stream. Compaction can rewrite conversation history without touching the audit record. "Deterministically rebuildable" applies only to projections over the content store, not to the audit log. Anything crossing streams needs an explicit correlation identifier rather than a shared sequence.
- Details: `ARCHITECTURE.md` section 7

## ADR-20260807-tool-call-state-machine

- Date: 2026-08-07
- Status: Accepted
- Context: The Phase 1 turn state machine was a single linear sequence through `awaiting_tool` and `executing_tool`. Every current major provider can return several tool calls in one assistant message, which that shape cannot represent: it has no state for three concurrent calls where one needs approval, one has timed out, and one has finished. The same sequence also placed `executing_tool` before `awaiting_approval`, contradicting the authority model.
- Decision: Model a turn as a two-level machine. The turn-level machine tracks conversation progress; each tool call gets an independent child machine with its own lifecycle, and the turn state is a function of its children. Approval always precedes execution.
- Consequences: Multiple tool calls per assistant message work in Phase 1. Per-call cancellation, timeout, and unknown-completion are expressible without special cases. Phase 1 still executes child calls sequentially by policy, so concurrency remains a scheduling change rather than a state-model change. Event payloads carry a tool-call identifier distinct from the turn identifier.
- Details: `ARCHITECTURE.md` section 8, `requirements/2026-08-07-agent-runtime-phase-1/`

## ADR-20260807-hand-written-protocol-contracts

- Date: 2026-08-07
- Status: Accepted
- Supersedes: the code-generation portion of ADR-20260804-foundational-architecture
- Context: The foundation required JSON Schema and OpenRPC as canonical sources with deterministic generated Rust and TypeScript types, plus fixture equivalence across every language binding. With two protocol boundaries and several planned client languages, maintaining generators and cross-language fixture matrices was set to cost more than the protocol implementations themselves, before any product behavior existed.
- Decision: Protocol contracts are prose and schema documents that define messages, ordering, and error semantics. Each language implements its own types and validation by hand. No generator, and no generated-artifact drift check in CI.
- Consequences: Contract documents are the human-readable source of truth but are not machine-enforced. Conformance is verified by shared test vectors — recorded message samples both sides must accept or reject — instead of by generated types. Adding a protocol field is a documented change plus a hand edit in each implementation. Divergence risk moves from generator correctness to test coverage, so the vector suite is mandatory rather than optional.
- Details: `ARCHITECTURE.md` sections 5 and 11

## ADR-20260807-local-first-scope

- Date: 2026-08-07
- Status: Accepted
- Amends: ADR-20260804-foundational-architecture
- Context: The foundation treated local and cloud-hosted execution as equally weighted deployment modes. Carrying hosting through every document forced tenancy, ingress, remote identity, and KMS concerns into designs for subsystems that had no implementation, and left the product without a clear thesis.
- Decision: Suncode is local-first. The runtime and OS core run on the user's machine, and hosted execution is out of scope. Retain two properties that keep hosting possible later without designing for it now: the client-facing API stays a network protocol rather than in-process calls, and authority checks never assume the caller is trusted because it is local.
- Consequences: Tenancy, ingress, remote identity, cloud KMS, workspace provisioning, and sandbox-host infrastructure leave all current designs. Client authentication is a local-credential problem. The credential store is the OS keychain. Cost and complexity drop across every package. Reintroducing hosting requires a new decision record.
- Details: `PRODUCT.md`, `ARCHITECTURE.md` section 4

## ADR-20260807-runtime-owns-durable-state

- Date: 2026-08-07
- Status: Accepted
- Supersedes: the state-ownership portion of ADR-20260804-foundational-architecture
- Context: ADR-20260804 assigned SQLite, secret handling, sessions, and permission enforcement to Rust. `ARCHITECTURE.md` and every requirement package written afterward assign durable state to the TypeScript runtime and keep only OS capability enforcement in Rust. The contradiction was blocking three requirement packages.
- Decision: The TypeScript runtime owns SQLite, migrations, secret encryption, the master key, session state, settings, and approval records. Rust owns OS capability enforcement, machine-affecting execution, and a private operation journal for idempotency and crash reconciliation. Rust never opens the runtime database.
- Consequences: The earlier ADR's state-ownership claim is void; its transport, layering, and contract-first claims survive except where later decisions amend them. Rust has durable state of its own, but only for operation bookkeeping, which is why the "Rust owns no persistence" phrasing does not appear in current documents.
- Details: `ARCHITECTURE.md` sections 2, 6, 7

## ADR-20260807-rust-boundary-rationale

- Date: 2026-08-07
- Status: Accepted
- Context: Documents claimed the Rust layer was the security boundary because it is the lowest trusted layer. That does not hold: the runtime spawns Rust as a child under the same OS user, so it can bypass Rust with direct filesystem calls at any time. Stating an enforcement guarantee the deployment does not provide is worse than stating none, because later designs rely on it.
- Decision: Rust is the single audited execution point for machine-affecting operations, not an OS-enforced sandbox around the runtime. Its justified value is containing third-party code (plugins, MCP servers, npm dependencies) which never receives direct OS access, providing one auditable choke point, and holding performance-sensitive filesystem work. Whether the runtime process itself is confined is a separate, currently unanswered question.
- Consequences: The threat model names the runtime process as trusted and in-scope-for-compromise rather than pretending otherwise. "Lowest trusted layer enforces security" is removed as a principle. Because the boundary's value is auditability rather than isolation, the Rust operation surface is kept deliberately narrow — high-churn semantic work such as language-server orchestration and index ranking belongs in TypeScript over Rust primitives.
- Details: `ARCHITECTURE.md` sections 2, 9

## ADR-20260807-domain-vocabulary

- Date: 2026-08-07
- Status: Accepted
- Context: The UI used "project" and "task" while the runtime used "workspace" and "session" for the same two concepts, with an amendment explicitly endorsing the split. Renaming the two most central nouns at a layer boundary creates persistent ambiguity in documents, protocol messages, and logs.
- Decision: One vocabulary in all layers, protocols, and interfaces. A **project** is a directory tree the user has opened. A **session** is one conversation against one project. A **turn** is one user submission and its execution. Consistency with the user-facing term takes priority over internal convention.
- Consequences: "Workspace" and "task" are retired as domain nouns. "Workspace" survives only for build-tool workspaces such as Cargo and pnpm.

## ADR-20260805-agent-knowledge-layout

- Date: 2026-08-05
- Status: Accepted
- Context: Early project knowledge was stored under tool-specific paths alongside transient brainstorming state.
- Decision: Store durable contributor and agent context in `.agents/`, organized into features, dated requirements, technical specs, and this decision index. Keep local tool state ignored and outside `.agents`.
- Consequences: Legacy tool-specific paths and directives are obsolete and must not be recreated.
- Related requirement: `requirements/2026-08-05-agent-knowledge-layout/`

## ADR-20260804-foundational-architecture

- Date: 2026-08-04
- Status: Partially superseded — see ADR-20260807-runtime-owns-durable-state, ADR-20260807-hand-written-protocol-contracts, ADR-20260807-local-first-scope
- Context: Suncode needs a secure, cross-platform foundation for a local coding-agent product.
- Decision: Use a contract-first polyglot monorepo; a TypeScript runtime supervises one Rust child; JSON-RPC 2.0 uses newline-delimited stdio; Rust owns SQLite, secret handling, sessions, operations, and permission enforcement; clients communicate only with the local runtime.
- Consequences: The monorepo layout, the supervision topology, the stdio transport, and the rule that clients never reach Rust or providers directly all remain in force. State ownership was reassigned to the runtime, contract-driven code generation was dropped, and cloud hosting left scope.
- Details: `ARCHITECTURE.md`
