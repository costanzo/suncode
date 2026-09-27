# Test Plan

## Scope

Verify Mobile SSE DTOs, cached cursor compatibility, Android packaging/tests, and Kotlin Native compilation.

## Unit tests

- Decode a `session.snapshot` envelope and event sequence fields.
- Preserve compatibility with cached projections that omit new cursor maps.

## Integration and conformance tests

- Remote Server SSE replay, heartbeat, `401`, and `410` behavior require server-owned tests and are not available in this repository.

## Regression checks

- Android Debug APK assembly.
- Android host tests.
- iOS Simulator shared and test-source compilation.
- YAML parsing and `git diff --check`.

## Manual checks

- Validate foreground/background connection lifecycle on Android and iOS devices after Remote Server deployment.

## Commands and results

- `./gradlew :androidApp:assembleDebug :shared:testAndroidHostTest :shared:compileKotlinIosSimulatorArm64 :shared:compileTestKotlinIosSimulatorArm64` — passed.
- OpenAPI and SSE AsyncAPI YAML parsing — passed.
- `git diff --check` — passed.

## Residual risks

- No deployed Remote Server was available to verify the live SSE endpoint or platform lifecycle behavior on devices.
