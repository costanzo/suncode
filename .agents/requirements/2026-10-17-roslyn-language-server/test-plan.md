# Test Plan

## Scope

Design review and production verification cover the catalog, Roslyn-specific Settings surface, Rust adapter, persistence, SDK bindings, and Avalonia projection.

## Unit tests

- Verify literal and recursive glob root matching, including configured-pattern priority.
- Verify version 6 persistence mapping, launch arguments, workspace settings, environment redaction, project isolation, and custom-solution rejection.
- Verify typed C# request serialization and the desktop `LanguageServerItem` projection.

## Integration and conformance tests

- Build the complete design-system application.
- Confirm the direct Roslyn route resolves without runtime errors.
- Start the installed Roslyn Language Server over stdio with production launch arguments and complete LSP initialization against a temporary .NET 10 project.

## Regression checks

- Confirm the existing four server-specific detail dispatch paths remain unchanged.
- Confirm the catalog reports five supported servers.
- Confirm Save and Reset are disabled for an unchanged draft and enabled after editing.
- Confirm custom project limit and custom solution fields appear only for their modes.

## Manual checks

- Review desktop and constrained-width layouts.
- Review light and dark themes.
- Exercise project loading, solution selection, restore, source generators, diagnostics, advanced disclosure, detection, enablement, save, reset, and retry states.

## Commands and results

- `npm run build` from `design-system/`: passed (268 modules transformed).
- Targeted Prettier check: passed.
- `git diff --check`: passed.
- Headless browser review: passed for the five-entry catalog, Roslyn direct route, light and dark themes, and 760px constrained width.
- Interaction review: Save and Reset start disabled; custom project-limit and custom solution fields appear on selection; Save enables after a draft change.
- Responsive review: the Settings panel and detail content have equal client/scroll widths at 760px with no horizontal overflow.
- `cargo test -p suncode-lsp`: passed, 26 tests.
- `cargo test -p suncode-data`: passed, 26 tests.
- Full Rust SDK tests: passed, 39 tests.
- Full C ABI tests: passed, 4 tests.
- Full Avalonia desktop tests: passed, 326 tests.
- Design-system production build: passed, 268 modules transformed.
- Targeted design-system Prettier check, Rust formatting checks, and `git diff --check`: passed.
- Installed `roslyn-language-server` stdio initialization with production arguments: passed.

## Residual risks

- The real-process smoke test proves startup and initialization on the current macOS arm64/.NET 10 installation; full MSBuild/NuGet/analyzer behavior still varies by project and tool version.
