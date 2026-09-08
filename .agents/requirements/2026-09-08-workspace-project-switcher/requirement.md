# Requirement

## Background

Workspace currently shows the active project name in its custom title bar, but switching projects requires leaving the current window or using a native application menu. The title should expose the same open and recent-project workflow as ProjectHub.

## Goals

- Make the current project title visibly interactive with a trailing chevron.
- Put `Open project` first and reuse ProjectHub's local-folder picker flow.
- Show recent projects directly below the open action.
- Open a recent project in a new Workspace window, or focus its existing Workspace window when already open.

## Non-goals

- Changing project identity, persistence, or the Rust SDK contract.
- Replacing ProjectHub or the native application menu.
- Opening multiple projects in one Workspace window.

## Requirements

1. The Workspace title bar shows the current project name and a trailing chevron in one accessible menu trigger.
2. The anchored menu contains `Open project`, a divider, and recent project rows showing name and path.
3. Recent rows do not expose whether a project already has an open window.
4. Project selection preserves the one-project-per-window model.
5. The menu closes after selection, on Escape, and when focus is dismissed outside the menu.
6. The design remains usable in light and dark themes and at the 620 DIP Workspace minimum width.

## Edge cases

- Long project names and paths elide without widening the menu or title bar.
- A stale recent path follows the same error handling as ProjectHub.
- Duplicate selections activate the existing window instead of creating another one.
- When no recent rows are available, `Open project` remains available.

## Acceptance criteria

- The design-system Workspace overview demonstrates the open menu and all required row content.
- Production implementation uses the existing project picker and duplicate-window activation flow.
- Focused tests cover new-window and existing-window selection behavior.
- Build, formatting, theme, keyboard, and constrained-width checks pass.

## Open questions

- Awaiting design approval before production implementation.
