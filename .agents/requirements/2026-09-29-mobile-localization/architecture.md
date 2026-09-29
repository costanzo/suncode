# Architecture

## Current state

Mobile uses shared Kotlin Multiplatform Compose UI in `shared/src/commonMain`, with Fake and Remote repositories. Presentation strings are concentrated in `App.kt`.

## Proposed design

Use a shared `MobileLocale` catalog, `MobileStrings`, and `LocalMobileStrings` composition local. Native locale stores implement a small common interface using SharedPreferences on Android and UserDefaults on iOS. `AppState` owns the active locale and persists changes from Settings.

## Boundaries and dependencies

Localization remains client presentation state. Remote DTOs, repositories, and server data are not translated.

## Compatibility and migration

No protocol or server changes are required. Existing installs without a locale use English.
