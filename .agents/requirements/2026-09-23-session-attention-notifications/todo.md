# Todo

## Design

- [x] Confirm notification event scope.
- [x] Confirm foreground suppression semantics.
- [x] Confirm primary and child-session behavior.
- [x] Approve desktop-only activation IPC boundary.
- [x] Select the Windows packaging and notification activation registration mechanism through a focused spike.
- [x] Review and approve the final attention and IPC wire contracts before source implementation.

## Implementation

- [x] Implement Rust attention events and reconciliation query.
- [x] Implement Rust/C/C# bindings.
- [x] Implement the desktop notification coordinator and ledger.
- [x] Implement foreground detection.
- [x] Implement single-instance activation IPC.
- [x] Implement activation routing and navigation.
- [x] Implement macOS notification delivery and activation.
- [x] Implement Windows notification delivery and activation.
- [x] Implement Linux notification delivery and activation.
- [x] Add platform packaging metadata.

## Verification

- [x] Run Rust focused tests.
- [x] Run C and C# SDK focused tests.
- [x] Run Avalonia focused tests.
- [x] Run IPC concurrency and startup-race tests.
- [ ] Run installed macOS smoke tests.
- [ ] Run installed Windows smoke tests.
- [ ] Run installed GNOME and KDE smoke tests.
- [x] Run `git diff --check` and broader repository checks appropriate to the final diff.

## Closeout

- [x] Update implemented feature records.
- [x] Update current SDK and IPC contracts.
- [x] Update architecture and deferred-scope wording to implemented status.
- [x] Record final verification and residual platform limitations.
