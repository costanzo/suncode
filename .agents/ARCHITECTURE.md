# Suncode Architecture

**Status:** Approved

**Date:** 2026-08-05

**Scope:** Repository harness, architectural boundaries, protocol foundation, and documentation

## 1. Purpose

Suncode is a coding-agent platform comparable in purpose to OpenCode, Claude Code, and Codex. It supports both local execution and isolated cloud-hosted execution while separating high-performance and security-sensitive machine operations from AI orchestration and presentation.

This specification defines the foundation for that system without implementing product behavior. The first milestone establishes an implementation-ready polyglot repository, a language-neutral protocol contract, build and verification conventions, continuous integration, and architectural documentation.

The milestone succeeds when a new contributor can clone the repository, run one documented verification command, and confirm that its contracts, workspaces, generated types, fixtures, and documentation are internally consistent on every supported operating system.

## 2. Architectural Principles

1. **Rust is the trusted OS core.** It owns machine-affecting operations, persistence, permissions, session durability, and secret encryption within the local or cloud workspace where it runs.
2. **TypeScript owns AI behavior.** It owns model integrations, context construction, agent loops, orchestration, and conversational approval handling.
3. **Clients are presentation shells.** Qt desktop, web, and mobile clients consume a shared client API and do not access the core or model providers directly. The desktop application must use Qt; Electron is prohibited.
4. **Contracts are language-neutral.** JSON Schema and OpenRPC define the boundary between Rust and TypeScript. Neither language's types are canonical.
5. **Configuration is interface-managed.** Users do not edit configuration files. Rust persists settings in SQLite and exposes them through APIs.
6. **The lowest trusted layer enforces security.** TypeScript may request operations and user approvals, but Rust makes the authoritative permission decision.
7. **Durable history is append-only.** Session events are the source of truth; materialized projections provide efficient current-state access.
8. **The first milestone builds the harness, not the product.** Compile-only boundaries and contract adapters are allowed; functional agent, operation, persistence, and UI behavior are excluded.

## 3. System Boundaries

### 3.1 Interface clients

The supported interfaces are:

- Qt desktop application
- Web application
- Mobile application

Clients contain presentation logic and user interaction. They communicate with the agent runtime through the same authenticated HTTP and WebSocket API in both deployment modes. Local clients use a loopback endpoint; remote clients use TLS-protected HTTPS and WebSocket Secure (WSS). Clients never access SQLite, the Rust process, or model providers directly.

A client may connect to a runtime on the same device or to an authorized cloud-hosted runtime. Mobile clients may use WSS to connect to the cloud service. In a cloud deployment, the TypeScript runtime and Rust OS core are colocated inside an isolated execution environment, and the Rust core operates on that environment's authorized cloud workspace rather than the client device.

### 3.2 TypeScript runtime

The TypeScript layer owns:

- Model-provider integrations
- Context construction and compaction
- Agent-loop state and control
- Tool orchestration
- Conversational permission prompts
- Translation between client events and core RPC calls
- Authenticated local and remote HTTP and WebSocket client APIs
- Lifecycle supervision of the Rust child process

TypeScript does not connect to SQLite and does not directly perform privileged filesystem, search, shell, or write operations.

### 3.3 Rust core

The Rust layer owns:

- Filesystem, search, glob, process, and write operations
- Workspace-boundary enforcement and path canonicalization
- Capability grants, approvals, and revocations
- Durable sessions and event sequencing
- SQLite connections, transactions, schema, and migrations
- Settings persistence
- Encryption and decryption of secrets
- Operation metadata and artifact references
- Recovery of durable state

Rust runs as one long-lived child process colocated with its supervising TypeScript runtime in both local and cloud deployments. Internal Rust crates are compilation and testing boundaries, not client-reachable services.

### 3.4 Dependency rules

- Clients depend only on the shared client SDK and presentation-specific libraries.
- The TypeScript runtime depends on generated protocol types and its Rust RPC client.
- Rust core crates depend on generated protocol types and internal Rust abstractions.
- Generated types depend only on the language-neutral contracts.
- No client imports agent-runtime or Rust-core internals.
- No TypeScript package opens the application database.
- No Rust crate contains model-provider or presentation logic.

## 4. Process Topology and Lifecycle

### 4.1 Local deployment

A local deployment uses one on-demand runtime per operating-system user:

1. A launcher acquires a per-user single-instance lock or attaches to the healthy runtime that already owns it.
2. The TypeScript runtime creates a random runtime-lifetime credential and an authenticated loopback endpoint.
3. TypeScript starts one long-lived Rust child process.
4. TypeScript and Rust complete protocol initialization and durable recovery before client operations are accepted.
5. The runtime exits after a configurable idle period only when no client, agent turn, core operation, or background job is active.

Qt desktop, web, and compatible same-device mobile clients share this runtime rather than launching independent agent and core processes.

### 4.2 Cloud deployment

A cloud deployment hosts the agent runtime and Rust OS core together in an isolated execution environment:

1. A Qt, web, or mobile client discovers the service HTTPS and WSS endpoints.
2. The service authenticates the user and authorizes access to a tenant and workspace before accepting RPC.
3. Cloud infrastructure assigns or resumes an isolated runtime environment scoped to that authorized workspace.
4. The TypeScript runtime starts or attaches to its colocated Rust child and completes protocol initialization and durable recovery.
5. Only the Rust core receives access to the workspace filesystem, persistence volume, secrets, and machine capabilities assigned to that environment.
6. The client initializes its WebSocket session and resumes ordered session events after its last acknowledged sequence.
7. Cloud lifecycle policy may suspend or terminate an idle environment only after durable state is safe to resume.

Ingress, authentication, scheduling, and isolation infrastructure support the three application layers; they do not create a client-accessible path to the Rust core. The runtime-to-core JSON-RPC transport remains private to the isolated environment.

### 4.3 Concurrency model

The first product architecture supports concurrent independent sessions. Each session initially runs one sequential agent loop. Rust coordinates shared resources and rejects or serializes conflicting mutations.

Parallel subagents within a session are a future capability. Event sequencing, cancellation, and identifiers must not prevent adding them later.

### 4.4 Client authentication and isolation

Binding to a loopback interface is not sufficient authentication. A local runtime generates a high-entropy runtime-lifetime credential and requires it on local HTTP and WebSocket connections. Discovery and handoff use an operating-system-appropriate channel restricted to the current user.

Remote connections require TLS and authenticated WSS. Authentication establishes a user identity; authorization independently binds every connection and request to allowed tenant, workspace, and session scopes. Credentials must not appear in WebSocket URLs or ordinary RPC payloads. Browser origins are validated, tokens are short-lived and revocable, and reconnects are reauthenticated.

The cloud service isolates workspaces, processes, storage, network access, secrets, and resource limits between tenants. The Rust core validates workspace and capability scope on every machine-affecting operation even when an ingress service or TypeScript runtime has already authorized the caller. Concrete local handoff, remote identity provider, token format, TLS termination, and isolation technology remain deferred security decisions.

## 5. Rust–TypeScript Protocol

### 5.1 Contract source of truth

The protocol is defined with:

- JSON Schema for message payloads and shared data types
- OpenRPC for methods, params, results, errors, and notifications
- Versioned example and conformance fixtures

Rust and TypeScript types are generated from these definitions. Generated source is never manually edited. Continuous integration regenerates types and fails if committed outputs drift from the contracts.

### 5.2 Transport and framing

TypeScript communicates with its Rust child using JSON-RPC 2.0 over standard input and standard output.

- Encoding is UTF-8.
- Framing is newline-delimited JSON.
- One physical line contains one complete JSON-RPC message.
- Newline characters inside JSON strings are escaped by JSON encoding.
- Rust standard output is reserved exclusively for protocol messages.
- Rust logs and diagnostics use standard error.
- Both sides enforce a negotiated maximum message size.
- Protocol corruption terminates the child; neither side attempts unsafe stream resynchronization.

Binary data and unbounded content are not embedded directly in RPC messages. Large results use bounded pagination, streaming notifications where appropriate, or Rust-managed artifact references whose access is permission-checked.

### 5.3 Initialization handshake

Before normal requests, the peers exchange:

- Protocol version and supported version range
- Runtime and core build versions
- Supported methods and optional capabilities
- Operating-system and architecture information
- Database schema version
- Maximum message size
- Session recovery and resume capabilities

An incompatible protocol version produces a typed initialization failure. Optional behavior is capability-negotiated rather than inferred from build versions.

### 5.4 Message conventions

- Request identifiers are unique strings within a runtime lifetime.
- Every session-scoped request contains a session identifier.
- Every mutating operation contains an idempotency key.
- Notifications report session events, operation progress, and lifecycle changes.
- Long-running requests support cancellation.
- Session events contain a strictly increasing per-session sequence number.
- Unknown fields are tolerated within a compatible protocol version.
- Missing required fields and invalid values fail schema validation.
- Errors use stable machine-readable codes, a safe human-readable message, and optional typed data.

### 5.5 Health and recovery

Heartbeats detect an unresponsive child without delaying active work. Timing thresholds are configurable runtime policy, not part of the wire contract.

If Rust exits:

1. Outstanding RPC calls fail with a typed `core_unavailable` error.
2. TypeScript restarts Rust with bounded exponential backoff.
3. Durable state is recovered through Rust from the event journal and projections.
4. Operations whose completion is unknown are not silently retried unless their idempotency contract makes retry safe.
5. Repeated startup failure leaves the runtime in a diagnostic state and prevents agent execution.

Clients reconnect using the last observed event sequence. The runtime resumes after that sequence without duplicating acknowledged events. Retention and compaction behavior will be specified with the session subsystem; the protocol must return an explicit `resume_unavailable` result when the requested sequence is no longer available.

## 6. Persistence and Configuration

### 6.1 Database ownership and location

Rust exclusively owns the SQLite connection, schema, migrations, transactions, and integrity checks. In a local deployment, the database resides in the operating system's per-user application-data directory. In a cloud deployment, it resides on durable storage scoped to the isolated tenant and workspace environment; the same database must not have multiple concurrent Rust writers. Rust resolves the location and exposes only a redacted diagnostic representation through RPC.

Users configure the application through Qt desktop, web, or mobile interfaces. There is no user-edited configuration file. TypeScript and clients use RPC-backed APIs for all settings access.

### 6.2 Stored data

SQLite stores:

- Global settings
- Workspace-specific settings keyed by stable workspace identifiers
- Encrypted API keys and tokens
- Session metadata
- Append-only session events
- Materialized session projections
- Permission policies and approval records
- Operation metadata and artifact references
- Database migration state
- Encryption algorithm and key-version metadata

Not every database field is encrypted. API keys, access tokens, refresh tokens, and other explicitly classified secrets are encrypted. Session messages and ordinary settings are protected by operating-system file permissions but remain plaintext unless a later data-classification design expands encryption coverage.

### 6.3 Secret encryption

On first launch, Rust generates a random master key. Local deployments store it in the operating system's credential store; cloud deployments store or wrap it with an authorized cloud KMS or secret manager scoped to the tenant and workspace. The master key is never stored in SQLite.

Sensitive database values use authenticated encryption:

- Each encrypted value receives a unique nonce.
- SQLite stores ciphertext, nonce, algorithm identifier, and key version.
- Associated data binds ciphertext to its record type, record identity, and field identity.
- Decryption failures are typed integrity errors and never fall back to plaintext interpretation.
- Logs, diagnostics, RPC errors, exported sessions, and crash reports redact secrets by default.

Master-key rotation re-encrypts secret records transactionally in resumable batches. Rotation state is durable so an interrupted migration can continue safely. The precise authenticated-encryption algorithm and credential-store abstraction will be chosen in the persistence and security implementation design.

### 6.4 Backup implications

Database backups preserve ciphertext for classified secret fields, but backups cannot be decrypted on another machine without an authorized master-key transfer. Portable backup and recovery are outside milestone one and require a separate design.

## 7. Durable Session Model

### 7.1 Event journal

Session history is an append-only event journal owned by Rust. Events include messages, agent state transitions, tool requests, approvals, operation progress, results, cancellations, errors, and recovery transitions.

Each session event has:

- A stable event identifier
- A session identifier
- A strictly increasing per-session sequence number
- An event type and schema version
- A timestamp recorded by Rust
- A typed payload
- Correlation and causation identifiers where applicable

### 7.2 Materialized projections

Rust maintains relational projections for efficient current-state queries. Appending an event and updating affected projections occurs in one SQLite transaction.

Projections are disposable: Rust can rebuild them deterministically from the event journal. TypeScript reconstructs its runtime state through Rust APIs and never reads or replays SQLite directly.

### 7.3 Ordering and conflicts

Sequence allocation and event append occur within the same database transaction. Concurrent independent sessions do not share a sequence counter. Operations affecting the same protected resource are serialized or rejected with typed conflict information. The detailed resource-locking policy belongs to the operations subsystem design.

## 8. Permission and Operation Security

Rust is the authoritative security boundary for all machine-affecting operations.

### 8.1 Evaluation

Before execution, Rust:

1. Parses and validates the requested operation.
2. Canonicalizes all relevant paths without trusting client-supplied normalization.
3. Resolves workspace and capability scope.
4. Evaluates existing grants and policy.
5. Returns either authorization, denial, or a typed approval requirement.

TypeScript may explain an approval request and collect the user's decision, but it cannot manufacture a grant. It submits the decision through a dedicated authorization RPC. Rust validates, persists, and applies that decision.

### 8.2 Grant scopes

The architecture supports:

- One-operation grants
- Session-scoped grants
- Workspace-scoped grants
- Explicitly persistent grants

Each grant is constrained by operation class, resource or canonical path scope, lifetime, and relevant argument restrictions. Persistent grants are managed through interfaces and stored by Rust in SQLite.

### 8.3 Auditability

Every security-sensitive decision produces an auditable session event, including the requested capability, evaluated scope, decision source, grant lifetime, and outcome. Audit payloads must not contain secret values.

## 9. Repository Harness

The repository uses a contract-first polyglot monorepo:

```text
suncode/
├── contracts/
│   ├── openrpc/
│   ├── schemas/
│   ├── examples/
│   └── fixtures/
├── rust/
│   ├── Cargo.toml
│   └── crates/
│       ├── core-process/
│       ├── rpc-server/
│       ├── operations/
│       ├── policy/
│       ├── sessions/
│       └── persistence/
├── typescript/
│   └── packages/
│       ├── agent-runtime/
│       ├── runtime-server/
│       ├── rust-client/
│       └── client-sdk/
├── apps/
│   ├── desktop-qt/
│   ├── web/
│   └── mobile/
├── generated/
│   ├── rust/
│   └── typescript/
├── tooling/
│   ├── contract-generation/
│   ├── contract-validation/
│   └── repository-scripts/
├── docs/
│   ├── architecture/
│   ├── adr/
│   ├── protocol/
│   ├── security/
│   ├── development/
│   └── testing/
└── .github/workflows/
```

This tree expresses intended ownership. Milestone one creates only the files and minimal package or crate shells needed for builds, dependency-rule checks, type generation, fixtures, and documentation. Empty speculative modules are not created solely to mirror the diagram.

### 9.1 Toolchains

- `rust-toolchain.toml` pins the Rust toolchain.
- The root Node manifest declares the supported Node range and pins pnpm through its `packageManager` field.
- Cargo and pnpm remain native, separate workspaces.
- Root commands provide cross-platform entry points for bootstrap, generation, formatting, linting, testing, documentation checks, and full verification.
- Root automation uses a cross-platform implementation rather than Make or Unix-only shell scripts.
- Exact toolchain versions are chosen and recorded during the implementation plan, using then-current stable releases that are supported on Windows, macOS, and Linux.

### 9.2 Generated artifacts

Contracts are the only hand-edited protocol source. Generation must be deterministic. CI runs generation and fails if it changes committed generated outputs. Each generated directory contains a notice identifying its source and regeneration command.

## 10. Documentation System

Documentation is an architectural deliverable:

- `docs/architecture/` describes system boundaries, dependency rules, process topology, and data flow.
- `docs/adr/` records durable decisions and their consequences.
- `docs/protocol/` defines transport framing, initialization, compatibility, cancellation, progress, errors, and recovery.
- `docs/security/` defines the threat model, trust boundaries, encryption model, permission enforcement, and local-client authentication.
- `docs/development/` documents prerequisites, repository commands, generation, supported platforms, and contribution conventions.
- `docs/testing/` documents the test pyramid, fixtures, conformance requirements, and platform matrix.

Initial architecture decision records cover at least:

1. Contract-first polyglot monorepo
2. TypeScript runtime supervising one Rust child
3. JSON-RPC 2.0 over newline-delimited stdio
4. Rust-owned SQLite with interface-managed configuration
5. OS-credential-store master key and encrypted secret fields
6. Append-only session journal with materialized projections
7. Rust-enforced capabilities and approvals
8. On-demand single-instance local runtime

Documentation links and referenced files are checked in CI. Each document identifies its status and scope so proposals are distinguishable from accepted architecture.

## 11. Verification Strategy

### 11.1 Contract verification

The harness verifies:

- JSON Schema validity
- OpenRPC validity and references
- Deterministic generated Rust and TypeScript types
- Equivalent acceptance and rejection of shared fixtures by both languages
- Initialization, version negotiation, and capability negotiation fixtures
- Valid requests, responses, notifications, progress, cancellation, and typed errors
- Malformed JSON, invalid envelopes, missing fields, unknown fields, oversized messages, unexpected EOF, and incompatible versions
- Idempotency-key and session-sequence constraints

### 11.2 Framing verification

Transport tests cover complete frames, partial reads, multiple frames per read, escaped newlines, malformed UTF-8, size limits, closed streams, stderr separation, and child termination.

### 11.3 Architecture and repository verification

Automated checks cover:

- Forbidden cross-layer imports and dependencies
- Formatting and linting for Rust, TypeScript, schemas, and documentation
- Clean builds for all milestone workspace members
- Documentation links
- Reproducible generation
- A clean Git worktree after the full verification command

### 11.4 Platform matrix

CI runs on Linux, Windows, and macOS because process creation, stdio behavior, path handling, application-data locations, filesystem semantics, and credential stores differ materially across platforms.

Milestone-one tests may use in-memory or fixture adapters. They do not implement real operations, SQLite repositories, credential-store access, or encryption.

## 12. Milestone-One Scope

### 12.1 Included

- Architecture documentation and diagrams
- Architecture decision records
- Repository directory and workspace foundations
- Toolchain and package-manager manifests
- Cross-platform root verification commands
- Formatting and linting configuration
- CI workflows for supported platforms
- JSON Schema and OpenRPC foundations
- Representative contract examples and conformance fixtures
- Deterministic Rust and TypeScript type generation
- Minimal compile-only packages, crates, and contract test adapters
- Contribution and development documentation

### 12.2 Excluded

- Model-provider integrations
- Functional agent loops
- Functional filesystem, search, glob, shell, or write operations
- SQLite repositories and migrations
- Secret encryption or credential-store integration
- Functional session persistence
- Qt desktop, web, or mobile product interfaces
- Installers, packaging, auto-update, or code signing
- Functional cloud ingress, identity, workspace provisioning, isolation, scheduling, and hosted runtime infrastructure
- Parallel subagents within one session
- Portable database and master-key backup

### 12.3 Completion criteria

Milestone one is complete when:

1. A fresh checkout can be bootstrapped using documented prerequisites and commands.
2. One root verification command validates formatting, linting, contracts, generated outputs, documentation, builds, and tests.
3. Contract fixtures pass against both Rust and TypeScript adapters.
4. CI passes on Windows, macOS, and Linux.
5. Generated output is deterministic and current.
6. Architectural dependency rules are enforced automatically.
7. Documentation contains no unresolved design placeholders within milestone-one scope.
8. No excluded product behavior has been implemented.

## 13. Deferred Designs

The following areas require separate design and implementation cycles after the foundational harness:

1. Client-runtime HTTP and WebSocket API, authentication-token handoff, and discovery
2. Rust process supervision and RPC implementation
3. SQLite schema, migrations, credential-store abstraction, encryption algorithm, rotation, and backup recovery
4. Session event taxonomy, retention, projection schemas, and replay
5. Permission policy, capability model, sandboxing, and operation conflict control
6. Filesystem, search, glob, shell, write, and artifact operations
7. TypeScript agent loop, model-provider abstraction, context management, and approvals
8. Qt desktop application
9. Local web application
10. Mobile application
11. Cloud ingress, identity, tenant isolation, workspace provisioning, scheduling, and KMS integration
12. Distribution, packaging, updates, and platform signing

Each deferred subsystem must preserve the boundaries and invariants defined in this specification unless a superseding architecture decision record explicitly changes them.

