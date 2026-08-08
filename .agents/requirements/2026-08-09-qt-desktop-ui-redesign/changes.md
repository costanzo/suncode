# Changes

## Source

- Added semantic Qt/QML theme tokens and shared `AppButton`, `AppField`, and `SectionLabel` primitives.
- Rebuilt `Main.qml`, `ConnectionPanel.qml`, `ConversationPanel.qml`, and `ReviewPanel.qml` around the Quiet Control Desk direction.
- Added standalone project hub, separate project windows, global settings, a model selector beside the composer, and a right-side agent/process panel.
- Added independent navigation/review collapse controls and improved empty, approval, credential, runtime, undo, and active-turn states.
- Removed the visible Activity tab for now; the runtime event/activity data remains intact for a future dedicated surface.
- Fixed hub startup so recent projects load without auto-selecting a project or requesting an empty session snapshot.
- Fixed QML theme binding shadowing in hub/settings/project windows so shared controls receive the intended design tokens.
- Restyled the composer into a floating card with rounded corners, shadow, model selector chip, and icon-only send/stop action.
- Added tonal separation and shadow transitions between the side bays and the central conversation region.
- Deepened the sidebar surfaces and raised the central workspace contrast so the panel boundaries read more clearly in dark mode.
- Added user-selectable light and dark appearance modes in global settings and wired them across hub, project, and main windows.

## Contracts and generated artifacts

- No protocol or generated artifact changes planned.

## Configuration and persistence

- Added user-scoped settings calls for the default model. Provider credentials remain in the OS credential store; no database schema changes were made.

## Tests

- `cmake --build apps/desktop-qt/build -j2` passed.
- Offscreen Qt startup reached the project hub without QML TypeError or Runtime SDK connection-race errors.
- `node .agents/skills/impeccable/scripts/detect.mjs --json` returned `[]`.
- `git diff --check` passed.

## Documentation

- Added this requirement package and updated durable Qt desktop feature/design notes.
