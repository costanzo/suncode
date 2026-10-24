# Test Plan

## Scope

Native menu composition, contextual enablement/check state, and actions routed into existing client behavior.

## Unit tests

- Assert required menu groups and no placeholder commands.
- Assert Git/model/checkpoint availability updates menu item state.
- Assert Dock menu separates actual open projects from the recent catalog, refreshes empty states, limits recent items, disambiguates duplicate names, and rechecks availability at invocation.

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
- Focused menu, Dock, localization, and project-window coordination tests passed: 31/31.
- `npm run build` in `design-system/` passed; Vite emitted only its existing large-chunk warning.
- `git diff --check` passed.
- Full desktop suite's previous completed run: 341 passed, 4 failed, 0 skipped. The failures are in unmodified browser responsive assertions and duration formatting assertions outside this change. A rerun during Dock work reported the same four failures but stalled afterward and was cancelled.

## Residual risks

- OS-native menu rendering and standard application lifecycle entries vary by platform.
- The isolated macOS app could not be attached by the UI automation environment, so native click-through remains unverified.
- The Dock review launch reported an already-active local agent and was stopped. Dock rendering remains unverified; design-system empty/modal states were reviewed in light/dark themes at 620px.
