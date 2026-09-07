# Test Plan

## Scope

Design-system review and Avalonia desktop projection, drawer, and live-event behavior.

## Unit tests

- Snapshot projection groups tools by turn and retains bounded user-message previews.
- Live `tool.output` chunks append to the selected tool activity item.
- Compact tool states distinguish succeeded, active, and failed calls.

## Integration and conformance tests

## Regression checks

## Manual checks

- Tool tree expansion and detail selection.
- Standalone no-turn, active-turn, and completed-turn primary states.
- Running live-output state and tail-follow affordance.
- Turn hover user-message preview.
- Inline active-tool jump affordance.
- Compact-width stacking and focus visibility.

## Commands and results

- `npm run build`: passed, 189 modules transformed.
- `git diff --check`: passed.
- Browser console warning/error check: passed with no entries.
- Responsive inspection at 1280px and 620px: passed without horizontal overflow.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj --no-restore`: passed.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj --no-restore`: passed, 59 tests.

## Residual risks

- Live output is best-effort by contract and may be absent after a snapshot resync; the terminal result remains authoritative.
