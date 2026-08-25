# Requirement

## Background

Many provider models expose a reasoning-effort control. SunCode currently selects only a model, so users cannot tune supported models and cannot see when a model does not accept this parameter.

## Goals

- Advertise reasoning-effort support at model capability level.
- Place a `low` / `medium` / `high` selector immediately to the right of the model selector.
- Disable and clear the selector for unsupported models.
- Pass the selected value through the SDK and OpenAI-compatible provider request.

## Non-goals

- Provider-specific custom effort vocabularies.
- Changing model defaults or persisting a global effort preference.
- Sending the field to models that do not advertise support.

## Requirements

- Extend the model capability DTO and `llm_model` catalog with `supports_reasoning_effort`.
- Extend `submit_turn` with an optional effort value.
- Validate accepted values and model capability in Rust.
- Preserve the effort value in approval/question continuation snapshots.
- Add the control beside the Avalonia model dropdown.

## Acceptance criteria

- Supported seeded models enable the selector and submit the selected value.
- Unsupported seeded models disable the selector and submit no effort field.
- Provider JSON includes `reasoning_effort` only when a supported effort was selected.
- ABI and contract documentation describe the new parameter.
