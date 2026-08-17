# Test Plan

## Scope

Verify the Avalonia build, C ABI loading, view-model projections, and unchanged Rust SDK behavior.

## Unit tests

- Event and snapshot projection tests for messages, activity, approvals, touched paths, and terminal turn state.
- SDK envelope parsing tests for success, error, and malformed responses.

## Integration and conformance tests

- Build the dynamic Rust SDK and Avalonia desktop app together.
- Run the existing runtime SDK and contract vector tests.

## Regression checks

- Run the Rust workspace test suite.
- Confirm the Avalonia production build has no Qt dependency and the Qt parity source remains complete.

## Manual checks

- Launch the project hub without configured credentials.
- Inspect the project window at minimum and default sizes.
- Open settings and verify dark/light switching.

## Commands and results

- `cargo fmt --manifest-path runtime/Cargo.toml --all -- --check`: passed.
- `cargo test --manifest-path runtime/Cargo.toml --workspace`: passed, 43 tests.
- `dotnet restore apps/desktop-avalonia/SunCode.Desktop.csproj`: passed without package downgrade or compatibility warnings; all Avalonia runtime assemblies resolved to `12.1.1`.
- Avalonia 12 extension compatibility: removed the Avalonia 11 `Avalonia.Svg.Skia` and `Markdown.Avalonia.Tight` binaries after the SVG renderer raised a runtime `TypeLoadException`; restore now resolves `Svg.Controls.Skia.Avalonia 12.0.0.15` and `ClassIsland.Markdown.Avalonia.Tight 12.0.0`.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj --no-restore`: passed on Avalonia `12.1.1` with 0 warnings and 0 errors.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj -c Release --no-restore`: passed on Avalonia `12.1.1` with an optimized Rust `cdylib`, 0 warnings, and 0 errors.
- `dotnet format apps/desktop-avalonia/SunCode.Desktop.csproj --verify-no-changes --no-restore`: passed.
- `node .agents/skills/impeccable/scripts/detect.mjs --json apps/desktop-avalonia DESIGN.md`: returned `[]`.
- Release startup: loaded the native runtime, listed projects, selected a project, and rendered runtime/Git projections without console errors.
- Screenshot checks: passed for the Qt/Avalonia hub, 1440x900 workbench, 900x620 compact workbench, settings, Git drawer, and in-window session dialog.
- Multi-window checks: passed for shared native runtime ownership, two simultaneous project windows, duplicate-project activation/disablement, dynamic recent-project menus, and hub restoration after the last project closes.
- Interaction checks: passed for project/session navigation, native project menu, title chrome, responsive resizing, Git drawer controls, settings, keyboard dismissal, composer gating, and Qt-derived focus styling.
- Component extraction check: the Avalonia Release build passed after moving the hub, workspace, chat area, project sidebar, agent sidebar, and Git viewer into independent compiled `UserControl` files.
- Avalonia 12 compatibility check: restore must resolve all core Avalonia assemblies to `12.1.1`, use Avalonia 12-native SVG and Markdown packages, then Debug and Release builds must complete without package downgrade or compatibility warnings.
- macOS parity follow-up: passed for project-window drag (AX position changed from `135,80` to `255,140`), traffic-light asset hover, branded Dock icon, native Cocoa full-screen transition animation and full-display client coverage, composer-overlay layout, and Markdown rendering against existing persisted OpenCode-project session messages.
- macOS native-shadow follow-up: Debug build passed with 0 warnings and 0 errors; window-only captures for the `980x712` hub, `900x672` settings window, and `1440x900` project window each included the expected 56pt native shadow extent on every side, retained custom traffic lights without duplicate system chrome, and preserved project-window dragging from `(135,80)` to `(235,140)`.
- Window chrome color follow-up: dark and light window-only captures confirmed the dedicated 0.5-DIP outer hairline remains visible but subordinate to the native shadow, renders as one physical pixel on a Retina display without broken corners, and leaves internal panel and control borders at their existing contrast.
- Composer keyboard behavior was validated through the tunnel-routed input handler: plain Enter is consumed and submitted after synchronizing the field text; Shift+Enter remains available to the multiline editor. A live provider turn was intentionally not sent during UI verification.
- Full-screen regression follow-up: the Debug build, C# format verification, layout audit, and `git diff --check` passed after routing transitions through `WindowState.FullScreen`, restoring normal chrome before native exit, keeping one transparent-capable composition surface, and removing manual frame overrides; the Avalonia 12.1.1 macOS backend source confirms this path invokes native `NSWindow.toggleFullScreen:`. Live full-screen interaction remains for user verification because computer-use testing was explicitly excluded.

## Residual risks

- Platform signing, notarization, distributable installers, destructive checkpoint restore, approval continuation, and live provider calls were not exercised against production credentials.
