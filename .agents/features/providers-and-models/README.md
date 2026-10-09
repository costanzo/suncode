# Providers and Models

**Status:** Implemented and focused-tested

The Rust agent owns the model catalog, provider routes, and wire adapters. Clients read providers and models through `list_models` and never contact a provider or open SQLite directly.

## Catalog and custom models

- Providers and models are rows in `llm_model_provider` and `llm_model`. Each provider row names a required `adapter_type`: `openai` for OpenAI-compatible endpoints or `anthropic` for the built-in Claude Messages route.
- Model rows carry request identifiers, context and auto-compaction limits, an optional output limit, capability flags (streaming, tool use, vision, structured output, cancellation, reasoning effort, Computer Use), and the advertised `reasoning_efforts` list.
- Custom providers come from two sources: persisted OpenAI-compatible rows, or trusted `Arc<dyn LlmProvider>` implementations registered in-process by a Rust host. The registry registers a provider and its models atomically and rejects duplicate model identifiers. Dynamic plugins and third-party adapter loading are not supported. There is no SDK method to add, remove, enable, or reorder providers or models.

## Endpoint settings

`set_provider_endpoint(provider, endpoint)` changes the base URL of one existing provider. Rust trims whitespace and trailing slashes, accepts only absolute HTTP or HTTPS URLs with a host and no embedded credentials, query, or fragment, and rejects unknown providers. A successful update persists the URL and atomically replaces that provider's live route. Provider identity, adapter, credential, models, enabled state, and ordering stay the same. Requests already in flight finish on the route they started with. A rejected URL leaves the stored endpoint unchanged. The C ABI returns only the provider ID and the normalized endpoint. Desktop Settings edits the endpoint on each provider detail page.

## OpenAI Responses wire format

Every `adapter_type=openai` provider posts to `<endpoint>/responses`. This includes the seeded DeepSeek, Qwen, Zhipu GLM, Kimi, Gemini, and OpenAI rows. Chat Completions is no longer used. Requests contain:

- canonical messages encoded as Responses `input` items, with images as `input_image` data URLs;
- function tools with `strict: false`;
- `stream: true`, `store: false`, optional `reasoning.effort`, and optional `max_output_tokens`.

SunCode never sends `previous_response_id`. The local transcript and context compaction remain the only source of conversation state, and `store: false` keeps responses from being retained on the provider side. A semantic SSE parser turns text deltas, function-call argument events, terminal response events, usage, request/response IDs, and errors into provider-neutral completions. Tool results are correlated by `call_id`. OpenAI built-in tools are not exposed. The OpenAI-compatible adapter accepts SunCode's ordinary `computer_*` function tools and preserves screenshot images in multimodal function results; provider-native client toolsets remain optional.

The `anthropic` adapter posts to `<endpoint>/messages` and supports the same ordinary `computer_*` function tools. Native client toolsets such as Computer Use remain available as an optional provider-specific optimization.

Both adapters apply the global HTTPS certificate and proxy settings described in [`network/`](../network/README.md). Credential, usage, and trace rules are covered in [`agent-phase-1/`](../agent-phase-1/README.md) and [`contracts/agent-sdk/README.md`](../../../contracts/agent-sdk/README.md).
