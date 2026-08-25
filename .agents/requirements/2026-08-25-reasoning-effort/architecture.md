# Architecture

## Current state

Model capability metadata is stored in `llm_model`, projected through Rust's `ModelDescriptor`, and consumed by Avalonia. `submit_turn` accepts session, input, idempotency key, and model.

## Proposed design

Add a boolean `supports_reasoning_effort` to the model catalog and `capabilities.reasoning_effort` to the model DTO. The C ABI accepts one nullable `reasoning_effort` string. Rust validates the value (`low`, `medium`, `high`) and rejects a non-null value for a model without the capability. The turn continuation stores the selected value so approval/question recovery reuses the original choice.

The OpenAI-compatible adapter adds `reasoning_effort` to the request JSON only when the validated value is present. Avalonia displays the selector directly after model selection and binds its enabled state to the selected model capability and composition state.

## Boundaries and dependencies

The database owns model capability metadata, core owns turn validation and continuation state, the LLM package owns provider wire translation, and Avalonia owns only selection presentation. No client reads SQLite or infers provider capabilities.

## Compatibility and migration

This is a new project contract change. The native ABI increments from 3 to 4. No compatibility adapter or database migration is added.

## Risks and rollback

Provider-specific values beyond the common three levels remain unsupported until a model catalog field can describe them. Reverting the capability, request, and UI changes restores the prior model-only contract.
