# Test Plan

## Scope

Validate the Avalonia visual-resource layer, custom window frames, desktop layouts, state presentation, responsive behavior, and preservation of existing interaction wiring.

## Unit tests

- Run all tests in `apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj`.
- Add focused tests if responsive or navigation C# logic changes.

## Integration and conformance tests

- Build the production desktop project with .NET 10.
- Confirm XAML loading and compiled bindings through the build.
- Confirm no production reference to React, Vite, Node.js, Bun, or design-system source is introduced.

## Regression checks

- Project open, recent-project selection, and settings launch remain wired.
- Session navigation and workspace pane toggles remain wired.
- Source-control and provider-trace drawers remain mutually exclusive.
- Dialog confirmation and cancellation handlers remain wired.
- Theme changes continue to apply to all windows.

## Manual checks

- Compare ProjectHub, Workspace, and Settings against `design-system/src/projects/desktop/` in dark and light themes.
- Resize project and settings windows through normal, constrained, and minimum dimensions.
- Check focus visibility, hover/pressed/selected/disabled states, truncation, scrolling, empty states, warnings, and overlays.
- Check image attachment selection, three-image limit, thumbnail removal, full-size preview, and clearing after text submission; confirm attachment bytes are not sent by the existing request path.
- Check custom title-bar drag, double-click maximize, traffic lights, and native resize hit regions.

## Commands and results

- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj`: passed with zero warnings and zero errors after Stage 1.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj`: passed all 45 tests after Stage 1.
- `git diff --check`: passed after Stage 1.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj`: passed with zero warnings and zero errors after Stage 2.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj`: passed all 45 tests after Stage 2.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj`: passed with zero warnings and zero errors after Stage 3.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj`: passed all 48 tests after Stage 3, including three responsive workspace-layout tests.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj`: passed with zero warnings and zero errors after Stage 4.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj`: passed all 50 tests after Stage 4, including Explorer presentation-state tests.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj`: passed with zero warnings and zero errors after the Conversation visual pass.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj`: passed all 50 tests after the Conversation visual pass.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj`: passed with zero warnings and zero errors after the Review, Git, and Provider trace visual pass.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj`: passed all 50 tests after the Review, Git, and Provider trace visual pass.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj`: passed with zero warnings and zero errors after the Settings visual pass.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj`: passed all 50 tests after the Settings visual pass.
- Build and test should run serially because both write the desktop project's default `obj` output; a parallel attempt produced a transient PDB file contention before the serial test passed.
- The design browser could not be launched at requirement initialization because its local Vite dependency was not installed; source tokens, CSS geometry, and committed specimens were used for the initial comparison.
- `npm install` in `design-system/`: completed successfully; the Vite preview was started for manual comparison.
- `npm run build` in `design-system/`: passed with 162 modules transformed.
- Manual browser review: ProjectHub and Workspace checked in light and dark themes; Workspace checked at 390 x 844; all seven Workspace submodules and Settings loaded without document horizontal overflow; Conversation attachment guide confirmed the three-image, thumbnail, remove, and preview behavior.
- Final `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj --no-restore`: passed with zero warnings and zero errors.
- Final `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj --no-restore`: passed all 50 tests.
- Final `git diff --check`: passed.

## Residual risks

- Pixel-level cross-platform rendering still requires manual inspection on each supported desktop platform.
