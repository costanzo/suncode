# Progress

- Status: Implemented; runtime verification pending
- Last updated: 2026-10-23

## Completed

- Read repository and design-system guidance; reviewed current Avalonia menu and command implementations.
- Implemented context-aware native menus for ProjectHub, Workspace, and merged project windows.
- Added native edit commands, dynamic session/model/checkpoint/project/window entries, pane checks, and pending approval/question navigation.
- Added the Workspace design-system specimen, menu CSS, and normative `DESIGN.md` hierarchy.
- Added focused menu structure and empty-state tests.
- Added the macOS Dock menu through Avalonia `NativeDock`, including live open projects, up to ten recent projects, the project picker, and ProjectHub actions. Open projects include merged tabs and use existing window activation.
- Added Dock tests for empty state, duplicate names, recent order/limit, live-registry recheck, and modal disablement.

## Verification gap

- Native macOS click-through is pending because the UI automation environment could not attach to the isolated app instance. No native runtime pass is claimed.
- An isolated Dock review launch was stopped after the local agent reported an already-active instance. Native Dock rendering and click-through remain unverified.

## Log

### 2026-10-23

- Initialized the delivery package after confirming the accepted full IDE-style menu direction.
- Completed implementation and focused verification. The full desktop suite still reports four failures in unmodified browser responsive expectations and turn-duration formatting.
- Reviewed the Workspace route in light theme and at the constrained 620px width; the interactive menu fixture exposes platform, context, checked, disabled, and empty states without overflow in the constrained layout.
- Reviewed the Dock specimen with empty light-theme and modal-disabled dark-theme states at 620px; neither has horizontal overflow.
