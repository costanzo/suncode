# Decision Index

Newest first. Historical context is retained only when it still explains a current constraint.

## ADR-20260816-avalonia-desktop-client

- Date: 2026-08-16
- Status: Accepted
- Supersedes: Qt desktop requirements in ADR-20260815-embedded-runtime-sdk, ADR-20260808-rust-unified-runtime, ADR-20260808-qt-client-state-boundary, and related Phase 1 client records; their Rust ownership and presentation-state conclusions remain accepted
- Context: Qt licensing concerns and its learning curve made the implemented Qt Quick/QML client unsuitable for continued development. The Rust SDK already provides a stable method-oriented C ABI, so the UI framework can change without moving runtime authority into the client.
- Decision: Replace the Qt production client with a .NET 10 Avalonia application. Keep presentation and transient interaction state in XAML/C#, call C ABI version 1 through a hand-written P/Invoke adapter, and emit the Rust runtime as a `cdylib` beside the managed executable. Retain the complete Qt/QML/CMake tree as the authoritative visual and interaction parity reference, while excluding Qt from the Avalonia production dependency graph.
- Consequences: Phase 1 production now depends on .NET 10 and Avalonia rather than Qt. Rust remains the only owner of providers, agent behavior, policy, SQLite, credentials, operations, approvals, recovery, and undo. Qt assets and font-selection behavior may be reused by Avalonia under their existing licenses. Native library packaging, signing, and notarization become desktop release concerns. CLI/TUI/Web remain deferred.
- Details: `.agents/requirements/2026-08-16-avalonia-desktop-migration/`, `apps/desktop-avalonia/`

## ADR-20260815-session-token-usage

- Date: 2026-08-15
- Status: Accepted
- Context: The desktop footer needs a durable cumulative token-consumption value for the selected session. Usage events are retained, but repeated events contain cumulative turn values and cannot be summed directly without double counting.
- Decision: Schema version 12 projects the latest provider-reported input, output, and total usage onto each `turns` row and backfills existing rows from retained events. The Rust SDK exposes a named `session_usage` aggregate. Qt displays only the compact session total and never opens SQLite or estimates missing provider usage.
- Consequences: Repeated usage updates replace one turn's counters; session totals sum across turns; providers that omit usage contribute zero. The footer remains a low-noise status surface rather than a cost or context dashboard.
- Details: `.agents/requirements/2026-08-15-session-token-usage/`, `../contracts/runtime-sdk/`, `../contracts/sqlite-schema.md`

## ADR-20260815-embedded-runtime-sdk

- Date: 2026-08-15
- Status: Accepted
- Supersedes: ADR-20260808-rust-sdk-client-facade and the client-facing transport portions of ADR-20260808-rust-unified-runtime, ADR-20260808-qt-client-state-boundary, and ADR-20260807-local-first-scope
- Context: Qt already embeds the Rust static library, but the facade reconstructs REST-like Axum requests and retains a standalone loopback HTTP/SSE server. Future TypeScript and Python packages need a reusable native runtime library, not a network service or duplicated runtime implementation.
- Decision: SunCode's runtime is an embedded Rust SDK only. Hosts call named Rust SDK methods through native bindings. Qt uses the stable C ABI, future TypeScript uses N-API, and future Python uses PyO3 or the stable C ABI. Remove the client-facing HTTP/SSE server, synthetic HTTP dispatch, authentication token, endpoint discovery, and standalone runtime binary. Provider adapters retain outbound HTTPS. One runtime data directory may be owned by one host process; cross-process attach and replacement IPC are out of scope.
- Consequences: SDK errors and outcomes are domain types rather than HTTP statuses; events use direct subscriptions rather than SSE; Qt no longer constructs paths; language SDKs embed the same Rust runtime and never open SQLite or implement runtime behavior. Separate processes cannot share one live runtime unless a future decision introduces an explicit IPC design.
- Details: `.agents/requirements/2026-08-15-embedded-runtime-sdk/`, `../contracts/runtime-sdk/`

## ADR-20260815-built-in-provider-expansion

- Date: 2026-08-15
- Status: Accepted
- Context: Developers need Kimi, Claude, and Gemini in the same local-first Qt workflow already used for DeepSeek, Zhipu GLM, and OpenAI.
- Decision: Add Kimi, Claude, and Gemini as trusted built-in runtime providers using the documented OpenAI-compatible chat-completions surfaces. Their stable SunCode model identities are `kimi-k2.7-code`, `claude-opus-5`, and `gemini-3.6-flash`; provider endpoints, wire models, and environment aliases remain runtime configuration.
- Consequences: Existing canonical message, tool, usage, streaming, cancellation, and error normalization code is reused. Provider-native Messages, Responses, and Gemini `generateContent` features remain deferred. Credentials stay provider-keyed in Rust-owned SQLite and Qt remains a presentation client.
- Details: `.agents/requirements/2026-08-15-model-provider-expansion/`

## ADR-20260815-multi-model-provider-catalog

- Date: 2026-08-15
- Status: Accepted
- Context: A provider can expose many models. The previous provider-global model override could send a different wire model while clients and durable sessions still named the original catalog model.
- Decision: Keep one trusted adapter per provider, register multiple stable model IDs in the Rust catalog, and route each request through a model route carrying the catalog wire model. Remove provider endpoint/model environment overrides; retain only static trusted endpoints and the existing non-interactive credential override path.
- Consequences: `/models` is the source for all selectable models, session and turn records keep stable model IDs, and provider credentials remain provider-scoped. Vendor model discovery and custom user model registration remain deferred.
- Details: `.agents/requirements/2026-08-15-multi-model-provider-catalog/`

## ADR-20260812-plaintext-provider-secrets

- Date: 2026-08-12
- Status: Accepted
- Context: Provider API keys need one cross-platform persistence path. The prior design used the OS credential store, and an intermediate implementation used a local encryption key beside SQLite. The product currently prioritizes a simple local-first implementation over at-rest encryption.
- Decision: Store provider API keys as plaintext values in the Rust-owned SQLite `secret_records` table. Keep keys out of protocol responses, events, audit records, and logs. Accept explicit environment-variable overrides only in non-interactive mode. Migrate legacy macOS Keychain entries into SQLite when possible; encrypted experimental rows that cannot be decoded are discarded by the schema migration.
- Consequences: API keys are exposed to any process or user that can read the SQLite database and its backups. The runtime must keep database access inside Rust, restrict the data directory where the platform permits it, and never include secret values in diagnostics or serialized DTOs. Reintroducing encryption or an OS credential store requires a new decision record and migration plan.
- Details: `ARCHITECTURE.md`, `contracts/persistence.md`, `contracts/sqlite-schema.md`

## ADR-20260811-built-in-provider-expansion

- Date: 2026-08-11
- Status: Accepted
- Context: The Qt desktop client needs model provider choice while preserving the local-first runtime boundary and runtime-owned credential persistence.
- Decision: Add Zhipu GLM and OpenAI as trusted built-in runtime providers alongside DeepSeek. Zhipu GLM and OpenAI use a shared OpenAI-compatible chat-completions adapter. Stable SunCode model identities are `deepseek-v4-flash`, `glm-5.2`, and `gpt-5.6-sol`.
- Consequences: Provider credentials are provider-keyed and stored by the Rust runtime in SQLite. Qt consumes provider/model availability from the runtime and stores/removes credentials through provider-keyed runtime routes. Third-party provider adapters and OpenAI Responses-native behavior remain deferred.
- Details: `.agents/requirements/2026-08-11-model-provider-expansion/`

## ADR-20260809-runtime-sdk-layout

- Date: 2026-08-09
- Status: Accepted
- Context: The root `rust/` workspace name described the implementation language instead of the product role. Future TypeScript and Python SDK packages need a home without implying that they own runtime state or duplicate runtime behavior.
- Decision: Rename the product runtime workspace to `runtime/`, keep the Rust core at `runtime/crates/core`, keep audited operations at `runtime/crates/operations`, and introduce `sdks/` for future TypeScript and Python language bindings. Keep `contracts/` at the root as the shared protocol and storage contract source.
- Consequences: Repository layout now names responsibilities rather than languages. Qt continues to link the Rust runtime static library through CMake. Future SDKs wrap the native runtime boundary instead of reading SQLite or calling providers directly.
- Details: `ARCHITECTURE.md`

## ADR-20260809-ephemeral-streaming-deltas

- Date: 2026-08-09
- Status: Accepted
- Context: OpenCode treats text, reasoning, and tool-input deltas as live projection data rather than durable database records. SunCode was persisting every `assistant.delta`, which made session startup slow when Qt replayed long histories and made token fragments look like part of the durable content model.
- Decision: Streaming deltas are live-only events. SQLite schema version 10 removes legacy retained delta rows, records per-session sequence high-water marks, and uses `session_messages` as the message/context read model including tool messages. Durable history keeps final `message.assistant` rows and lifecycle events, not raw token deltas.
- Consequences: Connected clients keep streaming UX, while session replay and startup load compact final messages and durable activity only. Sequence numbers remain monotonic after compaction or delta removal. Rebuilding message projections no longer requires replaying high-volume transient events.
- Details: `../contracts/sqlite-schema.md`

## ADR-20260808-rust-unified-runtime

- Date: 2026-08-08
- Status: Superseded in its client transport and process-topology details by ADR-20260815-embedded-runtime-sdk; Rust ownership remains accepted
- Supersedes: ADR-20260807-runtime-owns-durable-state, ADR-20260807-rust-boundary-rationale, and the process-topology portions of ADR-20260804-foundational-architecture
- Context: The TypeScript runtime plus Rust child split duplicates lifecycle, protocol, recovery, and state coordination without providing OS isolation between processes running as the same user. Phase 1 has one Qt client and benefits more from one ownership model than from a language boundary.
- Decision: Implement Phase 1 as one Rust runtime process. Rust owns provider integration, agent behavior, policy, SQLite, credentials, authenticated HTTP/SSE, and machine operations. Operations retain a narrow internal audited interface but are called in-process. Qt remains the only Phase 1 client. Production TypeScript, Node.js, and runtime-to-core JSON-RPC are removed after parity verification.
- Consequences: SQLite ownership moves to Rust; the client API and schema remain compatibility targets. The internal operations boundary provides reviewability, not OS isolation. Shared protocol vectors remain hand-written. Existing TypeScript and stdio code are migration sources, not the final product.
- Details: `ARCHITECTURE.md`

## ADR-20260808-rust-sdk-client-facade

- Date: 2026-08-08
- Status: Superseded by ADR-20260815-embedded-runtime-sdk
- Context: Phase 1 needs the Rust runtime to be reusable by Qt now and by future Web, CLI, and TUI adapters later, without forcing the client boundary to stay network-only.
- Decision: Expose the Rust runtime as a local SDK facade for direct client adapters, with Qt as the first consumer. Keep HTTP/SSE as a separate adapter over the same runtime core for compatibility and future non-Qt surfaces. The SDK is an embedding boundary, not a new authority boundary, and it does not change SQLite ownership or the local-first trust model.
- Consequences: Qt can call the Rust core without loopback transport, while Web/CLI/TUI can still use adapter-specific transports over the same runtime. The HTTP contract remains relevant for compatibility and future surfaces, but it is no longer the only Phase 1 client path.
- Details: `../apps/desktop-qt/`, superseded contract replaced by `../contracts/runtime-sdk/README.md`

## ADR-20260808-qt-client-state-boundary

- Date: 2026-08-08
- Status: Superseded in its transport details by ADR-20260815-embedded-runtime-sdk; presentation-state ownership remains accepted
- Context: The Phase 1 desktop needs reconnect, conversation, activity, approvals, touched files, undo, credentials, and diagnostics without becoming a second runtime or reading local state directly.
- Decision: The Qt C++ adapter holds only presentation state and an in-memory runtime credential. All durable state, file content, authority, recovery, provider, and diagnostic facts come from authenticated runtime DTOs and ordered SSE events. Conversation and activity are separate projections over the same event stream; Qt does not derive filesystem diffs by reading the project.
- Consequences: Client restart is recovered through snapshots/replay, and additional client surfaces can reuse the same API. Rich diff computation remains a runtime/core-derived future package rather than Qt authority.
- Details: `../apps/desktop-qt/`, `../contracts/runtime-sdk/README.md`, `features/qt-desktop-phase-1/`

## ADR-20260807-runtime-phase-1-defaults

- Date: 2026-08-07
- Status: Accepted
- Context: The first provider and vertical slice require concrete choices for authentication, message normalization, authorization, and bounded execution.
- Decision: Phase 1 uses DeepSeek V4 with API-key authentication. The canonical provider schema is SunCode-owned, role-based messages containing text content parts; the schema remains extensible, but multimodal content is deferred. The default interactive policy permits read-only project inspection and requires approval for writes, process execution, network access, secret use, destructive operations, and access outside the project. Non-interactive runs deny operations without an explicit profile grant. Default per-turn limits are 32 iterations, 32 tool calls, 10 minutes wall-clock, a configurable cost cap, and 8 MiB total output. Provider-reported token usage is recorded but is not a fixed turn termination threshold.
- Consequences: Provider credentials are user secrets resolved through plaintext SQLite secret records; environment variables are supported only as an explicit CI/script override. Provider adapters translate the canonical schema and never expose vendor payloads to clients. The runtime must enforce every limit and fail closed when authorization or credential resolution is unavailable.
- Details: `contracts/runtime-core/`, `contracts/runtime-sdk/`, `contracts/persistence.md`

## ADR-20260807-first-provider-deepseek-v4

- Date: 2026-08-07
- Status: Accepted; provider scope superseded in part by `ADR-20260811-built-in-provider-expansion`
- Context: The agent-runtime Phase 1 needs one concrete provider to define the first provider catalog, adapter, streaming normalization, usage handling, and behavioral evaluation slice. Supporting multiple providers before the canonical runtime contracts are proven would widen the implementation surface prematurely.
- Decision: DeepSeek V4 is the first built-in provider/model integration. Its stable SunCode identities are provider `deepseek` and model `deepseek-v4-flash`. The adapter remains trusted runtime code in Phase 1 and is the only provider adapter implemented in the initial vertical slice. The vendor endpoint, wire model identifier, capabilities, limits, pricing, and authentication method remain provider-contract inputs; no vendor-specific details are exposed to clients.
- Consequences: The provider gateway and fixtures are prepared against one concrete adapter boundary. This decision established DeepSeek as the first provider; later built-in provider expansion is governed by `ADR-20260811-built-in-provider-expansion`. Third-party adapters remain out of Phase 1.
- Details: `ARCHITECTURE.md`

## ADR-20260807-trusted-runtime-extension-isolation

- Date: 2026-08-07
- Status: Accepted
- Context: Treating third-party dependencies and extensions as isolated while they run in the trusted runtime would overstate the security boundary. A worker thread or child process without an OS sandbox still runs with the user's authority. The product needs a small, implementable Phase 1 before adding extension isolation and secret handoff machinery.
- Decision: Phase 1 excludes plugins, MCP servers, and third-party provider adapters from execution. Skills are data and instructions only. When third-party extensions are introduced, they must run in independent child processes and through a Rust-mediated, platform-specific OS sandbox with explicitly reported enforcement. Extension-originated requests carry an extension identity and are re-authorized at the runtime and Rust boundaries.
- Consequences: Extension IPC, sandbox profiles, scoped secret delivery, lifecycle, and failure recovery become prerequisites before third-party extensions are enabled. Built-in providers remain trusted runtime code; the trusted runtime is an explicit threat-model assumption, not an OS-enforced sandbox.
- Details: `ARCHITECTURE.md` sections 3.4, 9.5, and 14.1

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
- Details: `ARCHITECTURE.md` section 8

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
- Status: Superseded in its network-client-boundary consequence by ADR-20260815-embedded-runtime-sdk; local-first scope remains accepted
- Amends: ADR-20260804-foundational-architecture
- Context: The foundation treated local and cloud-hosted execution as equally weighted deployment modes. Carrying hosting through every document forced tenancy, ingress, remote identity, and KMS concerns into designs for subsystems that had no implementation, and left the product without a clear thesis.
- Decision: SunCode is local-first. The runtime and OS core run on the user's machine, and hosted execution is out of scope. Retain two properties that keep hosting possible later without designing for it now: the client-facing API stays a network protocol rather than in-process calls, and authority checks never assume the caller is trusted because it is local.
- Consequences: Tenancy, ingress, remote identity, cloud KMS, workspace provisioning, and sandbox-host infrastructure leave all current designs. Client authentication is a local-credential problem. The credential store is SQLite-backed local secret records. Cost and complexity drop across every package. Reintroducing hosting requires a new decision record.
- Details: `PRODUCT.md`, `ARCHITECTURE.md` section 4

## ADR-20260807-domain-vocabulary

- Date: 2026-08-07
- Status: Accepted
- Context: The UI used "project" and "task" while the runtime used "workspace" and "session" for the same two concepts, with an amendment explicitly endorsing the split. Renaming the two most central nouns at a layer boundary creates persistent ambiguity in documents, protocol messages, and logs.
- Decision: One vocabulary in all layers, protocols, and interfaces. A **project** is a directory tree the user has opened. A **session** is one conversation against one project. A **turn** is one user submission and its execution. Consistency with the user-facing term takes priority over internal convention.
- Consequences: "Workspace" and "task" are retired as domain nouns. "Workspace" survives only for build-tool workspaces such as Cargo and pnpm.

## ADR-20260815-embedded-ripgrep-search

- Date: 2026-08-15
- Status: Accepted
- Context: The built-in Rust content search was a literal in-process directory walk, while the grep tool contract described regular-expression search. Calling an `rg` executable would add an installation, PATH, and child-process dependency to the embedded runtime.
- Decision: Embed ripgrep's reusable Rust crates (`ignore`, `globset`, `grep-regex`, and `grep-searcher`) in the audited operations crate. Use Rust regular expressions, standard ripgrep ignore/hidden-file traversal, bounded project-relative glob filtering, and the existing bounded JSON result contract. Do not invoke an external `rg` process and defer PCRE2.
- Consequences: The desktop runtime has no ripgrep installation prerequisite and remains inside the Rust operation boundary. The search behavior now follows ripgrep standard filters and regex syntax; callers needing literal punctuation must escape it. Command-line-only features and output formats remain outside the operation contract.
- Details: `.agents/requirements/2026-08-15-embedded-ripgrep/`, `runtime/crates/operations/src/search.rs`

## ADR-20260815-embedded-git2-review

- Date: 2026-08-15
- Status: Accepted
- Context: The Qt project window needs the actual Git working-tree and index state, but invoking a Git executable would add an installation, PATH, and child-process dependency and allowing Qt to inspect `.git` would violate the SDK ownership boundary.
- Decision: Embed `git2` with vendored libgit2 in the audited Rust operations crate. Expose bounded read-only status and per-file diff methods through the typed Rust SDK and C ABI. Repository discovery may locate a root above the opened project, but every result and requested path remains filtered to the opened project. Qt owns only transient drawer presentation state.
- Consequences: The desktop can review all, staged, unstaged, untracked, renamed, deleted, binary, and conflicted local changes without an installed Git executable or system libgit2. Vendoring increases build time and binary size, and the Qt static-library link must include libgit2's native compression and character-conversion dependencies. Stage, discard, commit, refs, remotes, and credentials require separate policy and recovery designs.
- Details: `.agents/requirements/2026-08-15-git-diff-drawer/`, `runtime/crates/operations/src/git.rs`
