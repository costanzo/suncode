# Test Plan

## Scope

Verify six-provider registration, credential state, model availability, shared compatibility-adapter behavior, Qt settings integration, and regression safety for existing providers.

## Unit tests

- `cargo test -p suncode-runtime`
- Registry resolves all six stable model IDs.
- Credential state returns all six providers and isolates configured state.
- Shared compatibility adapter continues to normalize text, tool calls, usage, and cancellation.

## Integration and conformance tests

- `/credentials` returns six redacted states.
- `/models` returns six entries with provider-specific availability.
- Generic credential routes accept Kimi, Claude, and Gemini and reject unknown IDs.

## Regression checks

- `cargo check -p suncode-runtime`
- `cmake --build apps/desktop-qt/build -j2`
- `qmllint apps/desktop-qt/qml/features/settings/GlobalSettings.qml`
- `node .agents/skills/impeccable/scripts/detect.mjs --json --scope layout apps/desktop-qt/qml/features/settings/GlobalSettings.qml`
- Bounded offscreen Qt startup.
- `git diff --check`

## Manual checks

- Open Settings and expand Model providers.
- Select Kimi, Claude, and Gemini and verify only the selected provider page appears.
- Save and remove each key and confirm model availability changes.

## Commands and results

- `cargo test -p suncode-runtime` passed: 25 tests.
- `cargo check -p suncode-runtime` passed.
- `cmake --build apps/desktop-qt/build -j2` passed, with the existing macOS target-version linker warnings.
- `qmllint apps/desktop-qt/qml/features/settings/GlobalSettings.qml` completed with the existing import/unqualified-access warnings.
- `node .agents/skills/impeccable/scripts/detect.mjs --json --scope layout apps/desktop-qt/qml/features/settings/GlobalSettings.qml` returned `[]`.
- Offscreen Qt startup stayed alive through the bounded startup window without QML load errors; it was then terminated deliberately.
- `git diff --check` passed.

## Residual risks

- Live provider calls require user credentials and were not run in repository verification.
- Vendor behavior beyond the documented OpenAI-compatible surfaces remains out of scope.
