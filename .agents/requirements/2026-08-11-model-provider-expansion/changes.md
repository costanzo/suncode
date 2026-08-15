# Changes

## Source

- Added provider-keyed credential state for DeepSeek, Zhipu GLM, and OpenAI.
- Added a model provider registry that resolves the selected SunCode model ID to the matching runtime provider.
- Added an OpenAI-compatible provider adapter for Zhipu GLM and OpenAI using streamed `/chat/completions` requests.
- Shared canonical message normalization and SSE parsing across built-in providers.
- Updated model defaults to `glm-5.2` for Zhipu GLM and `gpt-5.6-sol` for OpenAI.
- Updated Qt runtime client credential APIs from DeepSeek-only calls to generic provider-keyed save/remove calls.
- Updated global settings to show provider-specific pages under the Model providers tree.
- Updated the conversation composer to disable sending when the selected model provider is not configured.

## Contracts and generated artifacts

- No generated artifacts.
- The authenticated HTTP adapter now exposes generic `POST /credentials/{provider}` and `DELETE /credentials/{provider}` routes while preserving the existing DeepSeek route.

## Configuration and persistence

- Added runtime environment overrides:
  - `SUNCODE_ZHIPU_ENDPOINT`
  - `SUNCODE_ZHIPU_MODEL`
  - `SUNCODE_OPENAI_ENDPOINT`
  - `SUNCODE_OPENAI_MODEL`
- Added non-interactive API key overrides:
  - `ZHIPU_API_KEY`
  - `ZAI_API_KEY`
  - `OPENAI_API_KEY`
- API key environment overrides still require `SUNCODE_NON_INTERACTIVE=true`.
- No SQLite schema changes.

## Tests

- `cargo check -p suncode-runtime` passed.
- `cargo test -p suncode-runtime` passed: 18 tests.
- `cmake --build apps/desktop-qt/build -j2` passed, with existing macOS link target-version warnings.
- `qmllint apps/desktop-qt/qml/GlobalSettings.qml apps/desktop-qt/qml/ConversationPanel.qml` completed with warnings for source-tree import resolution and existing unqualified-access style.
- `node .agents/skills/impeccable/scripts/detect.mjs --json --scope layout apps/desktop-qt/qml/GlobalSettings.qml apps/desktop-qt/qml/ConversationPanel.qml` returned `[]`.
- Offscreen Qt startup reached a running app without QML load errors; the process was terminated after the bounded startup window.
- `git diff --check` passed.

## Documentation

- Added this requirement package.
- Updated product, architecture, and decision records to reflect the expanded built-in provider set.
