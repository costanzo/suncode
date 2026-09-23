# Requirement

## Background

SunCode can continue a turn while its project window is behind another application or minimized. The desktop currently presents completion, failure, approval, and question state only inside an open project window. Users need an operating-system notification when background work finishes or requires attention, and selecting that notification must activate SunCode and navigate to the relevant session.

The existing agent event stream is session-scoped and the Avalonia client watches only the selected primary session. The desktop also has no single-instance activation IPC. The embedded agent must remain in the primary desktop process; notification activation must not become a second agent host or a general client transport.

## Goals

- Notify the user when background agent work completes, fails, or requires an approval or answer.
- Suppress system notifications whenever SunCode is the foreground application.
- Navigate a notification activation to the owning project and session.
- Support macOS, Windows, and Linux desktop environments.
- Route secondary-process activation to the existing desktop process through narrow local IPC.
- Preserve Rust ownership of session state and the existing embedded SDK boundary.

## Non-goals

- A daemon, tray-resident agent, background agent service, or second agent process.
- Cross-process access to the Rust SDK, SQLite, providers, operations, or live session streams.
- Remote push notifications or notification synchronization between machines.
- A durable general-purpose event journal.
- Notification actions that approve, deny, answer, retry, cancel, or otherwise mutate agent state directly from the operating-system notification.
- Completion notifications for child sessions.
- Notifications for cancelled or interrupted turns in the first delivery.
- An application-level notification preference or in-app notification center in the first delivery; the operating system remains the permission and delivery control surface.

## Requirements

### Notification eligibility

The desktop evaluates foreground state immediately before dispatching a notification. If any SunCode top-level window is active, SunCode is foreground and the notification is suppressed. This includes Workspace, ProjectHub, Settings, About, DialogWindow, and settings editor or confirmation windows owned by the application. A visible window that is minimized or behind another application does not make SunCode foreground.

A notification suppressed because SunCode was foreground is considered handled and is not emitted later merely because the application moves to the background.

When SunCode is not foreground, the desktop sends one system notification for each of these attention events:

| Event | Scope | Correlation key |
| --- | --- | --- |
| Successful turn completion | Primary sessions only | `turn_id` |
| Failed turn | Primary sessions only | `turn_id` |
| Approval requested | Primary and child sessions | `approval_id` |
| Question asked | Primary sessions only; child sessions cannot ask questions | `request_id` |

Cancelled and interrupted turns do not notify. Child-session completion and failure do not notify. A child approval does notify because it requires direct user action.

### Notification content

- The notification identifies SunCode, the project display name, and the session display title.
- Completion, failure, approval, and question notifications use distinct concise wording.
- Notification content must not include project absolute paths, prompts, assistant responses, tool arguments or results, provider errors, credentials, or other potentially sensitive content.
- Platform notification identifiers are stable for the attention-event correlation key so retries update or deduplicate the same notification rather than producing copies.

### Activation behavior

- Activating a primary completion or failure notification opens or activates the project window and selects the primary session.
- Activating a primary approval or question notification selects the primary session, where the existing review surface presents the pending item.
- Activating a child approval notification opens the parent primary session, opens the Child sessions surface, selects the child session, and presents its approval.
- If the request has already been resolved, the desktop still opens the relevant session but does not recreate the stale request.
- If the project or session no longer exists, is outside the current SDK user scope, or does not match the activation payload, the desktop rejects the route, records a bounded diagnostic, and activates an existing SunCode window without exposing an error notification.

### Single-instance activation IPC

- A primary desktop instance owns one activation endpoint for its agent data directory.
- A secondary process created by notification or URI activation sends one bounded activation request to the primary instance, waits for an acknowledgement, and exits before opening the Rust SDK.
- IPC messages carry only protocol version, request ID, activation kind, project ID, session ID, optional parent/child session IDs, correlation ID, and source. They never carry project paths or executable commands.
- Windows uses a current-user Named Pipe. macOS and Linux use a current-user Unix Domain Socket.
- IPC is an Avalonia desktop activation channel only. It does not attach to or proxy the agent SDK.

### Delivery and recovery

- Rust exposes a typed, bounded, global attention-event stream covering the four eligible event families across sessions owned by the SDK user.
- The stream remains a live notification surface rather than a durable event journal.
- The desktop keeps a bounded local delivery ledger keyed by attention-event correlation ID and disposition (`delivered` or `suppressed_foreground`).
- On stream lag or notification-monitor restart, the desktop reconciles against a bounded Rust-owned query derived from normalized turns, pending approvals, and pending questions, then applies its local ledger before dispatching.
- Ledger retention is bounded and must be long enough to cover the reconciliation window. It contains opaque identifiers and timestamps only, not notification text or conversation content.
- Notification transport or permission failures never change turn, approval, or question state and never surface as an agent failure.

### Platform behavior

- macOS uses the User Notifications framework and handles authorization denial without repeated prompting.
- Windows uses the supported desktop toast/app-notification activation mechanism for the selected packaging model and forwards secondary activation through the desktop IPC endpoint.
- Linux uses the freedesktop desktop notification service where available and supports default-action activation. Packaging installs the required desktop entry and activation metadata. Environments without actions may show a non-clickable notification but must not fail agent work.
- Platform backends report only bounded operational status to diagnostics.

## Edge cases

- Multiple projects and sessions may complete or request attention concurrently.
- An event may arrive while focus changes between applications; eligibility is decided once immediately before dispatch.
- A selected session may already have its ordinary UI watch while the global attention stream reports the same underlying state; the delivery ledger prevents duplicate system notifications.
- Approval and question requests may be resolved before their notification is activated.
- Notification activation may arrive before Avalonia initialization, project loading, or SDK startup completes; the activation router queues bounded requests until navigation is ready.
- A stale Unix socket or a primary process that exits during handoff must produce a bounded failure and safe retry of primary ownership, not two SDK hosts.
- Linux notification servers may omit action support or replace notifications according to desktop-specific policy.
- Notification permission may be denied or disabled after startup.
- An archived session may be opened only through an explicit supported reopen path; activation must not silently bypass session ownership checks.

## Acceptance criteria

- No system notification is sent while any SunCode window is the foreground application.
- A background primary turn completion produces exactly one notification and activation selects that session.
- A background primary turn failure produces exactly one notification and activation selects that session.
- A background primary approval or question produces exactly one notification and activation presents the pending item.
- A background child approval produces exactly one notification and activation opens the parent and child approval surface.
- Child completion, child failure, cancelled turns, and interrupted turns do not notify.
- Foreground-suppressed notifications are not replayed after SunCode moves to the background.
- Notification activation never opens a second Rust agent or SQLite owner.
- macOS, Windows, and Linux packaging and installed-application tests cover display and click activation.
- Focused tests cover eligibility, deduplication, IPC validation, startup races, activation routing, lag reconciliation, and clean shutdown.

## Open questions

- The exact Windows packaging/registration mechanism must be selected by an implementation feasibility spike before the backend is committed; the product behavior and IPC contract are already approved.
