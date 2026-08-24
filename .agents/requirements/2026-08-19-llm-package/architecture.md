# Architecture

## Current state

The runtime core owns the provider trait, built-in catalog, registry, HTTP clients, SSE parser, canonical provider types, database-backed credential store, persistent messages, and tool declarations. Provider implementations directly depend on core and its credential store.

## Proposed design

`suncode-llm` owns provider-neutral LLM types and provider execution:

```text
suncode-agent/core
    |- loads credentials from suncode-db
    |- implements ApiKeyResolver
    |- converts persistent messages at the agent boundary
    `- builds the built-in or host-extended registry
              |
              v
suncode-llm
    |- LlmProvider and ApiKeyResolver traits
    |- CompletionRequest, Message, ToolDefinition, Completion, Usage
    |- ModelProviderRegistry and custom registration
    |- built-in model catalog
    `- OpenAI-compatible HTTP/SSE adapter
```

The registry maps owned stable model IDs to an `Arc<dyn LlmProvider>` and a wire model. Registration is public and validates the whole registration before mutation. The built-in registry is a convenience constructor over the same API used by enterprise callers.

## Boundaries and dependencies

- `suncode-llm` depends only on general Rust libraries for HTTP, serialization, async execution, and cancellation.
- It does not depend on core, database, tool execution, SDK, or desktop code.
- Core depends on `suncode-llm` and `suncode-db` and adapts between their independent DTOs.
- `ApiKeyResolver` is implemented by core's credential store. The LLM package knows only provider IDs and returned secret strings.
- Tool schemas are request data. The LLM package never imports or executes SunCode tools.

## Data and control flow

1. Core loads credentials and constructs a built-in LLM registry.
2. The agent resolves a model route and converts retained persistent messages into LLM messages.
3. The agent passes tool schemas, cancellation, and a delta channel in a completion request.
4. The provider resolves its API key through `ApiKeyResolver`, performs HTTPS, and returns normalized completion data.
5. Core converts tool calls and usage back into persistent runtime DTOs.

Enterprise embedding code may construct an `OpenAiCompatibleProvider` with a private endpoint or implement `LlmProvider`, then register it with owned provider and model metadata. This is trusted in-process Rust composition, not an executable plugin boundary.

## Security and failure handling

Secrets remain owned by core and are never serialized into registry metadata, completion results, errors, or logs. Custom providers execute in-process with the host's authority and are not described as isolated. Provider failures remain normalized and bounded at the agent boundary.

## Compatibility and migration

This is a source-level package extraction. Persistent schema, SDK ABI, model IDs, provider IDs, endpoints, and user data do not change.

## Risks and rollback

The main risk is semantic drift during DTO conversion. Existing provider stream tests and agent round-trip tests cover the boundary. The package can be reverted without a data migration because no schema changes are involved.

## Open questions

Persisted enterprise configuration and a client-facing registration contract are intentionally deferred.
