# Requirement

## Background

The Rust OpenAI-compatible adapter used Chat Completions request and streaming shapes. OpenAI's Responses API is the current agent-oriented interface and uses typed input/output items and semantic SSE events.

## Goals

- Route every provider using the OpenAI-compatible adapter through `/responses`.
- Preserve SunCode's provider-neutral completion, tool, usage, cancellation, and SDK contracts.
- Keep SunCode's local transcript and compaction as the source of conversation state.

## Non-goals

- Enabling OpenAI-hosted conversation storage or `previous_response_id`.
- Adding OpenAI built-in tools or changing SunCode policy and approval ownership.
- Changing the Anthropic native adapter.

## Requirements

- Encode canonical messages as Responses input items.
- Encode function tools in Responses format and retain non-strict compatibility.
- Parse text deltas, function-call argument events, terminal response events, usage, IDs, and errors.
- Preserve image input and tool result correlation by `call_id`.

## Acceptance criteria

- OpenAI-compatible requests use `/responses` and `input`.
- Existing agent, SDK, and CLI behavior remains provider-neutral.
- Focused LLM, core agent, and CLI integration tests pass.
