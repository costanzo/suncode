# Requirement

## Background

The Rust runtime and Qt desktop client support DeepSeek, Zhipu GLM, and OpenAI as built-in model providers. Users also need first-class access to Kimi, Claude, and Gemini without bypassing runtime-owned credentials, model selection, or canonical agent behavior.

## Goals

- Add Kimi, Claude, and Gemini as trusted built-in providers.
- Preserve the existing local credential, model catalog, and Qt settings flows.
- Reuse the canonical chat-completions normalization path where vendor compatibility permits.

## Non-goals

- Provider-native Responses, Messages, or Gemini `generateContent` APIs.
- OAuth, hosted identity, account discovery, or automatic model enumeration.
- Multiple catalog entries per provider, multimodal input, or provider-specific controls.
- Database schema or client protocol generation changes.

## Requirements

- Register one stable SunCode model identity for each new provider: `kimi-k2.7-code`, `claude-opus-5`, and `gemini-3.6-flash`.
- Load API keys from runtime-owned SQLite secret records.
- Accept documented environment credentials only when `SUNCODE_NON_INTERACTIVE=true`.
- Advertise redacted credential state and model availability through the existing client API and Rust SDK facade.
- Send streamed chat-completions requests through the existing canonical message, tool, usage, cancellation, and error-normalization path.
- Let Qt users save and remove each provider key and inspect the registered model.

## Edge cases

- Empty API keys remain rejected by the Qt save action and never become configured state.
- An unsupported provider ID returns the existing typed `invalid_arguments` error.
- Existing provider credentials, sessions, model identities, and the legacy DeepSeek credential route remain compatible.
- Interactive startup rejects any provider-key environment override, including aliases for the three new providers.

## Acceptance criteria

- `/credentials` returns six redacted provider states.
- `/models` returns six model entries with availability derived from the matching credential.
- The registry resolves all six stable model IDs and rejects unknown IDs.
- Kimi, Claude, and Gemini can execute the existing streamed tool-call path through their documented compatibility endpoints.
- Qt settings exposes a working credential page for every built-in provider.
- Focused Rust tests, the Qt build, QML validation, UI detector, offscreen startup, and diff hygiene checks pass.

## Open questions

- Whether future work should expose multiple models per provider or provider-native protocol features.
