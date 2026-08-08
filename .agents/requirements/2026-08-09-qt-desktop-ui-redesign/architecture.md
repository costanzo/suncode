# Architecture

## Current state

The Qt Quick client is a fixed three-column `RowLayout` using stock controls and one dark color token object. Presentation state and runtime-facing behavior are already separated: QML calls the existing `RuntimeClient`, while durable state remains Rust-owned.

## Proposed design

Use a standalone project hub as the app entry point. The hub owns no selected project and only displays the runtime project list plus global settings/open-project actions. Opening a project creates an independent `ProjectWindow.qml` instance; each project window owns its own `RuntimeClient` presentation state and selects that project through the SDK facade. The hub is hidden while project windows exist and is shown again when the last project window closes.

Retain the three functional regions inside each project window but make the side regions presentation-only collapsible tool bays. Add a small reusable QML component vocabulary for icon buttons, actions, section labels, and selection controls. Expand `Theme.qml` into semantic color, spacing, radius, and typography tokens. Keep all runtime calls and DTO consumption unchanged.

The chosen visual direction is **Quiet Control Desk**: matte graphite surfaces, hairline separators, restrained teal interaction color, amber approval state, and a spacious conversation canvas. It draws on professional editing consoles without literal hardware decoration.

## Boundaries and dependencies

- Changes are limited to `apps/desktop-qt/qml/`, the C++ Qt adapter, QML module declarations, and durable design/requirement records.
- Collapse state is transient Qt presentation state.
- No Rust SDK, protocol, database, or provider dependency changes.

## Data and control flow

Runtime DTOs continue to populate the existing QML properties. The hub uses `RuntimeClient.autoSelectProject: false` and loads projects without loading a session snapshot. New top-level toggle controls change only `ApplicationWindow` Boolean properties that drive side-panel width and visibility.

## Security and failure handling

Approval, credential, undo, and diagnostic operations keep their existing RuntimeClient pathways. The redesign must not hide risk copy or imply stronger reversibility than the runtime provides.

## Compatibility and migration

No stored data migration is required. Qt 6.5 remains the minimum version.

## Risks and rollback

The main risk is QML layout regression at compact desktop widths. The source change is isolated and can be reverted without affecting runtime state.

## Open questions

- None.
