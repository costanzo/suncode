# Requirement

## Background

The Avalonia desktop module layout has grown monolithic and mixes window-level views, view-model responsibilities, UI models, and application styles.

## Goals

- Organize `Views/` by window role.
- Split `DesktopViewModel` into smaller partial files.
- Split `Models/UiModels` by module.
- Remove unused `DesktopViewModel` members.
- Split `App.axaml` styles into named child files.

## Non-goals

- No UI redesign.
- No behavior changes beyond deleting unused code.

## Requirements

- `Views/ProjectHub` contains the hub window and hub surface.
- `Views/ProjectWorkspace` contains the workspace window, chat, navigation, and review surfaces.
- `DesktopViewModel` compiles as multiple partial files.
- `Models/UiModels` is split into module-based files.
- `App.axaml` includes child style/resource files instead of one large inline block.

## Edge cases

- Existing tests must continue to pass.
- Namespace and XAML class names must remain aligned after moving files.

## Acceptance criteria

- The desktop project builds successfully.
- Desktop tests pass.
- The diff has no formatting issues.

## Open questions

- None.
