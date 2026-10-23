# Progress

- Status: Testing
- Last updated: 2026-10-23

## Completed

- Provider-neutral layer in `suncode-computer`: `AccessibilityBackend`, role/state vocabulary, bounded walk, redaction, inverse coordinate mapping, outline rendering.
- `computer_accessibility_tree` tool: parsing, validation, observe risk, catalog gating, collection outside the executor lock, frame-epoch staleness check, typed errors, redacted result value.
- macOS AX collector, run against a live iTerm2 window on a 1920×1080 display: 27 nodes, no truncation, plausible roles and boxes. Clicking box centers and Retina scaling are not yet exercised.
- Windows UIA and Linux AT-SPI2 collectors, compiled and clippy-clean for their targets.
- Core: outline redaction in compaction checkpoints and approval continuations, retention of the two newest outlines, backend installation, system guidance.
- Runtime-info fields through the Rust SDK and C# DTO; Avalonia Settings row with en-US/zh-CN strings; `DESIGN.md` and design-system specimen.
- Contract, feature doc, and `ADR-20261023-computer-accessibility-tree`.

## In progress

- Live conformance: macOS click alignment and Retina; all of Windows and Linux (see `test-plan.md`).

## Blocked

- Windows and Linux conformance needs those desktops; it cannot run on the macOS development host.

## Log

### 2026-10-23

- Requirement initialized. Scope confirmed with the user: read-only tree with boxes mapped to screenshot pixels, input stays coordinate-based, and macOS, Windows, and X11 Linux ship in the same delivery.
- Implementation landed. Deviations from the architecture draft:
  - Native bindings are hand-written over `core-foundation` 0.10.1, `windows` 0.62.2, and `dbus` 0.9.12, all already in the lockfile, rather than `objc2-application-services` or `atspi`.
  - Outline text is marked by a fixed first line (`OUTLINE_MARKER`) instead of a new content-part kind, so provider adapters are unchanged.
  - macOS falls back to the front window server window's owner when the system-wide `AXFocusedApplication` query fails, which happens for some host processes.
  - The macOS backend opens its window-server connection at construction, because the first CoreGraphics query can take seconds and would otherwise consume the collection deadline.
  - The walk always describes the root, so a slow start still returns a usable node.
