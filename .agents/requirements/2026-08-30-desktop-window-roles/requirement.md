# Requirement

## Background

The Avalonia client used one `MainWindow` to switch between the project hub and project workspace, while several secondary windows duplicated custom title bars.

## Goals

- Define five top-level window roles: ProjectHub, Workspace, DialogWindow, Settings, and About.
- Keep the custom title bar only on Workspace.
- Use system title bars and controls for ProjectHub, Settings, and About.

## Requirements

- ProjectHub is an independent startup window.
- Each opened project gets an independent Workspace window.
- Workspace uses `WindowDecorations=BorderOnly`.
- ProjectHub, DialogWindow, Settings, and About use full system decorations and contain no duplicate title bar controls.
- DialogWindow is the sibling desktop surface for secondary confirmation; it starts closed and opens a confirmation dialog only after an explicit action click.

## Acceptance criteria

- `MainWindow` is absent from the production Avalonia source.
- Opening, closing, settings, about, theme propagation, and project-window recovery continue to work.
- Focused .NET tests and design-system build pass.
