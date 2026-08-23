# Architecture

## Current state

`MessageItem` stored one compact detail string and the chat template rendered it inline beneath each tool name. The agent translated tool arguments after marking the tool as executing, so translation errors could leave the persisted tool state non-terminal.

## Proposed design (superseded)

The desktop projection retains separately formatted request, result, and error strings on each tool message. The conversation renders only `ToolSummaryText` and `ToolStateText`; clicking the card populates a local Avalonia overlay with the retained sections. The overlay is presentation-only and does not alter SDK or database contracts. The model-facing shell contract described here was later superseded by the OpenCode-compatible `bash(command, timeout, workdir)` contract.

The Rust agent marks a tool failed when `prepare_call` or method lookup fails after authorization. Shell translation accepts a non-empty legacy `command` fallback for compatibility, while the advertised shell contract remains `script` and empty input is rejected.

## Boundaries and dependencies

- Avalonia owns summary text, dialog presentation, and transient selection state.
- Rust core owns argument validation, tool state transitions, and compatibility translation.
- SQLite continues storing original tool requests, results, and error codes through existing events.

## Data and control flow

tool events -> desktop projection -> concise tool card -> detail overlay

model shell call -> argument translation -> failed tool state on invalid input -> failed turn

## Security and failure handling

The detail dialog is read-only. Raw request/result text remains selectable but no new execution path is introduced. Invalid shell input fails closed before process creation.

## Compatibility and migration

No protocol or persistence migration is required. Existing snapshots without separate request/result fields fall back to the existing detail payload.

## Risks and rollback

The Avalonia overlay adds a local modal surface; reverting the chat template and projection fields restores inline rendering. The Rust state transition change is independent and can be retained without the UI change.

## Open questions

- Whether a later client should provide a shared operation-detail component for provider traces and tool cards.
