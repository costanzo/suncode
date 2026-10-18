# Providers and Models

**Status:** Implemented and focused-tested

The Rust agent owns the model catalog, provider routes, and wire adapters. Clients read providers and models through `list_models` and never contact a provider or open SQLite directly.

## Catalog and custom models

- Providers and models are rows in `llm_model_provider` and `llm_model`. Each provider row names a required `adapter_type`: `openai` for OpenAI-compatible endpoints or `anthropic` for the built-in Claude Messages route.
- Model rows carry request identifiers, context and auto-compaction limits, an optional output limit, capability flags (streaming, tool use, vision, structured output, cancellation, reasoning effort, Computer Use), and the advertised `reasoning_efforts` list.
- Custom providers come from two sources: persisted OpenAI-compatible rows, or trusted `Arc<dyn LlmProvider>` implementations registered in-process by a Rust host. The registry registers a provider and its models atomically and rejects duplicate model identifiers. Dynamic plugins and third-party adapter loading are not supported. There is no SDK method to add, remove, enable, or reorder providers or models.

## Gemini Flash catalog

The latest stable Flash entry is `gemini-3.8-flash`, verified against Google's [model page](https://ai.google.dev/gemini-api/docs/models/gemini-3.8-flash) on 2026-10-12. It accepts up to 1,048,576 input tokens and 65,536 output tokens, supports vision, function calling, structured output, and reasoning levels `low`, `medium`, and `high`. `minimal` is unsupported. The previous-generation `gemini-3.6-flash` remains selectable.

The former `gemini-3.5` row used a legacy SunCode ID for wire model `gemini-3.5-flash`; the label referred to that identifier mismatch. Google's [deprecation page](https://ai.google.dev/gemini-api/docs/deprecations) currently routes `gemini-3.5-flash` to `gemini-3.6-flash` without an announced shutdown date. Startup replaces the legacy catalog row with the canonical `gemini-3.8-flash` row and updates saved session selections and scoped `default_model` settings. Migrated sessions use `low` in place of unsupported `minimal`; historical turn/call records retain their original model identifiers.

## xAI Grok catalog

The xAI provider uses `https://api.x.ai/v1` through the existing Responses adapter. Its credential and endpoint follow the same provider Settings path as other providers; credentials use the Rust-owned system store by default or explicit SQLite plaintext mode. Startup seeding adds it to existing databases without overwriting provider credential metadata or endpoint settings.

The catalog was checked against the [official xAI model list](https://docs.x.ai/developers/models) and individual model pages on 2026-10-12:

| Model / request ID | Context tokens | Selectable reasoning efforts |
| --- | --- | --- |
| `grok-4.7` | 500,000 | low, medium, high, xhigh |
| `grok-4.6` | 500,000 | low, medium, high, xhigh |
| `grok-4.5` | 500,000 | low, medium, high |
| `grok-4.3` | 1,000,000 | none, low, medium, high, xhigh |
| `grok-build-0.1` | 256,000 | none advertised |

All five accept text/image input, stream text, and support function calling and structured output. Auto-compaction reserves the existing 16,384-token margin. Output limits remain unset: the [Grok 4.7 guide](https://docs.x.ai/developers/grok-4-7) explicitly specifies no text output limit, and the other model pages do not publish separate output caps. Computer Use remains unadvertised pending verification of screenshot-bearing function results. Grok Build reasons internally but does not advertise configurable effort. The Grok 4.5 model page lists `xhigh`, but the [reasoning guide](https://docs.x.ai/developers/model-capabilities/text/reasoning) says it is treated as `high`; SunCode exposes the three distinct levels. Grok 4.7 Fast is excluded because it is unavailable on the public API. Image, video, voice, and multi-agent endpoints are outside this coding-model catalog addition.

## MiniMax and Meta catalogs

The [MiniMax model list](https://platform.minimax.io/docs/guides/models-intro) and [Meta model list](https://dev.meta.ai/docs/models) were checked on 2026-10-12. Both providers use the existing Responses adapter: MiniMax at `https://api.minimax.io/v1` and Meta at `https://api.meta.ai/v1`. Startup inserts missing providers and models without overwriting saved credentials or endpoints.

| Provider | Model / request ID | Context tokens | Output limit | Selectable reasoning efforts |
| --- | --- | --- | --- | --- |
| MiniMax | `MiniMax-M3.1-Flash-Preview` | 1,000,000 | not published | low, medium, high, xhigh, max |
| MiniMax | `MiniMax-M3` | 1,000,000 | not published | none advertised |
| Meta | `muse-wonder-1.0` | 1,048,576 | not published | minimal, low, medium, high, xhigh, max |
| Meta | `muse-spark-1.3` | 1,048,576 | 131,072 | minimal, low, medium, high, xhigh, max |

MiniMax M3.1 Flash Preview requires an [M Plan subscription key](https://platform.minimax.io/docs/m-plan/quickstart); a pay-as-you-go key does not grant access. Its display label includes this requirement. M3 remains the pay-as-you-go choice. The [MiniMax Responses reference](https://platform.minimax.io/docs/api-reference/responses-create) supports streaming, images, and function tools, but advertises only text output format, so structured output remains false. M3.1 always reasons and defaults to `max`; `none` is rejected. M3 defaults to reasoning off on Responses and accepts a non-`none` effort to enable it without tuning depth; SunCode does not expose configurable effort for that model. M2.7 and earlier generations are excluded because MiniMax has announced their retirement.

Muse Spark 1.3 is Meta's recommended latest Spark version. Muse Wonder 1.0 is a preview available through the public Model API in Standard mode; Wonder Full is subscription-only in Muse Code and is excluded. Both support images, function tools, structured output, streaming, and the [reasoning levels](https://dev.meta.ai/docs/reasoning) above; `none` is rejected. Spark's output limit comes from the [official quickstart](https://dev.meta.ai/docs/quickstart). Contributor variants and the self-hosted Muse Glimmer are excluded. SunCode replays local messages and tool outputs but does not preserve Meta's encrypted reasoning items across calls; each request reasons from the supplied transcript.

All four use the existing 16,384-token auto-compaction margin and client-side cancellation. Undocumented output limits remain unset. Computer Use remains unadvertised pending verification of screenshot-bearing function results. This catalog update verifies persistence and SDK routing, not live provider calls.

## Xiaomi MiMo catalog

The [official model catalog](https://mimo.mi.com/docs/en-US/quick-start/summary/model) and [Responses reference](https://mimo.mi.com/docs/en-US/api/chat/responses) were checked on 2026-10-12. Xiaomi MiMo uses `https://api.xiaomimimo.com/v1` through the existing Responses adapter with Bearer authentication. Startup adds its provider and model rows without changing existing credentials or endpoints. Token Plan subscribers must set the provider URL and key to the matching regional endpoint shown in their Xiaomi console; pay-as-you-go and subscription credentials use different base URLs.

The seeded models are `mimo-v2.6-pro` and `mimo-v2.6-flash`. Both support image input, streaming, function calls, and JSON object output. The [official OpenCode configuration](https://mimo.mi.com/docs/en-US/tokenplan/integration/opencode) specifies 1,048,576 context tokens and 131,072 output tokens; SunCode reserves the existing 16,384-token compaction margin. Computer Use remains unadvertised pending verification of screenshot-bearing function results. UltraSpeed requires customized service access and is excluded, as are the V2.5 models scheduled for retirement on 2026-10-21 and speech-only endpoints.

Responses reasoning is enabled by default. `none` disables it; every other accepted effort enables the same reasoning behavior. SunCode exposes `none` and `high` as the two distinct states rather than advertising multiple depths. Xiaomi recommends replaying reasoning items during Responses tool loops; SunCode currently replays local messages and tool outputs without reasoning items. Live provider calls remain unverified.

## Endpoint settings

`set_provider_endpoint(provider, endpoint)` changes the base URL of one existing provider. Rust trims whitespace and trailing slashes, accepts only absolute HTTP or HTTPS URLs with a host and no embedded credentials, query, or fragment, and rejects unknown providers. A successful update persists the URL and atomically replaces that provider's live route. Provider identity, adapter, credential, models, enabled state, and ordering stay the same. Requests already in flight finish on the route they started with. A rejected URL leaves the stored endpoint unchanged. The C ABI returns only the provider ID and the normalized endpoint. Desktop Settings edits the endpoint on each provider detail page.

## OpenAI Responses wire format

Every `adapter_type=openai` provider posts to `<endpoint>/responses`. This includes the seeded DeepSeek, Qwen, Zhipu GLM, Kimi, Gemini, OpenAI, xAI, MiniMax, Meta, and Xiaomi MiMo rows. Chat Completions is no longer used. Requests contain:

- canonical messages encoded as Responses `input` items, with images as `input_image` data URLs;
- function tools with `strict: false`;
- `stream: true`, `store: false`, optional `reasoning.effort`, and optional `max_output_tokens`.

SunCode never sends `previous_response_id`. The local transcript and context compaction remain the only source of conversation state, and `store: false` keeps responses from being retained on the provider side. A semantic SSE parser turns text deltas, function-call argument events, terminal response events, usage, request/response IDs, and errors into provider-neutral completions. Tool results are correlated by `call_id`. OpenAI built-in tools are not exposed. The OpenAI-compatible adapter accepts SunCode's ordinary `computer_*` function tools and preserves screenshot images in multimodal function results; provider-native client toolsets remain optional.

The `anthropic` adapter posts to `<endpoint>/messages` and supports the same ordinary `computer_*` function tools. Native client toolsets such as Computer Use remain available as an optional provider-specific optimization.

Both adapters apply the global HTTPS certificate and proxy settings described in [`network/`](../network/README.md). Credential, usage, and trace rules are covered in [`agent-phase-1/`](../agent-phase-1/README.md) and [`contracts/agent-sdk/README.md`](../../../contracts/agent-sdk/README.md).
