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

- Pending.

## Residual risks

- OS-native menu rendering and standard application lifecycle entries vary by platform.
