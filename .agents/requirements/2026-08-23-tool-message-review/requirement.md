# Requirement

## Background

Tool messages in the conversation displayed full JSON results inline, making the timeline difficult to scan. A recent failed turn also showed that a malformed shell call with `{"command":""}` was recorded as still executing even though parameter translation failed.

## Goals

- Show a concise human-readable operation summary in the conversation.
- Open request, result, and error details in a focused dialog when the user selects a tool message.
- Preserve enough request/result data for the detail view after snapshot reloads and live updates.
- Record pre-execution argument failures as failed tool states.
- Keep shell compatibility for non-empty legacy `command` input while rejecting empty scripts.

## Non-goals

- Changing the runtime tool protocol or model-facing shell schema.
- Hiding approval details or removing raw operation data from review surfaces.
- Making shell syntax portable across operating systems.

## Requirements

- Tool cards show an operation label and state only.
- Tool detail dialogs expose request, result, and error sections when present.
- Empty or missing shell scripts return `invalid_arguments`.
- A failed argument translation emits `tool.state=failed` before the turn fails.

## Edge cases

- Older shell calls using a non-empty `command` field remain executable through the compatibility translator.
- Malformed or absent tool result payloads leave the detail dialog usable.
- Failed tools remain visible in the timeline with a failed state instead of remaining in `executing`.

## Acceptance criteria

- Conversation cards no longer render full JSON result previews.
- Selecting a card opens a detail dialog with copyable request/result text.
- The referenced turn's malformed shell call is explained by the stored request and has a terminal failed state in new runs.
- Focused desktop and runtime tests pass.
