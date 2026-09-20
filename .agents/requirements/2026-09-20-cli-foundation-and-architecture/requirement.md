# Requirement

## Background

CLI remained explicitly deferred while the reusable Rust SDK lacked native streams, caller-owned async execution, atomic session watch, and graceful shutdown. Those SDK prerequisites now exist. The current repository baseline also contains two contract drifts: the built-in Claude seed selects the `anthropic` adapter while the SQLite provider-table constraint accepts only `openai`, and the C ABI test hard-codes version 10 while the current SDK ABI is 13.

## Goals

- Remove CLI from deferred product and architecture scope without claiming it is implemented.
- Approve one native Rust CLI process topology over `AsyncAgentSdk`.
- Define command, event, output, approval, signal, configuration, environment, exit-status, packaging, and single-instance contracts.
- Require every SunCode-owned CLI environment variable to use the `SUNCODE_` prefix.
- Fix provider schema/seed compatibility for fresh and immediately previous current-schema databases.
- Make the C ABI conformance test compare against the authoritative ABI constant.
- Restore focused and broad repository verification before CLI implementation begins.

## Non-goals

- Creating the CLI crate or shipping executable commands in this delivery.
- Adding a TUI, PTY, daemon, IPC, Browser Use, or Computer Use CLI surface.
- Implementing non-interactive policy profiles in this delivery.
- Adding provider credentials through environment variables.
- Changing agent authority, operation, persistence, or event semantics for a client.

## Requirements

- `PRODUCT.md`, root guidance, and architecture identify CLI as the approved next production client and retain TUI/Web/mobile/IDE as deferred.
- The planned CLI embeds `AsyncAgentSdk` directly and never uses the C ABI, SQLite, provider implementations, or operation modules directly.
- The CLI cannot attach to an active desktop process; the existing data-directory lock remains authoritative.
- The initial CLI capability profile excludes Browser and Computer Use until focused host-capability and packaging work exists.
- Interactive and JSONL output have separate documented contracts.
- Interactive approvals/questions and non-interactive fail-closed behavior are explicit.
- All approved CLI environment variables begin with `SUNCODE_`; provider credentials remain SQLite-only.
- Fresh SQLite databases accept `openai` and `anthropic` provider adapters.
- Opening the immediately previous current-schema provider table rebuilds only that table and preserves configuration and credentials.
- The C ABI test references `SUNCODE_AGENT_SDK_ABI_VERSION` rather than a stale literal.

## Edge cases

- Desktop already owns the default data directory.
- stdin is used for the prompt and an approval is later requested.
- Event stream lags during a turn.
- First interrupt while a turn runs and a second interrupt during cancellation.
- Existing provider endpoint or credential must survive the narrow table rebuild.
- JSONL output is redirected while diagnostics remain enabled.
- Persisted Computer or Browser enablement exists when the future CLI starts.

## Acceptance criteria

- CLI is removed from deferred scope and described as approved but not implemented.
- `contracts/cli.md` is sufficient to start a separate implementation delivery without inventing ownership or wire behavior.
- Fresh and previous-compatible SQLite provider catalogs initialize with Claude on the Anthropic adapter.
- Rust SDK and C ABI tests pass the repaired baseline.
- Agent workspace, desktop, formatting, applicable Clippy, and `git diff --check` pass.

## Open questions

- Exact profile schema for pre-authorized non-interactive execution is a separate core-policy delivery.
- Browser/Computer capability negotiation and distribution are separate host-capability deliveries.
