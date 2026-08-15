# Architecture

## Current state

The Rust runtime owns three built-in providers, provider-keyed SQLite credentials, model availability, canonical chat-completions normalization, and the Qt-facing SDK/API surface. Qt renders runtime DTOs and submits generic credential operations.

## Proposed design

Add Kimi, Claude, and Gemini to the trusted built-in registry. Each provider has one stable catalog entry and one configurable endpoint/model pair:

| Provider | Stable model | Default API base | Environment credentials |
| --- | --- | --- | --- |
| Kimi | `kimi-k2.7-code` | `https://api.moonshot.ai/v1` | `MOONSHOT_API_KEY`, `KIMI_API_KEY` |
| Claude | `claude-opus-5` | `https://api.anthropic.com/v1` | `ANTHROPIC_API_KEY`, `CLAUDE_API_KEY` |
| Gemini | `gemini-3.6-flash` | `https://generativelanguage.googleapis.com/v1beta/openai` | `GEMINI_API_KEY`, `GOOGLE_API_KEY` |

The defaults and compatibility surfaces were checked against vendor documentation on 2026-08-15:

- Kimi API quickstart: `https://platform.moonshot.ai/docs/guide/kimi-k2-7-code-quickstart`.
- Claude OpenAI SDK compatibility: `https://platform.claude.com/docs/en/api/openai-sdk`.
- Gemini OpenAI compatibility: `https://ai.google.dev/gemini-api/docs/openai`.

## Boundaries and dependencies

- Rust continues to own provider registration, credentials, requests, streaming normalization, and availability.
- Kimi, Claude, and Gemini use the existing OpenAI-compatible adapter; vendor wire data does not enter Qt or client contracts.
- Qt adds only provider navigation and generic credential pages.
- Provider and agent code do not gain direct machine-operation authority.

## Data and control flow

1. Runtime configuration resolves default or explicitly overridden provider endpoints and wire model IDs.
2. Credential state comes from provider-keyed SQLite records or explicit non-interactive environment aliases.
3. `/credentials` exposes configured booleans only.
4. `/models` joins each catalog entry to its provider credential state.
5. The registry selects the matching adapter for a session model and streams canonical deltas and tool calls.

## Security and failure handling

- API keys remain plaintext SQLite secrets and never appear in responses, events, audit rows, or logs.
- Environment overrides are rejected in interactive mode before runtime startup.
- Authentication, transient, invalid-request, protocol, cancellation, and missing-credential errors keep the existing bounded provider contract.
- Provider compatibility does not expand machine authority; tool calls still pass through policy and the audited operation dispatcher.

## Compatibility and migration

- No schema migration is required because secret records are provider-keyed.
- Existing model IDs and credential records remain unchanged.
- Existing generic credential routes accept three additional provider IDs.
- This delivery's provider endpoint/model environment overrides are superseded by `.agents/requirements/2026-08-15-multi-model-provider-catalog/`; current runtime behavior uses static trusted endpoints and a catalog-owned wire model per request.

## Risks and rollback

- Vendor compatibility behavior can diverge. Focused adapter tests cover the shared request/stream contract without live credentials.
- A provider can be rolled back independently by removing its catalog and registry entry while leaving unrelated credential records inert.
- Live provider calls are not part of repository verification and remain a residual integration risk.

## Open questions

- Whether Claude should later use its native Messages API for provider-specific features.
- Whether Kimi K3 should replace the coding-specific K2.7 default after a separate behavior evaluation.
