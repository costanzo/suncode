# Requirement

## Background

The Phase 1 vertical slice works, but several Rust and Qt files combine unrelated concerns. That makes adding another model provider, an LLM wire adapter, or a tool difficult and increases regression risk.

## Goals

- Establish a canonical LLM boundary independent of vendor wire formats.
- Add a model-provider registry so model metadata and provider construction are not hard-coded in API handlers.
- Give each built-in tool a dedicated Rust module and a single registry entry.
- Keep the existing Rust SDK and Qt QML-facing behavior compatible.
- Create focused seams for later extraction of Qt request, projection, and view code.

## Non-goals

- Adding a second production provider in this change.
- Changing the client protocol or persistence schema.
- Introducing plugins, remote tenancy, or a new client surface.

## Requirements

1. Vendor request/stream parsing stays inside a provider adapter module.
2. Agent code consumes canonical LLM result types and a provider trait/factory, not DeepSeek wire types.
3. Tool schemas are defined one tool per file and assembled by a registry.
4. Existing model IDs, SDK functions, approval behavior, and operation methods remain compatible.
5. New Rust modules are included by the Qt CMake runtime source dependency glob.

## Edge cases

- Unknown model/provider identifiers must fail with the existing `model_unavailable` behavior.
- Provider streaming cancellation and malformed tool-call errors must retain their current codes.
- A tool registry entry must not silently expose an operation lacking a policy mapping.

## Acceptance criteria

- `llm`, `model_provider`, and `tools` modules exist in the runtime core.
- DeepSeek implementation is isolated below `model_provider/deepseek`.
- Each built-in model tool has its own schema file.
- Rust tests, clippy, and the Qt CMake build pass.

## Open questions

- The full Qt/QML visual split should follow after the backend seams are exercised, because the current QML file is a stateful shell rather than a collection of independent views.
