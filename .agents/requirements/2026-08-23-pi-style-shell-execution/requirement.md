# Requirement

## Background

The Rust shell/process runner had a bounded reader that stopped consuming a stream after 256 KiB, passed model-facing timeout values through as milliseconds, killed only the direct child, and could not observe turn cancellation while a blocking operation was running. PI's shell runner continuously drains both streams, preserves a rolling preview plus full output, validates timeout values, and terminates process trees.

## Goals

- Make model-facing process and shell timeouts explicit seconds with a bounded 600-second maximum.
- Continuously drain stdout and stderr without allowing child processes to block on full pipes.
- Preserve a bounded tail preview and a complete artifact for oversized output.
- Propagate turn cancellation into blocking process execution.
- Terminate process groups on Unix and process trees on Windows.

## Non-goals

- Translating shell syntax between POSIX and PowerShell.
- Adding an OS sandbox or network isolation boundary.
- Adding interactive PTY support.

## Requirements

- timeout is a positive number of seconds at the model boundary and is converted to internal milliseconds.
- Invalid timeout values fail with invalid_arguments.
- Process output readers continue until EOF; preview output is bounded and full output is stored as an artifact when needed.
- Cancellation stops the child process and its descendants before the operation returns.
- Existing argv/process and platform shell contracts remain unchanged.

## Edge cases

- Commands that emit more than the preview limit must still complete instead of blocking on a pipe.
- Output containing non-UTF-8 bytes remains available through the base64 operation result or artifact.
- A missing artifact directory or temporary-file failure returns a stable retryable artifact error.

## Acceptance criteria

- Large-output and cancellation regression tests pass.
- Focused and workspace Rust tests pass.
- Existing process start error codes, project scope checks, and Windows no-window behavior remain intact.
