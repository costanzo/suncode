# Architecture

## Current state

The registry owns one adapter per provider, while each adapter stores one configured wire model. The catalog, client model selector, and durable session model ID therefore expose only one model per provider. Provider model and endpoint environment variables can silently alter the wire model behind a different stable identity.

## Proposed design

Keep one trusted adapter per provider and make the catalog the source of truth for model identity and wire model. Each `ModelDescriptor` carries the stable client ID, provider, wire model, capabilities, limits, and static API base. The registry returns a model route containing the provider adapter and that model's wire ID. `LlmProvider::complete` receives the wire model for each request.

The built-in catalog contains two models per provider:

| Provider | Models |
| --- | --- |
| DeepSeek | `deepseek-v4-flash`, `deepseek-v4-pro` |
| Zhipu GLM | `glm-5.2`, `glm-5.3` |
| OpenAI | `gpt-5.5`, `gpt-5.6-sol` |
| Kimi | `kimi-k2.7-code`, `kimi-k3` |
| Claude | `claude-sonnet-5`, `claude-opus-5` |
| Gemini | `gemini-3.5`, `gemini-3.6-flash` |

Provider endpoint and model environment overrides are removed. Static endpoints remain in the trusted catalog/registry. Runtime location and storage environment variables are unchanged.

## Boundaries and dependencies

- Catalog owns static model metadata and vendor wire IDs.
- Registry owns provider adapter selection and model routes.
- Provider adapters own HTTP and streaming only; they do not choose models.
- Agent owns the selected stable model ID in the turn continuation and passes the routed wire ID to the adapter.
- Qt consumes `/models`; it does not know provider endpoints or wire IDs.

## Data and control flow

1. Runtime constructs the static six-provider catalog and one adapter per provider.
2. `/models` returns all catalog entries and marks each from its provider credential state.
3. Session/turn admission validates the selected stable model ID and resolves a model route.
4. The agent passes the route's wire model to every streaming completion, including approval continuation.
5. Session persistence stores the stable model ID, never the vendor endpoint or secret.

## Security and failure handling

- Removing endpoint/model environment overrides eliminates an untrusted process-level way to redirect a provider request or create identity drift.
- Provider credentials remain provider-scoped SQLite secrets; environment credential aliases remain restricted to explicit non-interactive execution as a compatibility path.
- Unknown stable model IDs fail closed with `model_unavailable`.

## Compatibility and migration

- Existing stable model IDs continue to resolve.
- Existing sessions storing one of the original model IDs remain valid.
- Existing `SUNCODE_<PROVIDER>_MODEL` and `SUNCODE_<PROVIDER>_ENDPOINT` variables are ignored because those fields are removed from runtime configuration.

## Risks and rollback

- Some listed vendor models may not be enabled for every account. Availability is intentionally credential-level in this delivery; vendor rejection is surfaced through the existing provider error contract.
- The catalog can remove a model entry without changing the provider adapter or database schema.

## Open questions

- Whether model availability should later include vendor capability probing or account-specific allowlists.
