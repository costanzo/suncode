# Test Plan

## Scope

CEF adapter compatibility, loopback URL policy, preview pane lifecycle, responsive layout, and design-system rendering.

## Unit tests

- `BrowserPreviewTests`: loopback-only navigation and wide/narrow layout behavior.

## Integration and conformance tests

- Avalonia desktop build with CEF compatibility project.
- Rust tool process and SDK compile checks for preview lifecycle.
- Preview configuration validation and reload-generation polling should be covered by the Rust integration suite when persisted project fixtures are added.
- Packaged CEF startup, helper process, localhost load, input focus, and HMR on all supported targets.
- macOS arm64 publish must contain `Chromium Embedded Framework.framework` and four CEF helper apps under `SunCode.app/Contents/Frameworks`.

## Regression checks

- Existing desktop test suite and design-system build.

## Manual checks

- Toggle Browser preview from the workspace gutter; check stopped/live/loading states, reload, close, and narrow-window behavior.

## Commands and results

## Residual risks

- Rust development-server lifecycle and CEF package signing are not implemented yet.
