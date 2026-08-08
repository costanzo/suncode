# Architecture

## Current state

`provider.rs` combines canonical messages, DeepSeek HTTP/SSE handling, and all tool schemas. `agent.rs` consumes the concrete provider. The operations crate has one large dispatcher containing every operation.

## Proposed design

```text
agent -> llm::Provider / canonical types
                  ^
                  |
        model_provider::Registry
                  |
        model_provider::deepseek (wire JSON + SSE)

agent -> tools::Registry -> operations dispatcher
              one schema module per tool
```

The obsolete monolithic `provider` module is removed. Provider metadata is owned by the registry and can be extended without editing every API route.

## Boundaries and dependencies

`llm` depends on domain message/value types only. A provider adapter may depend on credentials, HTTP, and the LLM boundary. Tool schema modules depend on JSON only. Operations remain the audited execution boundary and do not depend on provider modules.

## Data and control flow

The agent selects a model through the registry, obtains a provider handle, sends canonical messages, receives canonical streamed deltas/results, and dispatches typed tool names through the existing policy and operations path.

## Security and failure handling

Credentials remain owned by `CredentialStore`. Provider errors retain their existing redacted code/retryability contract. The registry rejects unknown models before a network request.

## Compatibility and migration

No protocol or SQLite changes are required. Existing `provider::*` imports remain valid during this migration. Qt source registration continues to use recursive runtime discovery.

## Risks and rollback

The main risk is accidental behavior drift in SSE parsing or tool JSON schemas. Focused parser and agent round-trip tests provide rollback points.

## Open questions

The operations implementation will be split in a follow-up slice once the core registry contract has landed.
