# Architecture

## Current state

The pre-change design used a static catalog and `secret_records`; this requirement replaces both with persisted provider/model rows and dynamic provider IDs.

## Proposed design

```text
suncode-db
    |- llm_model_provider (provider identity, endpoint, plaintext api key)
    |- llm_model (model identity, wire code, limits, compaction, capabilities)
    `- seed data for built-in providers/models
              |
              v
suncode-agent/core
    |- reads catalog rows and updates provider keys
    |- creates ApiKeyResolver over provider rows
    |- builds suncode-llm ModelProviderRegistry
    `- passes persisted context/compaction limits to context builder
              |
              v
suncode-llm
    `- receives provider-neutral descriptors and trusted OpenAI-compatible providers
```

`suncode-llm` remains database-free. Its registry receives provider/model descriptors from core. Built-in and custom rows use the same OpenAI-compatible adapter, with provider endpoint and provider ID supplied from the database. Custom non-compatible implementations remain trusted Rust registration APIs.

## Schema

`llm_model_provider` has a stable text ID, display name, endpoint, optional API key, enabled flag, sort order, and timestamps. `llm_model` has a stable model ID, provider foreign key, display name, request model code, context length, auto-compact threshold, max output tokens, capability flags, enabled flag, sort order, and timestamps. Checks enforce positive limits, `auto_compact_tokens < context_tokens`, and non-empty identifiers.

## Data and control flow

1. Schema initialization creates tables and applies idempotent built-in seed rows.
2. Core loads enabled provider/model rows and maps them to `suncode-llm::ModelDescriptor`.
3. Core creates one `OpenAiCompatibleProvider` per provider using the persisted endpoint and a row-backed key resolver.
4. Registry routes model IDs to provider and wire model code.
5. Agent uses model context tokens and compaction threshold for `context::build_for_model`.
6. Credential SDK methods update `llm_model_provider.api_key` and return redacted provider status.

## Security and failure handling

API keys stay in the database and are only read by the provider resolver. Provider/model DTOs omit keys. Disabled providers/models do not route. Invalid rows fail database initialization or catalog assembly with a bounded runtime error instead of silently falling back.

## Compatibility and migration

The system is new and uses one current schema. The old `secret_records` table is removed from the current schema; no migration path is provided. Existing development databases with the old table set are rejected as incompatible.

## Risks and rollback

The main risk is changing credential ownership and model availability simultaneously. Tests cover seeded counts, CRUD, dynamic routing, redaction, and compaction. Reverting the source change does not require data conversion because incompatible databases are rejected by design.
