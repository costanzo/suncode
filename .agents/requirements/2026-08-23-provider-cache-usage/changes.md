# Changes

## Source

- Extended provider-neutral usage with nullable cache-miss and reasoning-token counts.
- Normalized nested and top-level OpenAI-compatible cache and reasoning aliases.
- Emitted all normalized optional usage in provider completion events.
- Made the leading host-environment message stable by using the session creation timestamp.

## Contracts and generated artifacts

- Extended the runtime SDK usage contract and shared vectors with additive normalized fields.
- Updated provider-normalization vectors for the expanded provider-neutral usage shape.

## Configuration and persistence

- Reused the existing `session_call.usage_json` column; no schema migration was required.
- Existing rows remain unchanged and readable.

## Tests

- Added unit coverage for the observed DeepSeek and Kimi response shapes and top-level fallback.
- Extended the agent round-trip test to verify stable prefixes and persisted usage diagnostics.
- Extended database projection coverage for the additive usage fields.

## Documentation

- Updated runtime feature, specification, SDK, persistence, and SQLite contract records.
