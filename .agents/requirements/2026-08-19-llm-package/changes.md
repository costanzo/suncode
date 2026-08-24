# Changes

## Source

- Added the `suncode-llm` Cargo package with provider-neutral messages, tools, completion results, usage, errors, model metadata, provider traits, routing, and registration.
- Moved the built-in catalog and OpenAI-compatible HTTP/SSE adapter out of runtime core.
- Added owned custom provider/model registration and an injectable `ApiKeyResolver`.
- Adapted core credentials, tools, persistent messages, tool calls, and usage at the package boundary.
- Added `AgentSdk::open_default_with_providers` so Rust hosts can extend the built-in registry before agent startup.
- Removed the old core `llm` and `model_provider` modules and core's direct `reqwest` dependency.

## Contracts and generated artifacts

- No SDK ABI or generated artifact changes are planned.

## Configuration and persistence

- No schema or persisted configuration changes are planned.

## Tests

- Added LLM tests for all built-in routes, custom trait registration, duplicate-registration atomicity, and a custom OpenAI-compatible endpoint.
- Existing agent and SDK regression tests continue to cover streaming, tool calls, usage, cancellation, and model availability.

## Documentation

- Updated architecture, runtime feature/specification, and the decision index.
