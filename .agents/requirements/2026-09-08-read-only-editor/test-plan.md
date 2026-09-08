# Test Plan

## Scope

Design-system state coverage and the implemented Avalonia/Rust file-view path.

## Unit tests

- ViewModel clears selected file before both new-session loading and same-session early return.
- Read failures and empty files map to explicit editor states.
- File extensions map to display labels and TextMate grammar extensions.

## Integration and conformance tests

- Explorer file click requests the bounded read path and opens the editor.
- Dependency file reads remain read-only and project-bound.

## Regression checks

- Existing session selection, conversation composer, review, drawers, and Explorer expansion remain unchanged.

## Manual checks

- Review `/projects/desktop/workspace/editor` in light/dark themes and at constrained width.
- Verify editor text can be selected but no edit affordance or caret is presented.

## Commands and results

- Passed: `npm run build` from `design-system/`.
- Passed: `git diff --check` from the repository root.
- Passed: browser checks for light/dark themes, 620px constrained layout, file selection, editor state switching, and session restoration.
- Passed: `cargo fmt --all -- --check` from `agent/`.
- Passed: `cargo test -p suncode-tool reads_bounded_utf8_files_without_following_symlinks` from `agent/`.
- Passed: `cargo test project_dependencies_are_read_only_and_browsed_on_demand` from `sdks/rust/`.
- Passed: `cargo test --manifest-path sdks/c/Cargo.toml`.
- Passed: `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj --no-restore`.
- Passed: `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj --no-restore` (76 tests).
- Passed: production implementation `git diff --check`.
- Passed: headless rendered-frame regression check for visible AvaloniaEdit text, line numbers, and TextMate highlighting after merging the required Fluent control style.

## Residual risks

- Unsupported file extensions intentionally fall back to selectable plain text without a TextMate grammar.
- TextMate grammar coverage comes from `TextMateSharp.Grammars` 2.0.3 through the integration package.
