# Requirement

## Background

SunCode exposed a custom `shell(script)` model tool while Pi and OpenCode expose `bash(command)`. A model call used the wrong argument field and failed before execution.

## Goals

- Match OpenCode's model-facing bash tool name and argument names.
- Use millisecond timeout semantics compatible with OpenCode.
- Keep historical SunCode shell requests executable through an internal compatibility path.

## Non-goals

- Changing the audited process operation or platform shell selection.
- Changing structured `process` timeout semantics.
- Making historical database rows look like new model requests.

## Requirements

- New tool definitions advertise `bash` with required `command`.
- New bash requests accept optional integer `timeout` in milliseconds, maximum 600000, and optional `workdir`.
- Historical `shell`, `script`, `shell`-field, `cwd`, and seconds-based timeout inputs remain internally translatable.
- New provider context guidance names the `bash` tool.
- Recoverable argument failures are returned to the model as tool results and do not terminate the current turn.

## Edge cases

- Empty commands fail with `invalid_arguments` before process creation.
- New timeout values of zero, non-integers, or values over 600000 fail closed.
- Invalid or malformed tool arguments include an error code and message in the matching `tool_call_id` result before the next model request.
- Legacy shell aliases continue to use their previous seconds-to-milliseconds conversion.

## Acceptance criteria

- The outgoing built-in tool schema contains `bash`, `command`, integer millisecond `timeout`, and `workdir`.
- A bash request with `{"command":"echo ok","timeout":120000}` translates to the platform shell with `timeout_ms=120000`.
- Focused Rust and Avalonia tests pass.
