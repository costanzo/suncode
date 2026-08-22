# Requirement

## Background

Session switching needs comparable diagnostics at the Avalonia and Rust boundaries.

## Goals

- Provide reusable level-filtered logging in both production components.
- Include timestamps and execution context in every emitted record.
- Keep desktop and runtime records in separate files while mirroring to stderr.

## Non-goals

- Persisting logs in SQLite or the project knowledge base.
- Logging credentials, message bodies, file contents, or provider secrets.

## Requirements

- Support `TRACE`, `DEBUG`, `INFO`, `WARN`, and `ERROR`, with `SUNCODE_LOG_LEVEL` as the minimum-level filter.
- Use `SUNCODE_LOG_DIRECTORY` when set; otherwise write under the runtime data directory's `logs` folder.
- Write Avalonia records to `desktop.log` and Rust records to `runtime.log`.
- Include an RFC 3339 timestamp, level, process/thread context, component, and bounded diagnostic fields.
- Fall back to stderr if the log directory or file cannot be opened.
- Roll files when they exceed `SUNCODE_LOG_MAX_BYTES` (default 10 MiB) and keep at most `SUNCODE_LOG_RETENTION` backups (default 5).

## Edge cases

- Missing or invalid level values fall back to `INFO`.
- Unix log files use owner-only permissions when supported.
- A retention value of `0` keeps only the active file; invalid size/retention values use defaults.
- Tests or hosts that do not initialize the runtime logger still receive stderr diagnostics.

## Acceptance criteria

- Both projects compile and the Rust runtime tests pass.
- Session switching logs expose stage boundaries and subscription close/join boundaries.
- Separate desktop and runtime files are created in the configured directory.

## Open questions

- Log rotation is size-based; time-based rotation is deferred.
