# Architecture

## Current state

The Avalonia client persists global settings through the Rust SDK generic settings API. Theme loading and application already use `theme_mode`, `DesktopViewModel`, and an `App.ThemeChanged` broadcast. UI strings are mostly hard-coded in XAML and C#; no localization service or resource dictionaries exist.

## Proposed design

Add an Avalonia-owned `LocalizationService` with:

- supported locale catalog (`en-US`, `zh-CN`)
- current locale and `LanguageChanged` event
- resource dictionary loading and English fallback
- formatted lookup for C# and ViewModel messages

Add `ui_locale` to the existing global Settings rows. `DesktopViewModel` loads it during initialization and saves it when the Appearance control changes. `App` subscribes to the service and propagates resource changes to every open window.

## Boundaries and dependencies

- Rust remains the owner of persisted settings; the desktop does not read SQLite.
- Avalonia owns presentation resources and translation lookup.
- Model/provider/project/user-authored text remains data and is not passed through localization.
- No new runtime package is required.

## Data and control flow

```text
Settings ComboBox
  -> DesktopViewModel.SaveLanguageAsync
  -> SDK SetSetting(global, ui_locale)
  -> LocalizationService.SetLocale
  -> replace Avalonia resource dictionary + LanguageChanged
  -> all open windows and localized message bindings refresh
```

## Security and failure handling

Locale values are an allowlisted presentation setting. Invalid persisted values fall back to English and are not sent to providers or tools. Resource-load failures also fall back to the embedded English dictionary.

## Compatibility and migration

Existing installations have no `ui_locale` row and therefore use English. No database migration is required. Existing `theme_mode` behavior remains unchanged.

## Risks and rollback

The main risk is stale translated text held in ViewModel properties. User-facing dynamic messages must store a key and arguments or be recomputed when `LanguageChanged` fires. Removing the locale row or disabling the service restores English behavior without affecting agent state.

## Open questions

Future RTL locales may require layout direction support; this delivery keeps locale metadata extensible but does not implement RTL.
