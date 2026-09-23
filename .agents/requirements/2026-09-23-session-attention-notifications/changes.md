# Changes

## Source

- Add a Rust global typed attention hub and bounded candidate projection.
- Expose live attention subscription and reconciliation through Rust, C, and C# SDK layers.
- Add an application-scoped Avalonia attention coordinator, foreground evaluator, delivery ledger, activation router, and lifecycle integration.
- Add macOS, Windows, and Linux native notification backends.
- Add per-data-directory desktop single-instance coordination and activation IPC.
- Extend project/session navigation to accept validated ID-based activation routes, including child approvals.

## Contracts and generated artifacts

- Update `contracts/agent-sdk/README.md` with attention DTOs, stream lifecycle, lag behavior, and candidate reconciliation.
- Add a hand-written desktop activation IPC contract document under `contracts/` if implementation keeps framing outside the requirement package.
- Increment the C ABI version for new subscription/query symbols.
- No generated protocol artifacts are introduced.

## Configuration and persistence

- Add a versioned, bounded desktop notification ledger containing opaque correlation IDs, disposition, and timestamps.
- Do not add a SQLite table or runtime configuration setting in the first delivery.
- Add platform packaging metadata for notification activation and Linux desktop integration.

## Tests

- Rust attention filtering, ordering, lag, and candidate-query tests.
- C handle lifecycle and callback tests.
- C# typed DTO, subscription, and disposal tests.
- Avalonia foreground suppression, ledger, routing, startup queue, and coordinator lifecycle tests.
- Cross-platform IPC framing, ownership, validation, race, and shutdown tests.
- Installed-application notification display and click-activation smoke tests on macOS, Windows, GNOME, and KDE.

## Documentation

- Update architecture, SDK contract, decisions, release instructions, and implemented feature documentation when source and focused verification exist.
- Describe source implementation and automated verification precisely while keeping installed-platform display/click conformance explicitly pending until all four release smoke environments pass.
