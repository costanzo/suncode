# Test Plan

## Scope

Workspace title-bar project switcher presentation, dismissal, project opening, and duplicate-window activation.

## Unit tests

- Verify title-bar drag detection ignores the project switcher trigger and popup.
- Verify recent-project menu state derives from current project records.

## Integration and conformance tests

- Verify `Open project` uses the existing picker/application path.
- Verify a closed recent project creates a Workspace window.
- Verify an already-open recent project activates its existing Workspace window.

## Regression checks

- Existing ProjectHub open/recent flows continue to work.
- Native project actions and title-bar dragging continue to work.

## Manual checks

- Light and dark themes.
- Hover, focus, open, Escape, outside dismissal, and item activation.
- Long project names/paths and 620 DIP constrained width.

## Commands and results

- `npm run build` from `design-system/`: passed on 2026-09-08.
- `git diff --check`: passed on 2026-09-08.

## Residual risks

- Avalonia popup placement and native title-bar pointer routing require production verification after design approval.
