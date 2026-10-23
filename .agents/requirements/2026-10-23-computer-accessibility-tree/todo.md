# Todo

## Design

- [x] Confirm the requirement and resolve open questions (ceilings, retention count).
- [x] Review the architecture, especially collection outside the executor mutex.
- [x] Approve new native dependencies and pinned versions.
- [x] Specify the Settings accessibility row in `DESIGN.md` and the design-system specimen.

## Implementation

- [x] Provider-neutral types, vocabulary, bounding, redaction, and outline in `suncode-computer`.
- [x] Action, tool definition, risk, catalog gating, and runtime dispatch.
- [x] Core provider-context retention and system guidance.
- [x] macOS AX collector.
- [x] Windows UIA collector.
- [x] Linux AT-SPI2 collector (X11).
- [x] Runtime-info fields through the Rust SDK, C ABI, and C#.
- [x] Avalonia Settings row and localization.
- [x] Unit and contract tests; live conformance on macOS only.

## Verification

- [x] Focused `suncode-computer` and core tests.
- [x] SDK, C ABI, and C# contract tests.
- [ ] Per-platform conformance fixtures, including box alignment and redaction (macOS done; Windows and Linux pending).
- [ ] Design-system and Avalonia visual comparison in both themes (builds and view-model tests pass; visual comparison not yet run).
- [x] `git diff --check`.

## Closeout

- [x] Update `features/computer-use/` and `contracts/agent-sdk/README.md`.
- [x] Record the ADR in `DECISIONS.md`.
- [ ] Remove this package once it no longer explains an active decision.
