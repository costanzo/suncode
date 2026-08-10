# Test Plan

## Scope

Verify that the runtime registers DeepSeek, Zhipu GLM, and OpenAI; credential state is exposed per provider; Qt builds with provider-keyed settings UI; and existing DeepSeek behavior remains intact.

## Unit tests

- `cargo test -p suncode-runtime`

## Integration and conformance tests

- `/credentials` returns three provider states.
- `/models` returns three model entries and marks availability by provider credential state.
- Existing DeepSeek tool-call round trip remains green.

## Regression checks

- `cargo check -p suncode-runtime`
- `cmake --build apps/desktop-qt/build -j2`
- `qmllint apps/desktop-qt/qml/*.qml` when available.
- `node .agents/skills/impeccable/scripts/detect.mjs --json --scope layout apps/desktop-qt/qml/GlobalSettings.qml apps/desktop-qt/qml/ConversationPanel.qml`
- Offscreen Qt startup.
- `git diff --check`

## Manual checks

- Open Settings.
- Expand Model providers.
- Select DeepSeek, Zhipu GLM, and OpenAI and verify only the selected provider page is shown.
- Confirm an unconfigured provider disables composer submission for its selected model.

## Commands and results

- `cargo check -p suncode-runtime` passed.
- `cargo test -p suncode-runtime` passed: 18 tests.
- `cmake --build apps/desktop-qt/build -j2` passed, with existing macOS link target-version warnings.
- `qmllint apps/desktop-qt/qml/GlobalSettings.qml apps/desktop-qt/qml/ConversationPanel.qml` completed with warnings for source-tree import resolution and existing unqualified-access style.
- `node .agents/skills/impeccable/scripts/detect.mjs --json --scope layout apps/desktop-qt/qml/GlobalSettings.qml apps/desktop-qt/qml/ConversationPanel.qml` returned `[]`.
- Offscreen Qt startup reached a running app without QML load errors; the process was terminated after the bounded startup window.
- `git diff --check` passed.

## Residual risks

- Live provider calls were not run unless real user API keys are present.
- OpenAI-compatible semantics may require provider-specific tuning as more models are added.
