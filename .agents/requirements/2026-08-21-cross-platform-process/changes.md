# Changes

## Source

- Replaced the advertised `bash` tool with separate `process` and `shell` definitions.
- Added Windows PowerShell and Unix POSIX shell command selection.
- Kept legacy `bash` dispatch for persisted approval continuations.
- Added platform-specific filtered environment entries and stable spawn/cwd errors.
- Configured every Windows child process with `CREATE_NO_WINDOW` so desktop shell and structured process calls do not flash a console window.
- Added ephemeral host OS, architecture, shell, path, timestamp, and weekday provider context.
- Preserved operation failure codes and details through tool and turn state.

## Contracts and generated artifacts

- Documented the current embedded `process/run` argv semantics.

## Configuration and persistence

- None planned.

## Tests

- Added operations tests for argv parsing, compatibility, failure classification, and actual host shell execution.
- Added a Windows-specific check for the no-window process creation flag.
- Added core tests for tool advertisement, platform shell translation, structured argv, policy, and host context.

## Documentation

- Updated architecture, runtime feature/specification, decision index, and this requirement package.
