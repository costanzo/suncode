# Desktop Native Menus

- Date: 2026-10-23
- Status: Implemented; runtime verification pending
- Related features: `avalonia-desktop-phase-1`
- Related specifications: `DESIGN.md`, Desktop Workspace design-system routes
- Related decisions: None

## Background

The Avalonia client has a small application menu and a project-specific menu labeled `Project actions`. Existing session, agent, and panel operations are discoverable mainly through controls inside the workspace.

## Goals

- Provide a familiar IDE-style native menu bar for ProjectHub and project windows.
- Make existing project, session, agent, layout, and window commands discoverable from menus.
- Preserve native edit behavior, existing confirmations, and correct enabled/checked state.
- Keep the written design authority and executable design-system specimen aligned with Avalonia.

## Non-goals

- Add command-search, new agent capabilities, terminal/build/run flows, Git mutations, or software update behavior.
- Change the agent SDK or persistence contracts.

## Requirements

- Project windows expose File, Edit, Navigate, Session, Agent, View, Window, and Help menus.
- ProjectHub exposes relevant File, Edit, Navigate, Window, and Help actions.
- Menu items call existing client actions; unavailable context disables the command.
- View and model selections show current state when their menus open.
- Undoing Agent file changes and archiving sessions retain explicit confirmation.
- Documentation specifies menu ownership and platform behavior; the design-system review surface includes the complete hierarchy and representative states.

## Edge cases

- A merged workspace menu must always act on the active project tab.
- Session/model/checkpoint menus must refresh when opened and tolerate an empty collection.
- Source Control is unavailable when system Git is unavailable.
- Model and reasoning choices must respect configuration and model capabilities.
- Small windows may suppress panes; view menu intent remains tied to the persisted pane state.

## Acceptance criteria

- All menu entries are implemented actions or disabled context-dependent commands.
- ProjectHub and project windows use consistent file/navigation/window/help terminology.
- Existing menu shortcuts match the shortcuts shown in Settings.
- Design-system specimen covers menu hierarchy and enabled, checked, and empty states.
- Focused Avalonia tests, desktop build, design-system build, `git diff --check`, and design-system visual review pass. Native macOS click-through remains pending because the automated UI session could not attach to the isolated app instance.
