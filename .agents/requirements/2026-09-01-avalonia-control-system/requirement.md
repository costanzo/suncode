# Requirement

## Background

The Avalonia desktop client already has approved tokens and surface layouts, but several reusable input and overlay patterns still live as page-local XAML in Settings and the workspace composer. That makes styling drift likely and slows follow-up work such as session rename dialogs, provider management, and shared path pickers.

## Goals

- Move repeated settings and composer primitives into reusable controls under `apps/desktop-avalonia/Controls/`.
- Introduce shared styled-system aliases for common control types so the desktop client can reuse one visual language.
- Replace settings-local provider, path-picker, dropdown, and dialog markup with reusable components.
- Keep the existing Rust SDK integration, bindings, and workflow behavior intact.

## Non-goals

- Redesign the approved desktop settings or composer workflows.
- Change provider persistence ownership, credential handling, or SDK contracts.
- Add unsupported desktop runtimes or design-system runtime dependencies.

## Requirements

- Add `SCFileSelector` for choosing either a file or a folder, configurable by parameter, with optional file-suffix filtering.
- Add `SCModal` as a reusable desktop overlay for small confirmation and edit flows, including session naming flows.
- Add `SCModelProvider` for settings provider management so provider overview and provider detail content are no longer written inline in the page.
- Add `SCComboBox` as a reusable flat or grouped selector and adopt it for settings selectors and the chat composer provider/model picker.
- Add reusable styled control aliases for toggle switches, numeric inputs, secret inputs, buttons, and text so the client can apply one system style globally.
- Keep the visual treatment aligned with the existing settings and desktop design-system specifications.

## Edge cases

- Folder selection and file selection must both return local filesystem paths when available.
- File filtering must normalize bare extensions such as `png` into picker patterns such as `*.png`.
- Grouped model selection must tolerate providers with no selectable models.
- Modal overlays must preserve Escape dismissal, overlay dismissal where enabled, focus restoration, and keyboard tab containment.
- Long provider URLs, model IDs, and filesystem paths must trim or wrap without introducing horizontal overflow.

## Acceptance criteria

- Settings uses reusable controls for path selection, provider management, and flat selectors.
- The chat composer uses the reusable grouped selector for model choice.
- Session naming overlays use the reusable modal component.
- Shared styles exist for button, text, toggle, numeric, and secret-input patterns and can be reused outside the initial screens.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj` succeeds.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj` succeeds.
- `git diff --check` succeeds.

## Open questions

- None.
