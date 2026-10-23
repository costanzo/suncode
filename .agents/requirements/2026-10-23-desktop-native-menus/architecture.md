# Architecture

## Current state

`App` attaches a native menu containing About and Window actions. `WorkspaceWindow` attaches Project actions and Window menus, while ProjectHub has no native window menu. UI commands are implemented by the active workspace view model and controls.

## Proposed design

Keep menu composition in the Avalonia window code-behind because the menus are transient presentation and invoke existing view actions. The active WorkspaceWindow is the source of menu state, including when hosted by MergedWorkspaceWindow. ProjectHub builds its own contextual menu. The application menu owns About and Settings entry points; standard system lifecycle items remain platform-owned.

The workspace menu groups commands under File, Edit, Navigate, Session, Agent, View, Window, and Help. `NeedsUpdate` refreshes dynamic session/project/checkpoint/model entries and stateful menu checks before display. ProjectWorkspace exposes narrow internal action methods for the native menu and existing controls to share behavior.

The design-system route under Projects → Desktop → Workspace documents the menu hierarchy and representative state behavior. `DESIGN.md` remains normative.

## Boundaries and dependencies

- Avalonia owns presentation and calls existing DesktopViewModel / ProjectWindowManager methods.
- Rust SDK and persistence contracts do not change.
- Native edit operations target the currently focused editable or selectable Avalonia control.
- Help links only to existing local in-app shortcuts and About surfaces.

## Data and control flow

Opening a menu requests a refresh. Current view model, layout, Git availability, and project-window state determine menu checks and enablement. Selecting an item calls the active window's existing UI action or routes through the existing app services.

## Security and failure handling

Confirmation dialogs remain required for session archival and checkpoint restoration. Pending approvals/questions are only navigated to; menus never approve them. Failed SDK operations retain current error/status behavior.

## Compatibility and migration

Menus remain hand-built Avalonia native menus. macOS keeps About and Settings in the application menu; Windows/Linux expose Settings from File. `⌘,` / `Ctrl+,`, `⌘1` / `Ctrl+1`, and `⌘9` / `Ctrl+9` retain existing behavior.

## Risks and rollback

Native menu backends can differ in menu ordering and command gestures. Keep platform checks narrow and inspect macOS plus a non-macOS build. The menu layer can be reverted without data migration.

## Open questions

- None.
