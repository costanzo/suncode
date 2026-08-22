# Decision Index

Newest first. Historical context is retained only when it still explains a current constraint.

## ADR-20260821-cross-platform-process-execution

- Date: 2026-08-21
- Status: Accepted
- Context: The model-facing `bash` tool converted every script to `/bin/sh -lc`, so approved process calls could not start on Windows. The same tool also obscured the important distinction between portable argv execution and platform-specific shell syntax.
- Decision: Advertise separate `process` and `shell` tools. `process` passes a program and explicit argv without implicit shell parsing. `shell` selects Windows PowerShell on Windows and POSIX `/bin/sh` on macOS/Linux. Inject ephemeral host OS, architecture, shell dialect, path style, and local date/time into provider requests. Keep `bash` only as a persisted-call compatibility alias and preserve stable process-start failures through the tool and turn projections.
- Consequences: Coding commands can use portable argv where possible, scripts use an explicit host dialect, and Windows no longer depends on `/bin/sh`, Git Bash, Cygwin, or WSL. Shell scripts are not portable unless authored for the reported dialect. Process execution remains approval-gated and does not gain OS or network isolation.
- Details: `.agents/requirements/2026-08-21-cross-platform-process/`, `runtime/crates/core/src/tools/`, `runtime/crates/operations/src/process.rs`

## ADR-20260820-configuration-and-provider-adapters

- Date: 2026-08-20
- Status: Accepted
- Supersedes: ADR-20260820-project-settings
- Context: Configuration was split between two tables and used a retired user scope, while custom provider rows did not state which `suncode-llm` implementation could speak their wire protocol. The conversation root also used the plural table name `sessions` while related tables were singular.
- Decision: Rename `sessions` to `session`. Replace `project_setting` and `setting_records` with one `configuration` table whose scopes are `global`, `project`, and `session`, with explicit owner foreign keys and global-to-project-to-session precedence. Add required `llm_model_provider.adapter_type`; accept only adapter identifiers implemented by `suncode-llm`, currently `openai` for OpenAI-compatible endpoints.
- Consequences: The current fresh schema contains 13 application tables. Custom provider identity remains independent from protocol compatibility, and unsupported adapters fail before registration. Configuration ownership and cascade cleanup are enforced relationally.
- Details: `contracts/sqlite-schema.md`, `contracts/persistence.md`, `.agents/requirements/2026-08-20-project-settings/`

## ADR-20260820-normalized-session-source

- Date: 2026-08-20
- Status: Accepted
- Context: `session_content` duplicated normalized messages, calls, tool uses, and turn state while also retaining large provider payloads. This caused unbounded database growth and required a second replay cursor table.
- Decision: Remove `session_content` and `session_sequences`. Normalized session tables are the durable source of truth. Runtime events are in-memory notifications only; subscriptions do not replay from SQLite. A lagged subscriber receives `resync.required` and reloads a normalized session snapshot.
- Consequences: Durable event sequence fields and replay cursors are removed. Provider context, tool results, approvals, recovery, and session history remain queryable in their dedicated tables. The later configuration consolidation brings the current schema to 13 tables.
- Details: `.agents/requirements/2026-08-20-session-storage/`, `contracts/runtime-sdk/README.md`, `contracts/sqlite-schema.md`

## ADR-20260820-session-storage

- Date: 2026-08-20
- Status: Accepted
- Context: Session persistence had separate legacy tables for turns, submissions, provider exchanges, tool calls, suspended continuations, and messages. This duplicated lifecycle ownership and made call/tool/message correlation incomplete.
- Decision: Use `session_turn` as the single turn fact table, including submission idempotency, cumulative usage, and approval recovery. Use `session_call` for each LLM request, its provider-reported usage, and independently nullable provider HTTP request and response-object identifiers; keep the SunCode-generated call ID as the physical key. Use `session_tool_use` as the exclusive durable owner of each tool request/result/state, and `session_message` for user, assistant, and thinking messages without duplicate usage. Derive transient provider-role tool messages from succeeded tool-use rows when rebuilding model context. Link readable messages and tool uses to calls where available. Order `session_message` by `created_at` with a rowid tie-breaker. Runtime events are live-only notifications and are not persisted as a duplicate journal.
- Consequences: The legacy `turns`, `turn_submissions`, `provider_exchanges`, `tool_calls`, `suspended_turns`, and `session_messages` tables are removed from the fresh current schema. Existing provider-exchange DTO and SDK names remain compatibility aliases for the `session_call` projection.
- Details: `.agents/requirements/2026-08-20-session-storage/`, `contracts/sqlite-schema.md`, `contracts/persistence.md`

## ADR-20260820-project-settings

- Date: 2026-08-20
- Status: Superseded by ADR-20260820-configuration-and-provider-adapters
- Context: Project-owned configuration was stored in the polymorphic `setting_records` table even though project settings have their own lifecycle and currently control the project's default model.
- Decision: Keep `projects` unchanged and add `project_setting(project_id, key, value_json, updated_at)` with a foreign key to `projects` and cascade cleanup. Restrict `setting_records` to user and session scopes. Preserve the existing settings SDK shape while routing project writes and effective reads through the dedicated table. Resolve `default_model` from the project when session or turn callers omit a model.
- Consequences: Project configuration has an explicit ownership boundary and can grow without adding nullable project columns. The project-settings delivery originally added `project_setting`; the later session-storage decision consolidates the current schema to 14 tables without changing this ownership boundary.
- Details: `.agents/requirements/2026-08-20-project-settings/`, `contracts/sqlite-schema.md`

## ADR-20260819-llm-package-boundary

- Date: 2026-08-19
- Status: Accepted
- Supersedes: the custom-provider deferral in ADR-20260815-multi-model-provider-catalog and the blanket in-process provider-adapter exclusion in ADR-20260807-trusted-runtime-extension-isolation; dynamic and executable extension restrictions remain accepted
- Context: Provider traits, canonical completion types, model metadata, HTTP adapters, credential persistence, tool declarations, and agent state were coupled inside runtime core. Enterprises also need to integrate private model gateways without coupling provider code to SunCode's SQLite schema.
- Decision: Create the standalone `suncode-llm` package. It owns provider-neutral completion contracts, model metadata and routing, built-in models, OpenAI-compatible HTTP/SSE behavior, and public registration of trusted `LlmProvider` implementations. It has no database dependency. Core owns credential persistence and implements `ApiKeyResolver`, supplies tool schemas per request, converts persistence DTOs at the agent boundary, and lets Rust hosts extend the built-in registry during `RuntimeSdk` construction. Custom registration is trusted in-process Rust composition; persisted desktop configuration, C ABI registration, dynamic loading, and executable provider plugins remain deferred.
- Consequences: Built-in and enterprise providers share one reusable LLM layer, identifiers can be owned rather than static, and provider tests no longer require runtime/database types. Custom implementations run with the host process's authority and are not sandboxed. The existing SDK ABI, schema, built-in IDs, and credential behavior remain unchanged.
- Details: `.agents/requirements/2026-08-19-llm-package/`, `runtime/crates/llm/`

## ADR-20260819-persisted-llm-catalog

- Date: 2026-08-19
- Status: Accepted
- Context: Provider endpoints, API keys, model request codes, context limits, and auto-compaction settings must be durable and support custom enterprise providers, while `suncode-llm` must remain database-free.
- Decision: Add `llm_model_provider` and `llm_model` to the current schema. Provider rows store built-in/custom provider identity, endpoint, plaintext API key, enabled state, and ordering. Model rows store provider ownership, display/request identifiers, context and auto-compaction token limits, output limits, capabilities, enabled state, and ordering. Seed six providers and twelve models idempotently. Core reads these rows and assembles the `suncode-llm` registry; the old `secret_records` table and static runtime catalog are removed from the current schema/source. `suncode-llm` never opens SQLite.
- Consequences: Custom providers can be represented and routed through persisted endpoint/model rows, and model-aware compaction settings are durable. Existing development databases with the previous table set are rejected without migration. Provider/model CRUD beyond credential updates remains a future SDK contract.
- Details: `.agents/requirements/2026-08-19-persisted-llm-catalog/`, `contracts/sqlite-schema.md`, `contracts/persistence.md`

## ADR-20260819-current-schema-bootstrap

- Date: 2026-08-19
- Status: Accepted
- Supersedes: ADR-20260819-sqlite-schema-v14 and the durable client-synchronization cursor portion of ADR-20260807-durable-stream-separation
- Context: SunCode is a new system, but its database implementation mixed current storage behavior with historical schema versions, upgrade functions, and a monolithic SQL file.
- Decision: Keep one current SQLite schema with no version or migration metadata. The dedicated `suncode-db` Cargo package at `runtime/crates/db` owns persistence DTOs, store operations, and schema/data resources; the runtime core consumes it as a library dependency. An ordered schema manifest applies one table-named SQL file per table and a separate ordered data manifest in one transaction. File names do not encode execution order. Reopening the current schema is idempotent, while an unexpected application table causes open to fail without conversion. Live session subscriptions recover through normalized snapshots rather than a persisted cursor table.
- Consequences: Database ownership and table review are explicit, old compatibility code is removed, and incompatible development databases are never silently rewritten. Pending/resuming approvals remain recoverable, terminal continuation payloads are released, and focused recovery/retention indexes remain. Future released-schema evolution requires a new compatibility decision.
- Details: `.agents/requirements/2026-08-19-db-module-layout/`, `../contracts/sqlite-schema.md`, `../contracts/persistence.md`

## ADR-20260819-sqlite-schema-v14

- Date: 2026-08-19
- Status: Superseded before adoption by ADR-20260819-current-schema-bootstrap
- Context: This proposal designed a version 14 upgrade from historical development databases.
- Decision: Retained only as decision history. Its table/index review and terminal snapshot cleanup informed the current schema, but its version tracking, legacy conversion, and compatibility path are not implemented.
- Details: `.agents/requirements/2026-08-19-sqlite-schema-v14-optimization/`

## ADR-20260819-general-purpose-coding-agent

- Date: 2026-08-19
- Status: Accepted
- Supersedes: the product-positioning conclusion of ADR-20260807-desktop-runtime-scope
- Context: Product and interface copy defined SunCode by the deployment topology of its first desktop release. That wording incorrectly narrowed a coding agent intended for broad software-development work.
- Decision: Define SunCode as a general-purpose coding agent. Treat the embedded Avalonia and Rust deployment as the current Phase 1 architecture, not as the product category or a permanent limit on future surfaces.
- Consequences: Product, design, contributor, and interface copy lead with broad coding utility. Current desktop ownership, persistence, authority, and deferred hosted scope remain unchanged until separate requirements change them.
- Details: `.agents/requirements/2026-08-19-general-coding-agent-positioning/`, `PRODUCT.md`, `ARCHITECTURE.md`

## ADR-20260816-avalonia-desktop-client

- Date: 2026-08-16
- Status: Accepted
- Supersedes: Qt desktop requirements in ADR-20260815-embedded-runtime-sdk, ADR-20260808-rust-unified-runtime, ADR-20260808-qt-client-state-boundary, and related Phase 1 client records; their Rust ownership and presentation-state conclusions remain accepted
- Context: Qt licensing concerns and its learning curve made the implemented Qt Quick/QML client unsuitable for continued development. The Rust SDK already provides a stable method-oriented C ABI, so the UI framework can change without moving runtime authority into the client.
- Decision: Replace the former desktop client with a .NET 10 Avalonia application. Keep presentation and transient interaction state in XAML/C#, call C ABI version 1 through a hand-written P/Invoke adapter, and emit the Rust runtime as a `cdylib` beside the managed executable. The alternate desktop source is removed; Avalonia is the sole desktop client.
- Consequences: Phase 1 production now depends on .NET 10 and Avalonia rather than Qt. Rust remains the only owner of providers, agent behavior, policy, SQLite, credentials, operations, approvals, recovery, and undo. Qt assets and font-selection behavior may be reused by Avalonia under their existing licenses. Native library packaging, signing, and notarization become desktop release concerns. CLI/TUI/Web remain deferred.
- Details: `.agents/requirements/2026-08-16-avalonia-desktop-migration/`, `apps/desktop-avalonia/`

## ADR-20260815-session-token-usage

- Date: 2026-08-15
- Status: Accepted for usage projection; versioned backfill details superseded by ADR-20260819-current-schema-bootstrap
- Context: The desktop footer needs a durable cumulative token-consumption value for the selected session. Usage events are retained, but repeated events contain cumulative turn values and cannot be summed directly without double counting.
- Decision: Project the latest provider-reported input, output, and total usage onto each `turns` row. The Rust SDK exposes a named `session_usage` aggregate. The desktop displays only the compact session total and never opens SQLite or estimates missing provider usage.
- Consequences: Repeated usage updates replace one turn's counters; session totals sum across turns; providers that omit usage contribute zero. The footer remains a low-noise status surface rather than a cost or context dashboard.
- Details: `.agents/requirements/2026-08-15-session-token-usage/`, `../contracts/runtime-sdk/`, `../contracts/sqlite-schema.md`

## ADR-20260815-embedded-runtime-sdk

- Date: 2026-08-15
- Status: Accepted
- Supersedes: ADR-20260808-rust-sdk-client-facade and the client-facing transport portions of ADR-20260808-rust-unified-runtime, ADR-20260808-qt-client-state-boundary, and ADR-20260807-desktop-runtime-scope
- Context: Qt already embeds the Rust static library, but the facade reconstructs REST-like Axum requests and retains a standalone loopback HTTP/SSE server. Future TypeScript and Python packages need a reusable native runtime library, not a network service or duplicated runtime implementation.
- Decision: SunCode's runtime is an embedded Rust SDK only. Hosts call named Rust SDK methods through native bindings. Qt uses the stable C ABI, future TypeScript uses N-API, and future Python uses PyO3 or the stable C ABI. Remove the client-facing HTTP/SSE server, synthetic HTTP dispatch, authentication token, endpoint discovery, and standalone runtime binary. Provider adapters retain outbound HTTPS. One runtime data directory may be owned by one host process; cross-process attach and replacement IPC are out of scope.
- Consequences: SDK errors and outcomes are domain types rather than HTTP statuses; events use direct subscriptions rather than SSE; Qt no longer constructs paths; language SDKs embed the same Rust runtime and never open SQLite or implement runtime behavior. Separate processes cannot share one live runtime unless a future decision introduces an explicit IPC design.
- Details: `.agents/requirements/2026-08-15-embedded-runtime-sdk/`, `../contracts/runtime-sdk/`

## ADR-20260815-built-in-provider-expansion

- Date: 2026-08-15
- Status: Accepted
- Context: Developers need Kimi, Claude, and Gemini in the same Qt coding workflow already used for DeepSeek, Zhipu GLM, and OpenAI.
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
- Status: Superseded by ADR-20260819-persisted-llm-catalog
- Context: Provider API keys need one cross-platform persistence path. The prior design used the OS credential store, and an intermediate implementation used a local encryption key beside SQLite. The current desktop release prioritizes a simple embedded implementation over at-rest encryption.
- Decision: Store provider API keys as plaintext values in the Rust-owned SQLite `llm_model_provider.api_key` column. Keep keys out of protocol responses, events, audit records, and logs. Accept explicit environment-variable overrides only in non-interactive mode. Import a legacy macOS Keychain value into an empty provider slot when possible.
- Consequences: API keys are exposed to any process or user that can read the SQLite database and its backups. The runtime must keep database access inside Rust, restrict the data directory where the platform permits it, and never include secret values in diagnostics or serialized DTOs. Reintroducing encryption or an OS credential store requires a new decision record and migration plan.
- Details: `ARCHITECTURE.md`, `contracts/persistence.md`, `contracts/sqlite-schema.md`

## ADR-20260811-built-in-provider-expansion

- Date: 2026-08-11
- Status: Accepted
- Context: The Qt desktop client needs model provider choice while preserving the embedded runtime boundary and runtime-owned credential persistence.
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
- Status: Accepted for streaming behavior; versioned cleanup details superseded by ADR-20260819-current-schema-bootstrap
- Context: OpenCode treats text, reasoning, and tool-input deltas as live projection data rather than durable database records. SunCode was persisting every `assistant.delta`, which made session startup slow when Qt replayed long histories and made token fragments look like part of the durable content model.
- Decision: Streaming deltas are live-only events. The normalized message projection retains final user/assistant/thinking content rather than raw token fragments; current provider context derives tool-role messages from `session_tool_use` under ADR-20260820-session-storage.
- Consequences: Connected clients keep streaming UX, while session replay and startup load compact final messages and durable activity only. Sequence numbers remain monotonic after compaction or delta removal. Rebuilding message projections no longer requires replaying high-volume transient events.
- Details: `../contracts/sqlite-schema.md`

## ADR-20260808-rust-unified-runtime

- Date: 2026-08-08
- Status: Superseded in its client transport and process-topology details by ADR-20260815-embedded-runtime-sdk; Rust ownership remains accepted
- Supersedes: ADR-20260807-runtime-owns-durable-state, ADR-20260807-rust-boundary-rationale, and the process-topology portions of ADR-20260804-foundational-architecture
- Context: The TypeScript runtime plus Rust child split duplicates lifecycle, protocol, recovery, and state coordination without providing OS isolation between processes running as the same user. Phase 1 has one Qt client and benefits more from one ownership model than from a language boundary.
- Decision: Implement Phase 1 as one Rust runtime process. Rust owns provider integration, agent behavior, policy, SQLite, credentials, authenticated HTTP/SSE, and machine operations. Operations retain a narrow internal audited interface but are called in-process. Qt remains the only Phase 1 client. Production TypeScript, Node.js, and runtime-to-core JSON-RPC are removed after parity verification.
- Consequences: SQLite ownership moves to Rust; the client API and current schema remain explicit contracts. The internal operations boundary provides reviewability, not OS isolation. Shared protocol vectors remain hand-written. Existing TypeScript and stdio code are implementation references, not the final product.
- Details: `ARCHITECTURE.md`

## ADR-20260808-rust-sdk-client-facade

- Date: 2026-08-08
- Status: Superseded by ADR-20260815-embedded-runtime-sdk
- Context: Phase 1 needs the Rust runtime to be reusable by Qt now and by future Web, CLI, and TUI adapters later, without forcing the client boundary to stay network-only.
- Decision: Expose the Rust runtime as an embedded SDK facade for direct client adapters, with Qt as the first consumer. Keep HTTP/SSE as a separate adapter over the same runtime core for compatibility and future non-Qt surfaces. The SDK is an embedding boundary, not a new authority boundary, and it does not change SQLite ownership or the runtime trust model.
- Consequences: Qt can call the Rust core without loopback transport, while Web/CLI/TUI can still use adapter-specific transports over the same runtime. The HTTP contract remains relevant for compatibility and future surfaces, but it is no longer the only Phase 1 client path.
- Details: superseded contract replaced by `../contracts/runtime-sdk/README.md`

## ADR-20260808-qt-client-state-boundary

- Date: 2026-08-08
- Status: Superseded in its transport details by ADR-20260815-embedded-runtime-sdk; presentation-state ownership remains accepted
- Context: The Phase 1 desktop needs reconnect, conversation, activity, approvals, touched files, undo, credentials, and diagnostics without becoming a second runtime or reading local state directly.
- Decision: The Qt C++ adapter holds only presentation state and an in-memory runtime credential. All durable state, file content, authority, recovery, provider, and diagnostic facts come from authenticated runtime DTOs and ordered SSE events. Conversation and activity are separate projections over the same event stream; Qt does not derive filesystem diffs by reading the project.
- Consequences: Client restart is recovered through snapshots/replay, and additional client surfaces can reuse the same API. Rich diff computation remains a runtime/core-derived future package rather than Qt authority.
- Details: `../contracts/runtime-sdk/README.md`

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
- Status: Accepted; durable client-sync cursor superseded by ADR-20260819-current-schema-bootstrap
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

## ADR-20260807-desktop-runtime-scope

- Date: 2026-08-07
- Status: Product-positioning conclusion superseded by ADR-20260819-general-purpose-coding-agent; network-client-boundary consequence superseded by ADR-20260815-embedded-runtime-sdk; Phase 1 desktop scope remains accepted
- Amends: ADR-20260804-foundational-architecture
- Context: The foundation treated desktop and cloud-hosted execution as equally weighted Phase 1 deployment modes. Carrying hosting through every document forced tenancy, ingress, remote identity, and KMS concerns into designs for subsystems that had no implementation.
- Decision: The Phase 1 runtime and OS core run on the user's machine, and hosted execution is out of current scope. Retain two properties that keep hosting possible later without designing for it now: client boundaries stay explicit, and authority checks never assume the caller is trusted merely because it runs in the same environment.
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
