# Architecture

## Current state

The desktop embeds the blocking Rust SDK through C/C#. CLI is deferred despite the Rust SDK now exposing `AsyncAgentSdk`, typed fused event streams, atomic session watch, and consuming shutdown. Bootstrap configuration already uses `SUNCODE_DATA_DIRECTORY`, `SUNCODE_DATABASE_PATH`, and `SUNCODE_NON_INTERACTIVE`. Provider credentials are SQLite-only.

## Proposed design

Approve `apps/cli` as a future native Rust binary and direct consumer of `AsyncAgentSdk`. It owns only terminal concerns. It runs one Tokio runtime, creates/selects a project and primary session through named SDK methods, establishes atomic watch, renders typed events, resolves interactive approvals/questions, maps terminal signals to cancellation, and consumes SDK shutdown.

The initial product has two line-oriented execution modes: interactive `chat` and one-turn `run`. Administrative command families expose models, credentials, sessions, configuration, and diagnostics through SDK methods. Machine consumption uses a versioned JSONL envelope. A TUI is not implied by or embedded in the initial CLI.

The existing process-local data-directory lock is preserved. Desktop and CLI are peer hosts, not client/server processes, and cannot concurrently own the same data directory. A different `SUNCODE_DATA_DIRECTORY` creates an independent local agent state.

The SDK now provides typed host startup options, so CLI can impose a capability ceiling excluding Browser and Computer Use without mutating shared persisted settings. General-purpose non-interactive automation still needs Rust-owned named policy profiles; CLI parsing cannot become a parallel authority implementation.

## Boundaries and dependencies

- `apps/cli` depends on `sdks/rust`, Tokio, argument parsing, terminal I/O, secret input, and serialization libraries.
- `sdks/rust` does not depend on the CLI.
- CLI never depends on `suncode-data`, `suncode-llm`, `suncode-tool`, C, or C#.
- CLI rendering consumes SDK DTOs and typed events; it does not interpret provider wire messages or database rows.
- Core remains the only policy decision maker.
- Provider credentials remain stored through SDK methods and are never environment-backed.

## Data and control flow

1. Parse arguments and the bounded `SUNCODE_` environment registry.
2. Detect terminal capabilities and select text or JSONL rendering.
3. Open `AsyncAgentSdk` with CLI host capabilities and logical user ID.
4. Open/select the project and create/resume a session.
5. Establish atomic watch and render the snapshot.
6. Submit a turn while concurrently consuming typed events and signals.
7. Resolve approval/question continuations only through SDK methods.
8. On lag, replace local state with a new atomic watch.
9. On completion or error, consume SDK shutdown and map the outcome to the CLI exit contract.

## Security and failure handling

CLI does not weaken project scope, policy, approval, audit, checkpoint, or undo rules. Human prompts require a TTY; redirected or non-interactive execution fails closed instead of reading surprise input. JSONL never includes secrets or provider wire content. `auth set` uses no-echo terminal input. The first interrupt requests turn cancellation; shutdown remains bounded and runs on every exit path after SDK startup.

## Compatibility and migration

This delivery changes no CLI executable because none exists. It changes product scope from deferred to approved/planned. The SQLite fix is a narrow current-schema compatibility action: only the provider table is rebuilt when its SQL check constraint lacks `anthropic`, and all row fields are copied before the legacy table is dropped. There is still no general migration framework or schema version.

## Risks and rollback

The largest CLI risks are duplicating policy in argument parsing, unstable machine output, consuming stdin twice, assuming desktop coexistence, and accidentally enabling host capabilities without terminal UX. The contract prohibits these patterns. Rollback may return CLI to deferred scope without persistence changes; the provider constraint repair remains valid independently.

## Open questions

- Policy profile schema and management commands.
- Platform packaging, signing, completions, and distribution.
