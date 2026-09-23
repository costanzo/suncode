# Implementation Plan

1. [x] Run a focused Windows packaging/app-notification activation spike and record the selected registration model without changing product semantics.
2. [x] Finalize hand-written attention DTOs, reconciliation bounds, and desktop activation IPC framing.
3. [x] Implement the Rust global attention hub and normalized candidate query with primary/child filtering.
4. [x] Add Rust SDK, C ABI, and C# subscription/query bindings with explicit close and lag behavior.
5. [x] Implement the desktop delivery ledger, foreground evaluator, and application-scoped attention coordinator.
6. [x] Implement the versioned per-data-directory single-instance lock, IPC transports, startup queue, and activation router.
7. [x] Implement ID-based navigation for primary completion/failure/approval/question routes and child approval routes.
8. [x] Implement macOS User Notifications authorization, delivery, and activation.
9. [x] Implement Windows app notification delivery, activation registration, and IPC forwarding for a secondary activation process.
10. [x] Implement Linux freedesktop notification delivery, capability detection, default-action activation, and desktop entry integration.
11. [ ] Add focused unit, contract, concurrency, lifecycle, and installed-application tests. (Automated coverage complete; installed smoke tests pending.)
12. [x] Update current contracts, architecture, feature records, release instructions, and the final verification record.
