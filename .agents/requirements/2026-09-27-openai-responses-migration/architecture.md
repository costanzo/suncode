# Architecture

## Current state

`suncode-llm` exposes canonical `Message`, `ToolCall`, `Completion`, and `Usage` types. The OpenAI-compatible adapter owns HTTP and SSE conversion.

## Proposed design

The adapter sends `input` items, function tools with `strict: false`, `reasoning.effort`, `max_output_tokens`, `stream: true`, and `store: false`. `ResponsesSseParser` converts semantic events into canonical deltas and completion values.

## Boundaries and dependencies

Only `suncode-llm` wire conversion changes. Agent core, persistence, SDK bindings, desktop, and CLI consume unchanged canonical values.

## Compatibility and migration

All providers selecting `adapter_type=openai` use the Responses shape. Anthropic remains on its own Messages adapter.

## Security and failure handling

Responses storage is disabled so provider-side response retention is not introduced. Existing API key, TLS, proxy, cancellation, and redacted diagnostic paths remain in Rust.
