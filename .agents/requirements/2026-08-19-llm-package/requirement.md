# Requirement

## Background

Provider traits, model metadata, HTTP adapters, streaming normalization, database-backed credential access, agent DTOs, and tool declarations currently meet inside the runtime core. This prevents provider code from being reused independently and makes custom enterprise provider injection difficult.

## Goals

- Create a standalone `suncode-llm` Cargo package under `runtime/crates/llm`.
- Move all provider-neutral LLM contracts, built-in model metadata, provider routing, HTTP adapters, and stream parsing into it.
- Keep the package independent of SQLite and all SunCode persistence types.
- Allow trusted Rust callers to register custom provider implementations and models.
- Allow trusted Rust hosts to inject registered providers into the runtime agent during SDK construction.
- Allow OpenAI-compatible enterprise endpoints to reuse the built-in adapter.
- Preserve current built-in provider behavior and the runtime SDK model surface.

## Non-goals

- Persist custom provider definitions or credentials.
- Add desktop UI for custom providers.
- Load arbitrary dynamic libraries, scripts, plugins, or out-of-process adapters.
- Add provider-native protocol implementations beyond the existing OpenAI-compatible behavior.

## Requirements

- The LLM package must not depend on `suncode-db`, `suncode-runtime`, or `suncode-tool`.
- Provider credentials must be obtained through an injected provider-neutral interface.
- Completion requests must carry provider-neutral messages and tool definitions owned by the LLM package.
- The registry must reject duplicate provider and model registrations deterministically.
- Model and provider identifiers must be owned strings so enterprise callers are not limited to static literals.
- Built-in DeepSeek, Zhipu GLM, OpenAI, Kimi, Claude, and Gemini models must remain available.

## Edge cases

- Missing credentials produce `provider_unconfigured` without exposing secret values.
- Unknown model IDs do not route.
- Duplicate provider/model IDs fail registration without partially modifying the registry.
- Malformed provider streams and tool arguments retain normalized errors.

## Acceptance criteria

- Core no longer contains `llm` or `model_provider` implementation modules.
- A test registers and routes a custom provider/model.
- The Rust SDK exposes a construction hook that extends the built-in registry before the agent starts.
- Existing agent/provider/runtime tests pass.
- Dependency inspection confirms the LLM package has no database dependency.

## Open questions

- Persistence and client configuration of enterprise providers remain a future requirement.
