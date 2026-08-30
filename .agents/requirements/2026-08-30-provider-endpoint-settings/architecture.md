# Architecture

## Current state

Rust SQLite rows own provider endpoints and the provider registry is built when the embedded SDK opens. Avalonia receives endpoint values through `list_models`, but the SDK exposes no endpoint mutation method.

## Proposed design

Add a named `set_provider_endpoint(provider, endpoint)` SDK operation. Rust validates and normalizes the URL, updates the existing provider row without changing its other fields, and atomically replaces that provider's live registry route and model descriptors. The C ABI returns a redacted provider update DTO containing only provider ID and endpoint. Avalonia invokes this method, reloads models/providers, and reports the result in Settings.

The navigation parent remains a page destination and gains a separate chevron toggle so opening the overview and expanding children are independent actions.

## Boundaries and dependencies

- Rust remains the only owner of provider persistence, endpoint validation, and live routing.
- Avalonia owns input, navigation state, and save feedback.
- The React/Vite specimen remains review tooling only.

## Data and control flow

1. User edits a provider URL and selects Save URL.
2. Avalonia calls the hand-written C# SDK wrapper and C ABI.
3. Rust validates/normalizes the endpoint and builds the replacement OpenAI-compatible route.
4. Rust updates `llm_model_provider.endpoint`, replaces the live route, and returns the normalized endpoint.
5. Avalonia reloads the model/provider projection and updates status.

## Security and failure handling

Only HTTP and HTTPS URLs with a host are accepted. Embedded username/password values are rejected. API keys never enter the request or response DTO. Invalid input and persistence errors leave the existing route active.

## Compatibility and migration

The SQLite schema is unchanged. The C ABI change is add-only within ABI version 4. Existing callers remain compatible.

## Risks and rollback

A persisted update followed by an unexpected in-memory registry replacement failure could temporarily diverge until restart; replacement is validated before persistence and uses the existing provider/model identities to make that failure unreachable under a current valid catalog. Rolling back the code leaves persisted endpoints readable at the next startup.

## Open questions

- None.
