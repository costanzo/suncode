# SunCode CLI Contract

Status: Foundation commands implemented; conversational commands remain in delivery.

## Purpose and boundary

The SunCode CLI is the next approved production client after the Avalonia desktop reference client. It is a native Rust executable under `apps/cli` that embeds `suncode-sdk::AsyncAgentSdk` in-process on one Tokio runtime. It does not call the C ABI, open SQLite directly, contact providers directly, duplicate policy or agent behavior, start a client-facing server, or attach to another SunCode process.

The CLI owns argument parsing, terminal capability detection, text and JSONL rendering, interactive approval and question prompts, signal handling, and exit status. Rust core and the SDK continue owning projects, sessions, turns, providers, credentials, policy, operations, persistence, recovery, undo, MCP, LSP, and runtime resources.

## Initial capability profile

The first implementation targets coding workflows through built-in file/search/process/web tools plus configured MCP and LSP integrations. TUI, PTY, Browser Use packaging, and Computer Use interaction are separate deliveries.

SDK startup accepts typed `SdkOpenOptions` with an immutable host capability ceiling. The CLI passes `browser_use: false` and `computer_use: false`, disabling Browser Use and Computer Use advertisement and backend initialization even if their global persisted settings were enabled by the desktop host. This is a host presentation/capability ceiling, not an authority grant and not a mutation of the persisted settings.

## Process and data ownership

One CLI process embeds one SDK instance. The CLI uses the ordinary SunCode data directory and database unless `SUNCODE_DATA_DIRECTORY` or `SUNCODE_DATABASE_PATH` selects another location. The existing single-instance data-directory lock remains authoritative: the CLI cannot open the same data directory while the desktop or another CLI process owns it. The CLI reports `agent_already_active` with guidance to close the other host or choose another `SUNCODE_DATA_DIRECTORY`.

Cross-process attach, a background daemon, socket discovery, and desktop-to-CLI handoff remain deferred. The CLI always calls consuming SDK shutdown before normal exit.

## Command surface

The complete initial command families are:

```text
suncode chat [PATH]
suncode run [PATH] (--prompt TEXT | --stdin)
suncode session list [PATH]
suncode session resume SESSION_ID
suncode session archive SESSION_ID
suncode models
suncode auth list
suncode auth set PROVIDER
suncode auth remove PROVIDER
suncode config list
suncode doctor
```

`chat` is an interactive multi-turn terminal conversation. `run` performs one submitted turn and exits after completion, denial, cancellation, question rejection, or failure. Both commands open or select the canonical project represented by `PATH`, defaulting to the current directory. They create a primary session unless a session ID is explicitly resumed.

The currently implemented foundation subset is `doctor`, `models`, `auth list`, `auth set`, `auth remove`, and `config list`. Unimplemented `run`, `chat`, and session commands are rejected by argument parsing rather than exposed as placeholders.

The command grammar is add-only within the first major CLI contract. A later full-screen TUI must use a separate subcommand or executable mode and must not silently replace line-oriented CLI behavior.

## Turn and event flow

For a new or resumed session the CLI:

1. opens `AsyncAgentSdk`;
2. opens/selects the project and creates or resolves the session;
3. calls atomic `watch_session` and renders its normalized snapshot;
4. consumes the typed fused event stream while a turn is submitted;
5. applies events idempotently;
6. repeats atomic watch after lag;
7. resolves approvals and questions through named SDK methods;
8. calls consuming SDK shutdown before exit.

The CLI never combines standalone snapshot and subscription calls as if they were atomic.

## Approvals and questions

Interactive approval prompts show the operation name, declared risk, bounded arguments, project/session context, and the exact choices `deny`, `allow_once`, and `allow_session`. The CLI does not invent broader grants. Structured questions preserve the SDK prompt order, available choices, multiple-selection rule, and custom-answer rule.

Prompts require an attached interactive terminal. A non-interactive stdin/stdout pipeline never attempts to read an approval or question from stdin after the user prompt has been consumed. Without a matching pre-authorized policy profile, an operation requiring approval fails closed and a structured question produces the documented non-interactive outcome.

The initial interactive CLI may ship before named non-interactive policy profiles. General-purpose CI execution is not complete until Rust core owns persisted or explicitly selected profiles with bounded risk, project, command, network, and external-tool grants. CLI flags may select a profile but cannot implement or widen policy themselves.

## Output modes

Human output is the default when stdout is a terminal. Assistant final text is written to stdout. Streaming status, tool activity, approval prompts, warnings, and diagnostics are written to stderr so stdout can be redirected.

`--output jsonl` produces UTF-8 JSON Lines on stdout only. Each line uses this envelope:

```json
{
  "schema_version": 1,
  "type": "event-or-result-name",
  "occurred_at": "RFC3339 timestamp",
  "session_id": "stable session id or null",
  "turn_id": "stable turn id or null",
  "data": {}
}
```

The JSONL renderer adapts typed SDK events and terminal outcomes; it does not expose provider wire payloads, database rows, credentials, absolute dependency paths, or diagnostic logs. JSONL field names and event meanings are add-only within `schema_version: 1`. Human formatting is not a machine contract.

## Exit status

The initial stable exit statuses are:

| Code | Meaning |
| --- | --- |
| `0` | Requested command or turn completed successfully |
| `1` | Agent turn or requested operation failed |
| `2` | Invalid arguments, environment, configuration, or input |
| `3` | Required provider/model/credential is unavailable |
| `4` | Approval, question, or policy prevented non-interactive completion |
| `5` | Data directory is already owned by another SunCode process |
| `130` | Interrupted by the user |

Provider HTTP status codes and OS process exit codes are never forwarded directly as the CLI process exit status.

## Signals and shutdown

During an active turn, the first interrupt requests `cancel_turn` and continues draining events until the turn reaches a terminal state or the shutdown deadline expires. A second interrupt requests immediate CLI termination but still attempts bounded SDK shutdown. When no turn is active, interrupt exits with status 130 after shutdown.

Normal return, argument failure after SDK open, event-stream failure, and panic containment all attempt consuming SDK shutdown. Secrets and user/provider content are not written to diagnostics during this path.

## Configuration and environment

All SunCode-owned environment variables begin with `SUNCODE_`. The approved CLI registry is:

| Variable | Purpose |
| --- | --- |
| `SUNCODE_DATA_DIRECTORY` | Override the application data root |
| `SUNCODE_DATABASE_PATH` | Override the SQLite database path |
| `SUNCODE_NON_INTERACTIVE` | Force fail-closed non-interactive policy behavior |
| `SUNCODE_USER_ID` | Override the logical local user ID |
| `SUNCODE_MODEL` | Select the default model for a new CLI session |
| `SUNCODE_REASONING_EFFORT` | Select the default reasoning effort when supported |
| `SUNCODE_OUTPUT` | Select `text` or `jsonl` output |
| `SUNCODE_POLICY_PROFILE` | Select a future Rust-owned non-interactive policy profile |
| `SUNCODE_COLOR` | Select `auto`, `always`, or `never` |

Precedence is explicit command-line option, then the corresponding `SUNCODE_` environment variable, then persisted project/session configuration where applicable, then the documented default. An invalid value fails with exit status 2 rather than silently falling back.

Provider API keys are deliberately excluded from the environment registry. `suncode auth set` reads a secret from an interactive terminal without echo and persists it through the SDK credential method. Provider-specific variables such as `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, and newly invented `SUNCODE_*_API_KEY` variables are ignored.

## Packaging

The CLI is built as a Rust binary depending on `sdks/rust`; it is not part of the agent crate workspace and does not make the SDK depend on CLI libraries. Initial release artifacts do not bundle the Browser Use runtime and do not claim Browser or Computer Use support. Platform packaging, signing, completions, manpages, and update distribution require focused release work before the CLI is described as shipped.

## Deferred CLI scope

- Full-screen TUI and alternate-screen rendering.
- PTY sessions and interactive child processes.
- Background daemon or cross-process attach.
- Browser Use and Computer Use terminal UX and packaging.
- Shell integration that mutates the parent shell directory or environment.
- Automatic updates, plugin installation, and hosted execution.
