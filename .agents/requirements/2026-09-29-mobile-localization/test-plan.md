# Test Plan

## Scope

Locale normalization, persistence, immediate Compose state changes, and Android/iOS wiring.

## Unit tests

- Supported codes and native names.
- Invalid locale fallback.
- Formatted English/Chinese strings.
- In-memory persistence.

## Regression checks

- Existing repository and pairing behavior remains unchanged.
- User-authored data remains unchanged.

## Commands and results

- `./gradlew :shared:testAndroidHostTest :androidApp:compileDebugKotlin` — passed on 2026-09-29.
- `./gradlew :shared:iosSimulatorArm64Test` — attempted; Kotlin/Native link is blocked by missing usable Xcode (`xcrun xcodebuild -version`).

## Residual risks

- Accessibility labels are either localized alongside visible controls or intentionally null for decorative icons. Server-originated errors and user-authored content remain data-driven by design.
