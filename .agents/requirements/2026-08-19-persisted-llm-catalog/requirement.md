# Requirement

## Background

The standalone `suncode-llm` layer currently owns a static model catalog, while provider credentials and model availability are managed separately by runtime core. Enterprise deployments need provider endpoints, API keys, models, request model codes, context limits, and compaction settings to be durable and queryable.

## Goals

- Add `llm_model_provider` for built-in and custom providers, endpoint, API key, and lifecycle metadata.
- Add `llm_model` for model IDs, request/wire codes, provider ownership, context and auto-compaction settings, capabilities, and ordering.
- Seed all current built-in providers and models as idempotent reference data.
- Make core load the persisted catalog and construct `suncode-llm` routes from it.
- Support dynamic provider IDs and custom providers without requiring a core enum.
- Keep `suncode-llm` independent of SQLite; database access remains in `suncode-db` and catalog assembly remains in core.

## Non-goals

- Database access from `suncode-llm`.
- Schema migrations or compatibility conversion for old databases.
- C ABI operations for creating custom providers/models in this slice.
- Provider-native protocols beyond the existing OpenAI-compatible adapter.

## Requirements

- `llm_model_provider.provider_id` is unique and stable; endpoint and API key are provider-scoped.
- API keys are stored as plaintext in the provider row under the current Phase 1 storage decision and never serialized in model DTOs or diagnostics.
- `llm_model.provider_id` references its provider and `(provider_id, model_id)` is unique.
- Model rows include request model code, context length, auto-compact threshold, output limit, capabilities, enabled state, and display ordering.
- Built-in seed data must not overwrite user-edited endpoint, key, or model settings on reopen.
- Provider/model reads expose redacted or non-secret values to callers.

## Acceptance criteria

- Current schema has the two new tables and no `secret_records` table.
- Fresh databases contain six built-in providers and twelve built-in models.
- Store CRUD/query methods can list providers/models and update provider API keys.
- Core builds a registry from database rows; custom provider rows route through the OpenAI-compatible adapter.
- Context compaction uses the persisted model context and auto-compact settings.
- Existing behavior and focused database/LLM/runtime tests pass.

## Open questions

- Client-facing CRUD for custom catalog rows remains a later SDK contract change.
