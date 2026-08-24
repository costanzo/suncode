# Test Plan

## Scope

Verify static multi-model registration, route-specific wire model requests, provider-level availability, session validation, Qt selector behavior, and removal of provider model/endpoint environment reads.

## Unit tests

- Registry returns at least two models per provider.
- Every stable model ID resolves to the expected provider and wire model.
- Shared adapter sends the wire model supplied by the route.
- Agent round trips continue to use the selected model.

## Integration and conformance tests

- `/models` returns twelve catalog entries and two models per provider.
- OpenAI model IDs are independently selectable and persisted.
- Unknown model IDs remain rejected.

## Regression checks

- `cargo test -p suncode-agent`
- `cargo check -p suncode-agent`
- `cmake --build apps/desktop-qt/build -j2`
- `qmllint apps/desktop-qt/qml/features/settings/GlobalSettings.qml`
- Impeccable layout detector for the settings surface.
- Bounded offscreen Qt startup.
- `git diff --check`

## Manual checks

- Open Settings > Defaults and verify all catalog models are selectable.
- Select both OpenAI models and verify the stable selected ID changes.
- Start a session with a non-default model and verify the turn uses that model route.

## Commands and results

- `cargo test -p suncode-agent` passed: 25 tests.
- `cargo check -p suncode-agent` passed.
- `cmake --build apps/desktop-qt/build -j2` passed, with the existing macOS target-version linker warnings.
- `qmllint apps/desktop-qt/qml/features/settings/GlobalSettings.qml` completed with the existing import/unqualified-access warnings.
- Impeccable layout detector returned `[]` for the settings surface.
- Offscreen Qt startup stayed alive through the bounded startup window without QML load errors; it was then terminated deliberately.
- `git diff --check` passed.

## Residual risks

- Live vendor entitlement and model availability are not tested without user credentials.
