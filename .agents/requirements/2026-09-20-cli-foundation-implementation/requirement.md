# Requirement

## Background

The CLI architecture and SDK host capability ceiling are approved and implemented. The repository still has no `apps/cli` executable, so command parsing, configuration precedence, machine output, credential input, exit mapping, and SDK lifecycle behavior are not executable or testable before the conversational loop is added.

## Goals

- Create a buildable native Rust binary at `apps/cli` depending only on the Rust SDK and terminal/argument/serialization libraries.
- Open the SDK with Browser and Computer host capabilities disabled.
- Implement `doctor`, `models`, `auth list`, `auth set`, `auth remove`, and `config list`.
- Implement `--output text|jsonl`, `--color`, and `--user-id` with their approved `SUNCODE_` environment variables.
- Preserve explicit SDK shutdown on successful and failed commands after startup.
- Implement stable CLI error-to-exit mapping for the foundation commands.
- Read credentials only from a no-echo interactive terminal.

## Non-goals

- Implementing `run`, `chat`, or session commands.
- Consuming live session events.
- Interactive approvals, structured questions, or signal-driven turn cancellation.
- Named policy profiles or general CI execution.
- TUI, PTY, Browser Use, Computer Use, daemon, or IPC support.
- Distribution, signing, shell completions, or automatic updates.

## Requirements

- The binary name is `suncode` and the package remains independent from the agent workspace.
- Argument parse failures exit 2; help and version exit 0.
- Explicit options override `SUNCODE_OUTPUT`, `SUNCODE_COLOR`, and `SUNCODE_USER_ID`.
- Invalid `SUNCODE_` values fail instead of falling back.
- The logical default user is derived locally and remains an ownership label, not authentication.
- JSONL output uses the contract envelope with `schema_version: 1` and no session/turn IDs for administrative commands.
- Human reports use stdout; errors and SDK diagnostics use stderr.
- `auth set` refuses non-interactive stdin/stderr and never accepts API keys through environment variables or command arguments.
- SDK business errors map to the documented exit status groups.
- The CLI calls consuming SDK shutdown before emitting a successful report.
- The CLI capability profile disables Browser and Computer Use.

## Edge cases

- Fresh data directory and database.
- Data directory already owned by another process.
- Invalid output/color environment values.
- Broken stdout after an SDK command succeeds.
- Shutdown failure after a command succeeds.
- Credential set invoked through a pipe.
- Unknown provider removal or credential update.

## Acceptance criteria

- Unit tests cover parsing, precedence, invalid environment, JSONL envelope, and exit mapping.
- Integration tests execute the built binary against isolated data directories.
- `doctor --output jsonl`, `models`, and non-TTY `auth set` behavior match the contract.
- CLI Clippy, locked/offline build, broad Rust/native/desktop regression, formatting, and `git diff --check` pass.

## Open questions

- Conversational lifecycle and terminal rendering belong to the next CLI delivery.
