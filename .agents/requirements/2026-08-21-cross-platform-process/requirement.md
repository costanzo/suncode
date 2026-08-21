# Requirement

## Background

The model-facing `bash` tool translated every request to `/bin/sh -lc`, which fails on the Windows desktop runtime before a process can start. The underlying process operation already supports argv-based execution, but the tool boundary conflated structured processes with shell scripts.

## Goals

- Make structured process execution portable across Windows, macOS, and Linux.
- Provide an explicit platform shell capability for scripts that need shell syntax.
- Tell the model the host OS, shell dialect, path style, and current local time.
- Return stable, actionable process-start error codes without exposing secrets.

## Non-goals

- Translating Bash scripts into PowerShell or vice versa.
- Adding an unrestricted HTTP/weather capability in this delivery.
- Claiming OS-level sandboxing for child processes.

## Requirements

- `process.run` accepts `program` plus string `args`; it must not invoke a shell implicitly.
- `shell.run` accepts a script and selects `/bin/sh -lc` on Unix and Windows PowerShell on Windows.
- The old `bash` operation remains an internal compatibility alias but is no longer advertised to new model calls.
- Process start failures distinguish executable-not-found, permission-denied, working-directory-unavailable, and generic start failures.
- Windows child processes run without allocating or displaying a console window.
- The runtime context is ephemeral prompt data and contains no credentials or absolute project paths.

## Edge cases

- A caller may still send legacy `command` arguments to `process.run`; they are accepted only as a single executable for compatibility, never parsed as shell text.
- Shell syntax remains platform-specific and is not silently translated.
- The default shell executable must be available through the platform installation; missing shell binaries return a diagnostic start code.

## Acceptance criteria

- The Windows path no longer attempts to start `/bin/sh` for a shell tool.
- Windows shell and structured process calls do not flash a console window in the desktop application.
- Unix builds retain `/bin/sh -lc` behavior.
- Structured process calls execute without shell interpolation on all supported targets.
- Focused unit tests cover shell command selection and process argument validation.
- Rust formatting, focused tests, and `git diff --check` pass.

## Open questions

- A separately authorized HTTP client/weather tool remains a future requirement.
