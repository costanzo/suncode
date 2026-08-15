# Requirement

## Background

The runtime currently advertises one model per provider. Provider model environment overrides can change the wire model sent to a vendor without changing the stable model ID shown to clients, which makes model selection and durable session history ambiguous.

## Goals

- Advertise multiple stable models for every built-in provider.
- Route each selected stable model ID to the correct provider adapter and wire model.
- Remove provider endpoint/model environment overrides from production configuration.
- Keep credentials provider-scoped and model availability derived from provider credential state.

## Non-goals

- Live vendor model discovery, per-user custom model registration, or provider-native model management APIs.
- Removing environment variables used for runtime location, storage, non-interactive execution, or legacy credential overrides.
- Provider-native request formats, multimodal content, or model-specific UI controls.

## Requirements

- Register at least two models for each built-in provider.
- Keep existing stable model IDs compatible, including `gpt-5.6-sol` and `deepseek-v4-flash`.
- Pass the selected model's wire ID into every provider completion request, including approval continuations and resumed turns.
- Validate session and turn model IDs against the static catalog.
- Return every catalog model from `/models` with provider-level configured/unconfigured availability.
- Let Qt display and select all registered model IDs without provider model environment overrides.

## Edge cases

- An unknown model ID remains `model_unavailable` and cannot create a session or turn.
- A configured provider makes all of its registered models available; missing credentials make all of its models unconfigured.
- A model's stable ID may differ from its vendor wire ID in the catalog without changing client behavior.
- Existing sessions keep their stored stable model ID after the catalog expands.

## Acceptance criteria

- `/models` returns at least two entries for each of the six providers.
- OpenAI includes both `gpt-5.5` and `gpt-5.6-sol` as independently selectable models.
- Provider adapters send the selected wire model from the route rather than a provider-global configured model.
- Provider endpoint/model environment variables are no longer read.
- Qt's default model selector contains all catalog entries and saves the selected stable ID.
- Focused Rust, Qt, QML, UI detector, startup, and diff checks pass.

## Open questions

- Whether future releases should allow users to enable/disable individual models without changing the built-in catalog.
