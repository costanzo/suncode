# Architecture

## Current state

The shared SSE parser recognizes nested `cached_tokens` and a small set of cache-read aliases. Core emits five normalized usage fields, and the database stores that JSON without retaining the provider-private response. The leading system message embeds the current second on every call.

## Proposed design

Extend provider-neutral `Usage` with nullable cache-miss and reasoning counts. Normalize compatible wire aliases in `suncode-llm`, emit all normalized fields from core, and retain them in the existing JSON usage column. Capture the session creation timestamp when constructing a continuation and use it in the host-environment message.

## Boundaries and dependencies

Wire-format compatibility remains in `suncode-llm`. Core maps provider-neutral usage to runtime events. `suncode-db` remains a provider-agnostic JSON persistence owner. Avalonia continues consuming the normalized SDK DTO.

## Data and control flow

The final SSE usage object is parsed into provider-neutral usage, copied into `provider.exchange.completed`, and serialized unchanged into `session_call.usage_json`. Session creation time flows through the turn continuation to every model call in that session.

## Security and failure handling

No raw response, header, credential, or provider-private payload is persisted. Optional fields distinguish unavailable values from explicit zero.

## Compatibility and migration

The JSON fields are additive and require no table migration. Existing rows remain readable. Legacy suspended continuations fall back to a stable unspecified session-start value.

## Risks and rollback

Alias precedence could select the wrong value when a provider returns inconsistent duplicates; standard nested fields take precedence. Removing the new optional fields and restoring call-time context rolls back the change without a schema migration.

## Open questions

- None.
