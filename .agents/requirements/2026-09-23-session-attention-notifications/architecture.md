# Architecture

## Current state

Rust publishes typed events to bounded, session-scoped streams. Avalonia atomically watches only the selected primary session and closes that watch when selection changes. Consequently, the desktop cannot reliably observe completion or attention state for another primary session or a child approval.

The desktop creates several independent windows in one process and reuses a shared native SDK handle through managed wrappers. It has project-window coordination inside `App`, but process startup has no desktop single-instance lock, activation router, URI handler, or local IPC endpoint. The Rust data-directory lock prevents a second agent owner only after the secondary process attempts to open the SDK, which is too late for notification activation handoff.

## Proposed design

### Rust attention projection

Add a typed `AgentAttentionEvent` family derived at the Rust-owned lifecycle boundary:

- `primary_turn_completed`
- `primary_turn_failed`
- `approval_requested`
- `question_asked`

The envelope carries stable project/session/turn correlation and, for a child approval, parent and child session identity. It does not contain notification wording or presentation policy. Core publishes eligible events into a separate bounded global attention hub in addition to the existing session event hub. The global hub prevents the desktop from opening one session subscription per running turn and allows an approval in an unselected child session to reach the application coordinator.

The Rust SDK exposes:

- a typed attention event stream for live delivery;
- a bounded attention-candidate query over normalized terminal turns, pending approvals, and pending questions for lag/restart reconciliation.

The candidate query is not a new event journal. It derives current and recent facts from normalized data, uses a bounded time window and result limit, and returns stable correlation identifiers so the desktop ledger can deduplicate.

The C ABI adapts the typed stream to an opaque callback subscription with explicit close semantics. The managed C# SDK owns callback lifetime and typed deserialization. This is separate from the selected-session dormant watch because notification monitoring begins at application initialization and does not project conversation UI.

### Desktop notification coordinator

One application-scoped `SessionAttentionCoordinator` owns:

- the managed attention subscription;
- foreground eligibility evaluation;
- the bounded notification delivery ledger;
- project/session label resolution through named SDK calls;
- native notification backend selection;
- lag reconciliation and retry policy;
- notification activation handoff to the activation router.

The coordinator runs independently of project-window ViewModels. Closing or switching one project window therefore does not lose monitoring for turns still owned by the same embedded agent. It starts after the shared SDK is ready and closes before the final SDK reference is released.

The foreground evaluator treats SunCode as foreground when any application top-level is active. Eligibility is evaluated once immediately before native dispatch. Both delivered and foreground-suppressed events enter the ledger. Transport failures do not enter the ledger as delivered; they are bounded retry candidates while their normalized state remains eligible.

### Notification backends

`INativeNotificationBackend` exposes initialization/permission status, notification delivery, activation callbacks, and shutdown. Implementations are desktop presentation infrastructure:

- macOS: User Notifications framework delegate and response handling;
- Windows: desktop app notification/toast registration and activation for the selected packaging model;
- Linux: freedesktop notifications over D-Bus with default action when advertised by the server.

The backend receives already-redacted display text and an opaque activation token. It does not receive an SDK handle or authorization to mutate agent state.

macOS authorization is requested in response to the first accepted primary turn submission rather than at bare application startup. If an eligible event races the unresolved permission request, the coordinator keeps that event only until the permission result is known and then either dispatches or records a bounded permission failure. Windows and Linux initialize their registration without a foreground permission prompt.

### Activation router and IPC

`DesktopActivationRouter` is the only component that converts activation messages into navigation. It accepts both in-process backend callbacks and requests received over local IPC. Requests are versioned and validated before they enter a bounded startup queue.

The desktop instance identity is derived from the canonical agent data directory so independently configured development data directories do not share an activation endpoint.

- Windows uses a Named Mutex for primary ownership and a Named Pipe created for the current user only.
- macOS and Linux use an advisory instance lock plus a Unix Domain Socket inside a per-user restricted runtime directory. Linux prefers `$XDG_RUNTIME_DIR`; macOS prefers the per-user temporary directory. The endpoint name uses a bounded hash of the data-directory identity to avoid Unix socket path limits.

A secondary process performs only activation bootstrap:

1. Parse and validate the OS activation arguments.
2. Discover that the primary instance owns the desktop lock.
3. Connect to the activation endpoint.
4. Send one length-prefixed UTF-8 JSON request.
5. Receive an acknowledgement with the same request ID.
6. Exit without constructing Avalonia windows or opening the agent SDK.

IPC requests support only `activate.session`. Unknown message types, oversized frames, invalid identifiers, extra trailing frames, and version mismatches are rejected. The channel conveys intent, not authority; the primary process revalidates project and session ownership through the SDK before navigation.

### Navigation

The application-level navigation path resolves project and session IDs through SDK-backed collections, opens or activates the existing project window, selects the session through the existing ViewModel path, and then focuses the relevant surface.

For a child approval, the route identifies both parent and child. The router opens the parent primary session, refreshes child sessions, selects the child, opens the Child sessions panel, and relies on the existing approval projection. It never resolves an approval directly from notification data.

Activation requests may arrive before framework initialization, SDK readiness, or project-window construction. The router retains a small bounded queue and processes requests serially after readiness. A later request for the same correlation key replaces an earlier queued duplicate.

## Boundaries and dependencies

- Rust remains the source of truth for projects, sessions, turns, approvals, questions, and ownership validation.
- The attention stream is a Rust SDK feature; notification wording, foreground policy, permission UX, native delivery, IPC, and navigation remain Avalonia responsibilities.
- The activation IPC never enters `sdks/rust`, `sdks/c`, SQLite, providers, operations, policy, or approval resolution.
- The CLI does not listen to desktop attention events and does not connect to desktop activation IPC.
- Existing selected-session atomic watch behavior remains unchanged.
- No design-system specimen is required for native notification chrome. Any later in-app notification setting, status, banner, or notification center must first update `DESIGN.md` and `design-system/`.

## Data and control flow

### Background event delivery

1. Rust projects a turn, approval, or question state into normalized storage.
2. Rust publishes the ordinary session event and, when eligible by type/scope, one typed attention event.
3. The desktop coordinator receives the attention event.
4. The coordinator checks its ledger.
5. The coordinator evaluates whether any SunCode window is foreground.
6. If foreground, it records `suppressed_foreground` and stops.
7. If background, it resolves bounded display labels, sends the native notification, and records `delivered` after backend acceptance.

### Activation

1. The operating system invokes the notification backend callback or launches SunCode with activation arguments.
2. An existing primary process routes an in-process callback directly; a secondary process forwards the same typed activation request through IPC and exits.
3. The primary activation router validates the request and waits for application readiness.
4. The router opens or activates the project window and selects the target session.
5. Approval/question routes expose the existing pending interaction surface without performing the interaction.

### Reconciliation

1. The attention stream reports lag or the coordinator restarts.
2. The coordinator calls the bounded Rust attention-candidate query using its last reconciliation watermark with an overlap window.
3. Results are ordered deterministically by occurrence time and correlation ID.
4. The desktop ledger removes delivered and foreground-suppressed candidates.
5. Remaining candidates pass through the current foreground and native-delivery policy.
6. A fresh live stream is established.

## Security and failure handling

- Named Pipe ACLs and Unix runtime-directory/socket permissions restrict IPC to the current OS user.
- The Unix server validates socket ownership, removes only a stale socket at its exact resolved endpoint, and never recursively deletes a runtime directory.
- IPC frames and startup queues have small fixed limits and timeouts.
- Activation messages carry opaque IDs only and cannot request operations, paths, commands, approvals, or question answers.
- All identifiers are revalidated through Rust-owned SDK methods before navigation.
- Notification text is intentionally content-free beyond product, project display name, session title, and attention kind.
- Notification, permission, D-Bus, COM/WinRT, socket, or pipe failures are diagnostic-only and cannot fail an agent turn.
- If a secondary process cannot contact the primary during a bounded startup race, it retries endpoint discovery briefly. It must not open the SDK while the primary lock remains owned.
- Clean shutdown stops intake, closes the attention stream and native backend, stops IPC acceptance, drains or rejects queued activation requests, removes the owned Unix socket, and then releases the instance lock.

## Compatibility and migration

The Rust/C/C# additions are additive at the source level but require a C ABI version increment because new native symbols and subscription handles are introduced. Existing session event envelopes and `watch_session` behavior do not change.

The desktop ledger is a versioned local presentation-state file, not a SQLite schema change. Unknown future ledger versions fail closed by skipping notification replay rather than mutating or deleting the file.

macOS bundle metadata, Windows packaging/registration metadata, and Linux desktop entry files require release integration. Development execution that is not installed may display notifications but is not considered proof of click activation.

## Risks and rollback

- Platform notification activation APIs differ substantially and may impose packaging requirements not exercised by current macOS-only publish instructions.
- Linux notification action support varies by server and desktop environment.
- Foreground detection can race focus changes; the accepted rule is a single check immediately before dispatch.
- A global attention stream adds another bounded fan-out path in core and must not block session event publication.
- Reconciliation may rediscover old facts; the desktop ledger and bounded window prevent duplicate delivery.
- A malformed single-instance design could prevent startup or create two SDK hosts. Startup-race tests and strict lock ownership are required before platform rollout.

Rollback disables notification coordinator startup and platform registration, removes the desktop activation endpoint, and leaves all normalized agent state intact. The additive SDK attention APIs may remain unused without changing existing clients.

## Open questions

- Select and prove the supported Windows packaging/registration mechanism in a focused spike before implementation of the Windows backend.
