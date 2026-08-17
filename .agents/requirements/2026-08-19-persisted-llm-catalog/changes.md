# Changes

## Source

- Added `llm_model_provider` and `llm_model` schema/data files.
- Removed `secret_records` and the static runtime catalog as current sources of truth.
- Added Store catalog reads, provider/model upserts, and provider API-key updates.
- Loaded persisted rows into Core's `suncode-llm` registry and applied persisted compaction thresholds.

## Configuration and persistence

- Provider API keys remain plaintext in `llm_model_provider.api_key`. Provider list records expose only `api_key_configured`; plaintext is available solely through the provider-key resolver path.

## Tests

- Added seeded-count/idempotency, provider key, schema/index, and context-threshold tests.
- Existing database, LLM, runtime, and tool tests pass.

## Documentation

- Updated current SQLite, persistence, architecture, feature, specification, and decision records.
