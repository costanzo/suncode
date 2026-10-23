# Implementation Plan

1. [x] Confirm the requirement, open questions, and new native dependencies.
2. [x] Define the provider-neutral model in `suncode-computer`: `AccessibilityBackend`, request/snapshot/node types, role and state vocabulary, bounding, redaction, outline rendering, and inverse coordinate mapping. Use a fake backend for unit tests.
3. [x] Wire the `accessibility_tree` action, tool definition, risk, catalog gating, short-lock frame copy, collector thread, cancellation, generation re-check, and redacted result.
4. [x] Add core retention of the newest tree outlines and update system guidance.
5. [ ] Implement the macOS AX collector (done, live tree verified on a 1920×1080 display) and verify click alignment on Retina and non-Retina displays (pending).
6. [ ] Implement the Windows UIA collector (done) and verify it under 100%, 150%, and mixed per-monitor DPI (pending).
7. [ ] Implement the Linux AT-SPI2 collector on X11 (done) and verify bus availability, extents, and redaction (pending).
8. [x] Add runtime-info fields through the Rust SDK, C ABI, and C# binding with focused contract tests.
9. [x] Update `DESIGN.md` and the design-system specimen, then implement the Avalonia Settings row and strings.
10. [ ] Run the focused and broader verification in `test-plan.md` (macOS done; Windows/Linux live and visual checks pending).
11. [x] Promote behavior to `features/computer-use/`, update contracts, and record the ADR.
