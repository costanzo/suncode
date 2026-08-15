# Architecture

## Current state

DeepSeek is the implemented provider in the Phase 1 runtime. Provider credentials are local secrets and the Qt desktop client consumes model and credential state from the runtime.

## Proposed design

Add Zhipu GLM and OpenAI as trusted built-in providers inside the Rust runtime. Both providers use an OpenAI-compatible chat-completions adapter so provider-specific HTTP details remain behind the runtime provider boundary.

The runtime advertises a small catalog:

- `deepseek-v4-flash` for DeepSeek.
- `glm-5.2` for Zhipu GLM.
- `gpt-5.6-sol` for OpenAI.

The defaults were checked against vendor documentation on 2026-08-11:

- OpenAI official models documentation: `https://platform.openai.com/docs/models`.
- Zhipu OpenAI-compatible documentation: `https://docs.bigmodel.cn/cn/guide/develop/openai/introduction`.

## Boundaries and dependencies

- Rust runtime owns provider registration, provider selection, credentials, request normalization, SSE parsing, and model availability.
- Qt owns presentation and sends generic provider credential requests through the runtime API.
- The Qt client does not call model providers or open SQLite directly.
- The OpenAI-compatible adapter uses the existing canonical role/message/tool shape; multimodal content and provider-native Responses semantics are deferred.

## Data and control flow

1. The runtime loads DeepSeek, Zhipu GLM, and OpenAI credential state from plaintext SQLite secret records, or from explicit non-interactive environment overrides.
2. `/credentials` returns one redacted configured/unconfigured state per provider.
3. `/models` returns all registered models and marks availability according to each model provider's credential state.
4. New sessions and turns validate the selected model through the provider registry.
5. The selected provider adapter sends a streamed `/chat/completions` request and normalizes text, tool calls, finish reason, and usage into SunCode runtime events.

## Security and failure handling

- API keys are persisted as plaintext SQLite secret records, and never in events, audit records, logs, or protocol responses.
- The SQLite data directory and its backups must be treated as sensitive because they contain provider API keys.
- Interactive environment-variable API key overrides remain rejected unless `SUNCODE_NON_INTERACTIVE=true`.
- Missing credentials fail as `provider_unconfigured`.
- Provider authentication errors are redacted to provider-level error codes and messages.

## Compatibility and migration

- Existing DeepSeek credential routes remain compatible.
- New generic credential routes are provider-keyed: `/credentials/{provider}`.
- Existing sessions that use `deepseek-v4-flash` continue to resolve to the DeepSeek adapter.

## Risks and rollback

- OpenAI-compatible provider behavior can differ across vendors. The adapter is intentionally narrow and can be disabled by removing the Zhipu/OpenAI catalog entries and registry mappings.
- OpenAI's newer Responses API is not adopted in this change; that migration should be a separate provider-specific requirement.

## Open questions

- Whether the product should later expose multiple models per provider instead of one default catalog entry per provider.
