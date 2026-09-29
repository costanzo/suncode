# Requirement

## Background

The Kotlin Multiplatform Mobile client presents an English-only Compose UI. It needs the same immediate English/Simplified Chinese language selection and persistence behavior as the Avalonia client.

## Goals

- Add a global interface-language setting in Mobile Settings.
- Support `en-US` and `zh-CN` with stable locale values and native names.
- Apply the selected language immediately across phone and tablet Compose surfaces.
- Persist the selection on Android and iOS.
- Keep user/project/session/provider content unchanged.

## Non-goals

- Translating Remote Server data, model output, project names, paths, or protocol identifiers.
- OS-language auto-detection beyond the existing default English fallback.

## Requirements

1. Unsupported or missing locale values fall back to `en-US`.
2. Shared Compose code owns presentation resources and language switching.
3. Android persists the locale in app-private SharedPreferences; iOS persists it in UserDefaults.
4. Language selection is available from Settings and updates all open Compose surfaces.
5. Focused common tests cover locale normalization, native names, formatting, and persistence.

## Edge cases

- A language change while a pairing or session dialog is open updates its labels immediately.
- Pairing payloads, host names, project names, session titles, messages, and server errors remain authored data.
- Tablet and phone layouts use the same locale state.

## Acceptance criteria

- Users can choose English or 简体中文 from Mobile Settings.
- Selection survives Android/iOS process restart.
- Sessions, Hosts, Settings, pairing, approval/question, and theme dialogs update immediately.
- Shared tests and Android compilation pass.
