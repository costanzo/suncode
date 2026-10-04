# Desktop Attention Notifications

User-facing behavior is in [`../features/session-attention-notifications/`](../features/session-attention-notifications/README.md). This spec records the boundary and security rules.

## Ownership

- Rust owns the typed attention events (`primary_turn_completed`, `primary_turn_failed`, `approval_requested`, `question_asked`), a bounded global attention hub separate from session event hubs, and a bounded attention-candidate query derived from normalized data for lag/restart reconciliation. The query is not an event journal.
- One global hub avoids opening a session subscription per running turn and lets an approval in an unselected child session reach the desktop.
- Avalonia owns notification wording, foreground policy, permission UX, native delivery, the delivery ledger, activation IPC, and navigation. None of these enter `sdks/rust`, `sdks/c`, SQLite, providers, operations, policy, or approval resolution. The CLI does not consume attention events.

## Delivery ledger

- Stored in `desktop-notification-ledger.json` under the data directory, capped at 2048 entries (`Infrastructure/NotificationLedger.cs`).
- Both delivered and foreground-suppressed correlation IDs are recorded. Transport failures are not recorded as delivered; they remain bounded retry candidates while the normalized state stays eligible.
- Foreground eligibility is checked once immediately before native dispatch.

## Activation IPC

Implemented in `Infrastructure/DesktopActivation.cs`.

- Instance identity derives from the canonical data directory, so separate development data directories do not share an endpoint.
- Windows: Named Mutex for primary ownership plus a current-user Named Pipe.
- macOS/Linux: advisory instance lock plus a Unix Domain Socket in a restricted per-user runtime directory (`$XDG_RUNTIME_DIR` on Linux, per-user temp on macOS). The endpoint name uses a bounded hash to stay within socket path limits. The server removes only a stale socket at its exact resolved path.
- A secondary process validates activation arguments, sends one length-prefixed UTF-8 JSON request, waits for an acknowledgement with the same request ID, and exits without constructing windows or opening the SDK. It retries endpoint discovery briefly and never opens the SDK while the primary lock is held.
- Requests are versioned; unknown kinds, oversized frames, invalid identifiers, trailing frames, and version mismatches are rejected. Messages carry opaque IDs only and convey intent, not authority. The primary revalidates project and session ownership through the SDK before navigating.
- The startup activation queue is bounded and processed serially after readiness; a later request with the same correlation key replaces a queued duplicate.
- Child-approval activation opens the parent session, selects the child, and shows the existing approval surface. It never resolves an approval from notification data.

## Failure handling

Notification, permission, D-Bus, COM/WinRT, socket, and pipe failures are diagnostic-only and cannot fail an agent turn. Clean shutdown stops intake, closes the attention stream and backend, stops IPC acceptance, drains or rejects queued activations, removes the owned socket, and then releases the instance lock.
