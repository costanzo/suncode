# Changes

## Source

- Added Tool activity navigation, standalone specimens, and the Workspace bottom drawer composition.
- Refined Conversation to show assistant messages, turn markers, hover/focus user-message previews, and one active-tool jump row.
- Added the Avalonia `ToolActivityViewer` and a turn-grouped client projection sourced from existing snapshots and `tool.*` events.
- Replaced the inline operation-detail modal with the mutually exclusive Tool activity bottom drawer.

## Contracts and generated artifacts

- No production protocol or generated artifact changes.

## Configuration and persistence

- No changes.

## Tests

- Built the design-system production bundle.
- Verified wide and 620px layouts, marker hover, active-tool selection, and console output in the in-app browser.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj --no-restore`: passed.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj --no-restore`: passed, 59 tests.

## Documentation

- Updated `DESIGN.md` with the assistant-first history and Tool activity interaction contract.
