# Architecture

## Current state

The approved architecture identifies a Qt desktop client but explicitly excludes its product implementation from the foundational milestone. No desktop source project, UI framework choice beyond Qt, client-runtime API, client adapter, or UI behavior currently exists.

The draft three-layer protocol record proposes authenticated WebSocket JSON-RPC between clients and the TypeScript runtime, ordered `session.event` notifications, snapshots, replay, and capability negotiation. Those proposals are useful dependencies but are not yet canonical contracts.

There is a documentation discrepancy to resolve before implementation: the summary for `ADR-20260804-foundational-architecture` in `../../DECISIONS.md` says Rust owns SQLite, sessions, and approvals, while the newer approved `../../ARCHITECTURE.md` assigns runtime state and SQLite to TypeScript and authoritative OS-capability enforcement to Rust. This UI design follows `../../ARCHITECTURE.md` and does not attempt to settle the historical ADR text.

## Proposed design

### Technology direction

- Use Qt 6 for the desktop application.
- Prefer Qt Quick/QML for the presentation layer, subject to a proof of concept for text selection, accessibility, virtualization, rich Markdown, and diff performance.
- Keep client protocol types and transport adapters outside QML. Expose presentation-safe view models to QML.
- Use Qt facilities for windowing, accessibility, settings that are strictly device-local, native dialogs, clipboard access, notifications, and platform integration.
- Do not embed a browser runtime or use Electron.

This direction is a proposal, not an approved technology decision. A focused Qt Widgets comparison is required if the proof of concept exposes blocking accessibility or text-rendering limitations.

### Presentation modules

The eventual application should be divided by user-facing responsibility:

- `application-shell`: lifecycle, top-level navigation, window state, theme, and global shortcuts.
- `runtime-connection`: discovery, authentication handoff, capability negotiation, health, reconnect, and event resume.
- `projects`: recent projects, folder selection, availability, and active project presentation.
- `sessions`: session list, lifecycle actions, search, unread state, and session header.
- `conversation`: message projection, streaming, activity rows, composer, attachments, and turn control.
- `approvals`: approval presentation, grant-scope choices, decision submission, and resolved audit state.
- `inspector`: changes, file preview, diff, and activity detail.
- `settings`: runtime-backed settings forms and device-local presentation preferences.
- `diagnostics`: redacted health, versions, capabilities, and support information.

Module names describe responsibilities and do not require creating source directories until the implementation milestone needs them.

### State ownership

| State | Authoritative owner | Client responsibility |
| --- | --- | --- |
| Session messages, events, status, and history | TypeScript runtime | Render projections and reconcile ordered updates |
| Project identity and availability | Runtime, backed by Rust validation where required | Selection and presentation |
| Turns and cancellation | TypeScript runtime | Submit intent and show acknowledged state |
| Approval requests and records | TypeScript runtime, with Rust enforcing OS capabilities | Explain request and collect explicit decision |
| Filesystem and process results | Rust through the runtime | Render safe typed results |
| Global and project settings | TypeScript runtime | Edit through client API |
| Window geometry, pane widths, visual density | Qt client | Persist per device without secrets |
| Unsent drafts | To be decided | Preserve locally or through a draft API |

The UI must not infer success from button clicks. Mutating controls enter a pending state and commit visible state only after runtime acknowledgement or corresponding ordered events.

## Boundaries and dependencies

```text
Qt desktop views
      |
presentation view models and client adapter
      |
hand-written client protocol adapter
      |
authenticated HTTP/WebSocket API
      |
TypeScript agent runtime
      |
private core protocol client
      |
Rust OS core
```

- The desktop application depends only on Qt, presentation-specific libraries, and its own hand-written client protocol adapter.
- QML or widget code must not import runtime or Rust implementation packages.
- The client-runtime contract must expose presentation-safe domain methods; privileged Rust RPC shapes must not leak into the desktop API.
- The runtime owns authentication, authorization context, durable state, event ordering, and idempotency.
- The client owns only ephemeral presentation state and explicitly classified device-local preferences.

## Data and control flow

### Startup

1. The shell renders a non-interactive startup state.
2. The connection adapter discovers or receives a configured runtime endpoint.
3. The adapter completes authenticated connection and capability negotiation.
4. The client requests initial project, session-list, settings, and connection projections.
5. The client enables mutations only after required projections and capability checks complete.

### Session open and event resume

1. The user selects a session.
2. The client requests a snapshot plus the current per-session event sequence.
3. The client renders the snapshot and subscribes to ordered events.
4. Each event is deduplicated and applied only in sequence.
5. On a gap, the client pauses mutation for the affected session and requests replay.
6. If replay is unavailable, the client replaces the affected projection from a fresh snapshot and informs the user.

### Turn submission

1. The client validates only presentation constraints and submits input with a unique idempotency key.
2. The composer becomes pending but retains its content until acknowledgement.
3. The runtime acknowledges or rejects the request with a typed reason.
4. Ordered session events drive queued, working, message, activity, approval, failure, cancellation, and completion presentation.
5. The draft clears only when submission is acknowledged.

### Approval

1. The runtime emits a typed approval request tied to a session event and OS operation.
2. The client renders the exact safe description and only the grant scopes offered by the runtime.
3. The user explicitly chooses deny or an offered scope.
4. The client submits the decision with an idempotency key and disables duplicate actions.
5. Runtime events determine the final approval and operation state; Rust remains the enforcement boundary.

## Client state model

The client should use normalized projections keyed by stable runtime identifiers rather than storing duplicated view-specific copies. At minimum it needs:

- Connection and negotiated-capability state.
- Project summaries and active project selection.
- Session summaries, active session selection, and per-session event cursor.
- Conversation projections and transient streaming buffers.
- Pending client mutations keyed by idempotency key.
- Approval prompts keyed by approval identifier.
- Inspector selection and diff/file/activity projections.
- Runtime-backed settings plus separately stored device-local preferences.
- Per-session composer drafts, with ownership pending a later decision.

Reducers or equivalent state transitions must be deterministic and testable without rendering the full application.

## Required client-runtime API capabilities

The UI requirement creates a backlog for a separate contract design. The eventual API needs capabilities in these domains:

| Domain | Needed operations or events |
| --- | --- |
| Runtime | initialize, capabilities, health, versions, reconnect, diagnostics |
| Project | list recent, open local, select, availability, settings |
| Session | list, snapshot, create, rename, pin, archive, delete, search |
| Turn | submit, acknowledge, cancel, retry semantics, status |
| Events | subscribe, ordered sequence, replay, snapshot fallback |
| Approval | request event, offered scopes, decide, expiry, resolution |
| Models | list available models/modes, defaults, validation |
| Context | attach file reference, text attachment, limits, removal before send |
| Artifacts | metadata, bounded content, diff, file preview, pagination |
| Settings | schemas, values, validation, update, secret replace/remove |
| Notifications | actionable session state suitable for native notification |

Contract design must decide whether initial data uses HTTP snapshots plus WebSocket events or JSON-RPC methods over one WebSocket.

Because there is no generated SDK, this client hand-writes its adapter against the contract document. The adapter is a single isolated layer — view models and QML never encode or decode transport messages themselves — and it is verified against the shared test vectors, which are the only protection against drifting from the runtime's implementation.

## Security and failure handling

- Authenticate before accepting or rendering project-scoped data.
- Bind every request and subscription to authorized runtime, project, and session scope.
- Treat runtime text as untrusted and use a safe Markdown subset with code execution disabled.
- Permit only explicit, validated navigation for file references and external URLs.
- Keep credentials out of URLs, ordinary payloads, local preferences, crash data, and UI logs.
- Redact sensitive notification text when the screen is locked or notification previews are disabled.
- Apply bounded buffering and backpressure to event streams and tool output.
- Preserve drafts and current selections during recoverable disconnects.
- Prevent mutations on projections known to have an event gap.
- Display typed errors with a user action when one exists and a correlation identifier safe for diagnostics.

## Compatibility and migration

- The desktop client and runtime negotiate API versions and optional capabilities.
- Unsupported optional features are hidden or disabled with an explanation; version behavior is never inferred from build numbers alone.
- Unknown compatible event fields are ignored. Unknown required event types trigger safe resynchronization rather than speculative rendering.
- Device-local UI preferences must be versioned and resettable independently of runtime settings.
- There is no existing UI data to migrate.

## Risks and rollback

- **Qt rendering risk:** rich selectable Markdown and large diffs may perform poorly. Mitigate with an early proof of concept and bounded virtualization tests.
- **Contract timing risk:** UI work may invent backend behavior before API design. Mitigate by completing the client-runtime contract before production screens.
- **State divergence risk:** reconnects and multiple clients may produce stale UI. Mitigate with ordered events, idempotent mutations, replay, and snapshot replacement.
- **Approval ambiguity risk:** compact UI can hide meaningful scope. Mitigate with explicit scope text, stronger treatment for persistent grants, and security review.
- **Scope growth risk:** editor, terminal, Git, and cloud administration features could overwhelm Phase 1. Keep them outside the primary acceptance path.
- **Platform inconsistency risk:** custom chrome may harm native behavior and accessibility. Prefer standard window behavior and validate every supported OS.

Because this delivery contains documentation only, rollback consists of superseding or revising this requirement. No product data or source implementation is affected.

## Open questions

- Resolve the architecture/ADR ownership discrepancy described in Current state.
- Approve Qt Quick/QML or select Qt Widgets after the proof of concept.
- Define the first supported OS matrix and packaging constraints.
- Approve the client-runtime transport and local authenticated handoff.
- Decide how much of the adapter can be shared with the CLI, given no generated SDK and two languages.
- Define draft ownership and background-session behavior after the UI closes.

