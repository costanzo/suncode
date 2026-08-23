# Changes

## Source

- Added the model-facing `webfetch` schema and core mapping.
- Added an approval-gated `NetworkAccess` policy risk and pre-approval validation.
- Added a bounded Rust HTTP/HTML operation with artifact retention.
- Added WebFetch approval and tool timeline summaries.

## Contracts and generated artifacts

- Provider requests now advertise seven built-in tools.
- No C ABI, SQLite, or SDK DTO changed.

## Configuration and persistence

- No new configuration or database state.
- Existing session Full Control applies to WebFetch.

## Tests

- Added registry, policy, argument, local HTTP, conversion, charset, MIME rejection, artifact, and desktop presentation coverage.

## Documentation

- Updated runtime and operations feature/specification facts and recorded the authority decision.
