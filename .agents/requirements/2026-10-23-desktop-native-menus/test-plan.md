# Test Plan

## Scope

Native menu composition, contextual enablement/check state, and actions routed into existing client behavior.

## Unit tests

- Assert required menu groups and no placeholder commands.
- Assert Git/model/checkpoint availability updates menu item state.

## Integration and conformance tests

- Build Avalonia desktop and execute its focused test suite.
- Build the design-system project.

## Regression checks

- Check shortcut labels against existing keyboard bindings and Settings.
- Verify dynamic project/session menus handle empty and populated collections.
- Check `git diff --check`.

## Manual checks

- Inspect design-system specimen in dark and light themes at desktop and constrained widths.
- Inspect actual macOS app and workspace menus if the runtime is available.

## Commands and results

- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj --no-restore -p:SkipFontPreparation=true` passed.
- Focused menu, localization, and navigation shortcut tests passed: 34/34.
- `npm run build` in `design-system/` passed; Vite emitted only its existing large-chunk warning.
- `git diff --check` passed.
- Full desktop suite: 341 passed, 4 failed, 0 skipped. The failures are in unmodified browser responsive assertions and duration formatting assertions outside this change.

## Residual risks

- OS-native menu rendering and standard application lifecycle entries vary by platform.
- The isolated macOS app could not be attached by the UI automation environment, so native click-through remains unverified.
