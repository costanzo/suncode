# Requirement

## Background

The Avalonia desktop client currently presents English-only UI text. Users need to choose Chinese or English from Settings, with the selected language applied across the desktop client and persisted for later launches.

## Goals

- Add a global interface-language setting to Settings → Appearance.
- Support English (`en-US`) and Simplified Chinese (`zh-CN`) now.
- Apply a language change immediately to all open desktop windows.
- Keep the setting extensible for future locales without changing persisted values or control logic.
- Preserve user content, model output, file paths, provider names, and code as authored data.

## Non-goals

- Translating model responses or project content.
- Adding system-language auto-detection in the first release.
- Localizing the Rust agent's protocol, tool names, or persisted domain values.
- Adding a separate settings database or client-side persistence path.

## Requirements

1. Persist the selected locale as a global configuration value named `ui_locale`.
2. Missing or invalid values fall back to `en-US`.
3. Language options use stable BCP-47 values and native display names.
4. XAML resources use dynamic resource lookup so open windows update without recreation.
5. C# and ViewModel-generated user-facing messages resolve through the same localization service.
6. Resource keys are checked for parity between supported locales.
7. The Settings design specimen and durable design guidance describe the language row and immediate-apply behavior.

## Edge cases

- A language resource key missing from a non-default locale falls back to English.
- A locale change while a modal Settings window is open updates Settings and its owner window together.
- Existing status messages are re-rendered after a language change instead of retaining their previous translation.
- Long Chinese labels must remain usable at the existing Settings minimum width.

## Acceptance criteria

- Users can select English or 简体中文 from Settings → Appearance.
- Selection is saved through the existing SDK settings boundary and restored after restart.
- ProjectHub, Workspace, Settings, About, dialogs, notifications, and local status/validation copy update when the language changes.
- Unsupported or missing locale values safely display English.
- Focused localization tests and the Avalonia test project pass.

## Open questions

- Whether a future release should add a `System default` option. This delivery does not add it.
