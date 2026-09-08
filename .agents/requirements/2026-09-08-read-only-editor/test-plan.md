# Test Plan

## Scope

Design-system state coverage now; Avalonia file-view behavior after user confirmation.

## Unit tests

- ViewModel clears selected file when a session is selected.
- Read failures and empty files map to explicit editor states.

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

## Residual risks

- Exact AvaloniaEdit/TextMate package compatibility and grammar coverage remain implementation-time decisions.
