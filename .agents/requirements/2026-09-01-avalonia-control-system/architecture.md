# Architecture

## Current state

- Settings owns path selectors, provider overview, provider detail, and section-specific field styling inline in `SettingsWindow.axaml`.
- The workspace owns session overlays inline in `ProjectWorkspace.axaml`.
- The chat composer owns a one-off grouped model picker using `MenuFlyout`.
- Shared desktop styles exist, but there is no system-level Avalonia component layer for these recurring interaction patterns.

## Proposed design

- Add reusable controls under `apps/desktop-avalonia/Controls/`:
  - `SCFileSelector` wraps a text field plus picker button and opens either a file picker or folder picker.
  - `SCComboBox` wraps either a flat `ComboBox` or a grouped flyout trigger, depending on supplied data.
  - `SCModelProvider` renders provider overview cards and provider detail editing fields for settings.
  - `SCModal` provides a reusable overlay shell for small dialogs with shared actions and focus behavior.
- Add `Styles/Application/ControlStyles.axaml` for shared control aliases and `SCModal`'s template.
- Keep persistence and SDK calls in existing C# code-behind and view-model methods; the new controls stay presentation-only.

## Boundaries and dependencies

- Controls may consume Avalonia primitives, SVG icons, and existing desktop models.
- Controls must not access SQLite, project files directly, or model providers.
- Settings and workspace code-behind remain responsible for calling the SDK-facing `DesktopViewModel` methods.

## Data and control flow

- `SettingsWindow` updates reusable control properties from the existing `DesktopViewModel`.
- Reusable controls emit user-interaction events back to the host view.
- The host view translates those events into the existing save, remove, select, and rename flows.
- `SCComboBox` grouped selections map back to existing `ModelItem` instances so no provider/model persistence contract changes.

## Security and failure handling

- File selection returns only user-chosen local paths and does not read file contents.
- Credential text stays in-memory in the UI until the existing save command forwards it to the Rust SDK.
- Modal dismissal keeps current cancellation semantics; no operation is auto-approved or auto-saved.

## Compatibility and migration

- Existing views keep their x:Class names and event-driven flow.
- Shared control aliases are additive and do not require a broader app-wide migration in the same change.
- The new controls replace the current settings and session-dialog markup without changing persisted settings formats.

## Risks and rollback

- Avalonia template-part styling can drift if Fluent control structure changes; keep the overrides minimal and class-scoped.
- Grouped selector state could desynchronize if hosts rebuild item collections without reselecting the active item; hosts must refresh selected items after collection reloads.
- Rollback is straightforward because the change is isolated to the desktop presentation layer.

## Open questions

- None.
