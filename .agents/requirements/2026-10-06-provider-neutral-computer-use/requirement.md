# Provider-Neutral Computer Use

## Background

Computer Use execution is already Rust-owned, but its advertised contract is currently Anthropic's native `computer_toolset_20260801` client toolset. OpenAI-compatible providers reject that client toolset, and the model catalog therefore limits the feature to the seeded Claude routes.

## Goals

- Expose Computer Use as a SunCode-owned function-tool capability independent of a provider's native Computer Use protocol.
- Preserve the existing audited desktop backend, approval gates, emergency stop, control ownership, frame lifetime, and transient screenshot policy.
- Carry screenshot results through providers that support ordinary function calls and multimodal context.
- Make model capability reporting describe the generic contract, while retaining provider-native optimization as optional.

## Non-goals

- Enabling Computer Use for models that do not support function calling and vision.
- Adding a new operating-system sandbox, multi-display control, or non-interactive Computer Use.
- Enabling the currently non-interactive CLI host without an approval and screenshot presentation flow.
- Returning durable screenshot bytes in SQLite, logs, traces, or SDK snapshots.

## Requirements

- Register stable `computer_*` ordinary function tools from the Rust core when the host backend, global setting, control owner, and selected model contract allow them.
- Route generic Computer Use calls through the existing `ComputerAction` executor and policy/approval path.
- Serialize multimodal tool results for Anthropic and OpenAI Responses without dropping screenshot content.
- Add explicit provider/model capability metadata for generic Computer Use and optional native toolset support.
- Update system guidance, Settings status, design-system specimen, contracts, feature documentation, and focused tests.

## Edge cases

- A model with tool calling but no vision must not receive the full Computer Use catalog.
- A provider route that cannot encode image-bearing function results must fail closed for generic Computer Use.
- User control, emergency stop, disabled host ceilings, non-interactive execution, stale coordinate frames, and missing permissions retain current behavior.
- Existing persisted model rows must migrate without losing provider credentials or model ordering.

## Acceptance criteria

- A supported non-Claude model using the OpenAI-compatible adapter receives ordinary `computer_*` tools and can complete a mocked screenshot/action turn.
- Anthropic can use the same generic contract; native Computer Use remains an optional optimization and is not required for advertisement.
- Screenshot images survive the provider request conversion and remain absent from durable persistence.
- Desktop Settings clearly distinguishes generic model support, backend readiness, permissions, and control ownership.
- Focused Rust, C# and design-system checks pass, with no unrelated worktree changes reverted.

## Open questions

- Whether to expose a separate user-invoked screenshot SDK method is deferred; this delivery keeps screenshots model-facing and transient.
