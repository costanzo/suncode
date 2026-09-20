# Architecture

## Current state

The Rust SDK exposes the necessary lifecycle and administrative methods, but no binary composes them for a terminal host.

## Proposed design

Create one independent Rust package at `apps/cli` with a `suncode` binary. Modules separate argument grammar, resolved configuration, command execution, output adaptation, and error/exit mapping. `main` owns SDK open and consuming shutdown.

The executable parses only implemented commands; `run` and `chat` remain explicit unknown-command errors rather than placeholders. SDK startup always uses `SdkOpenOptions` with Browser and Computer disabled. Administrative command execution returns an in-memory `CommandReport`; main awaits shutdown before writing a success report, preventing a successful machine result from preceding a failed cleanup.

Text mode writes reports to stdout and failures to stderr. JSONL mode writes one versioned result or error envelope to stdout, while SDK diagnostics remain on stderr. Foundation command envelopes use null session and turn identifiers.

Credential storage accepts only no-echo interactive terminal input. The credential value stays in memory for the SDK call and is never included in a report, error, argument, environment variable, or diagnostic.

## Boundaries and dependencies

- CLI depends on `suncode-sdk`, Tokio, Clap, Serde/JSON, Chrono, and rpassword.
- CLI does not depend on core, data, database, provider, tool, C, or C# crates directly.
- SDK remains unaware of CLI commands and rendering.
- Clap owns syntax validation only; it does not implement policy.
- Environment resolution recognizes only approved `SUNCODE_` variables.

## Data and control flow

1. Parse arguments without opening the SDK.
2. Resolve explicit options over `SUNCODE_` environment values and validate them.
3. Open `AsyncAgentSdk` with the CLI capability ceiling.
4. Execute one administrative SDK command and build a report.
5. Consume SDK shutdown.
6. Emit the success report, or emit the highest-priority command/shutdown/output error.
7. Return the stable exit status.

## Security and failure handling

Secrets never enter argv or environment. Non-TTY credential input fails closed. JSON errors expose only code and message. Data-directory contention maps to exit 5. Provider/model/credential availability maps to exit 3. The CLI does not weaken SDK validation or policy.

## Compatibility and migration

This is the first CLI executable. The JSONL envelope and exit statuses follow `contracts/cli.md`. Human column spacing is not a compatibility contract. No C ABI, persistence, or desktop change is required.

## Risks and rollback

Risks include secret leakage, stdout contamination, skipped shutdown, configuration ambiguity, and accidentally advertising unsupported tools. No-echo input, stream separation, deferred report emission, tested precedence, and SDK host capabilities address them. Rollback removes `apps/cli` without persistence changes.

## Open questions

- None for the administrative foundation.
