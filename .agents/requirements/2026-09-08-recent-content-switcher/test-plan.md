# Test Plan

## Scope

Design-system behavior, Avalonia ViewModel rules, and production-client regression coverage.

## Unit tests

- Focused ViewModel/model tests cover title precedence, mixed ordering, deduplication, the 20-item limit, current-item exclusion, previous-current visibility after switching, visible count/empty state, session rename/archive refresh, and project/dependency file identity.

## Integration and conformance tests

- Build the design system to validate imports, JSX, and CSS.

## Regression checks

- Confirm the existing project switcher and central Conversation/Editor transition remain available.
- Run `git diff --check`.

## Manual checks

- Open and dismiss the recent-content menu.
- Select a file and confirm title plus central Editor update.
- Select a session and confirm title plus central Conversation update.
- Confirm the current title is absent from the menu and the previous title appears after switching.
- Confirm each recent-content row, hover surface, and click target spans the dropdown's available width.
- Review hover/focus, light, dark, long-content, and constrained-width presentation.
- Inspect browser console errors.

## Commands and results

- `npm run build` in `design-system/` — passed.
- `git diff --check` — passed.
- Browser interaction review at `/projects/desktop/workspace` — passed in light/dark themes and at 620px width; no console warnings or errors.
- Session/file semantic color review — passed in light and dark themes; icons and type labels remain distinguishable without replacing text/icon cues.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj --no-restore` — passed with no warnings.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj --no-restore --filter FullyQualifiedName~RecentContentTests` — 6 passed.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj --no-restore` — 82 passed.
- Browser interaction review confirmed the current item is absent, the previous item returns after switching, and the visible count is correct in light and dark themes; no console warnings or errors.

## Residual risks

- Recent history intentionally resets with the project-window ViewModel and is not restored across application launches.
