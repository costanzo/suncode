# Architecture

## Current state

The agent advertised `bash`. Its argument translator replaced the command with `/bin/sh` and `-lc`, regardless of host platform. The operations crate launched the resulting program with `std::process::Command` and discarded the underlying spawn error.

## Proposed design

Keep `process.run` as a structured argv operation. Add a model-facing `process` tool for it and a `shell` tool for scripts. The shell translator chooses a platform-native shell command at the agent boundary; the audited operations layer remains responsible only for bounded process execution, cwd validation, environment filtering, output limits, and cancellation.

## Boundaries and dependencies

The agent owns tool schemas, compatibility aliases, and host prompt context. The operations crate owns process launching and stable failure classification. No client, provider, or SQLite code learns platform process details.

## Data and control flow

`process` tool -> `program`/`args` -> `process/run` -> `Command::new(program).args(args)`.

`shell` tool -> platform shell `program`/`args` -> `shell/run` -> the same bounded process runner.

## Security and failure handling

Shell execution remains approval-gated process execution. The environment stays filtered and the working directory stays project-relative. Spawn errors are classified into stable codes; credentials and complete environment contents are never serialized.

## Compatibility and migration

`bash` remains recognized as a legacy shell alias for persisted or in-flight calls. New tool definitions advertise only `process` and `shell`.

## Risks and rollback

PowerShell and POSIX shell syntax differ. The model context explicitly identifies the dialect, and users can still use structured argv calls for portable commands. Reverting the agent tool definitions restores the previous API, but would reintroduce the Windows defect.
