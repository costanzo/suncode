# SunCode CLI Contract

Status: Administrative commands, one-shot `run`, and session list/resume/archive implemented; interactive `chat` remains deferred.

## Purpose and boundary

The SunCode CLI is the next approved production client after the Avalonia desktop reference client. It is a native Rust executable under `apps/cli` that embeds `suncode-sdk::AsyncAgentSdk` in-process on one Tokio runtime. It does not call the C ABI, open SQLite directly, contact providers directly, duplicate policy or agent behavior, start a client-facing server, or attach to another SunCode process.

The CLI owns argument parsing, prompt-source input, text and JSONL rendering, signal handling, and exit status. Rust core and the SDK continue owning projects, sessions, turns, providers, credentials, policy, approvals, questions, operations, persistence, recovery, undo, MCP, LSP, and runtime resources.

## Initial capability profile

The first implementation targets coding workflows through built-in file/search/process/web tools plus configured MCP and LSP integrations. TUI, PTY, Browser Use packaging, and Computer Use interaction are separate deliveries.

SDK startup accepts typed `SdkOpenOptions` with an immutable host capability ceiling. The CLI passes `browser_use: false` and `computer_use: false`, disabling Browser Use and Computer Use advertisement and backend initialization even if their global persisted settings were enabled by the desktop host. This is a host presentation/capability ceiling, not an authority grant and not a mutation of the persisted settings.

## Process and data ownership

One CLI process embeds one SDK instance. The CLI uses the ordinary SunCode data directory and database unless `SUNCODE_DATA_DIRECTORY` or `SUNCODE_DATABASE_PATH` selects another location. The existing single-instance data-directory lock remains authoritative: the CLI cannot open the same data directory while the desktop or another CLI process owns it. The CLI reports `agent_already_active` with guidance to close the other host or choose another `SUNCODE_DATA_DIRECTORY`.

Cross-process attach, a background daemon, socket discovery, and desktop-to-CLI handoff remain deferred. The CLI always calls consuming SDK shutdown before normal exit.

## Command surface

The implemented line-oriented command families are:

```text
suncode run [PATH] (--prompt TEXT | --stdin)
suncode session list [PATH]
suncode session resume SESSION_ID (--prompt TEXT | --stdin)
suncode session archive SESSION_ID
suncode models
suncode auth list
suncode auth set PROVIDER
suncode auth remove PROVIDER
suncode config list
suncode doctor
```

`run` creates a primary session in the canonical project represented by `PATH`, submits one turn, and exits. `session resume` reopens an existing primary session, submits one new turn using its durable conversation context, and exits. Both require exactly one prompt source and finish after completion, suspension, cancellation, or failure.

`session list` opens the selected project through the SDK and includes active and archived primary sessions; `session archive` applies the SDK-owned lifecycle transition. Interactive `chat` is not part of the implemented command surface and remains rejected by argument parsing.

Resume does not print existing history or resolve a suspended interaction. If the session has a durable pending approval or question, it fails closed with exit status 4 without contacting the provider. A future interactive client may resolve those states through the named SDK methods.

The command grammar is add-only within the first major CLI contract. A later full-screen TUI must use a separate subcommand or executable mode and must not silently replace line-oriented CLI behavior.

## Turn and event flow

The shared one-shot `run`/`session resume` flow:

1. opens `AsyncAgentSdk`;
2. creates a new primary session or reopens the supplied primary session;
3. calls atomic `watch_session` before submitting the turn;
4. for resume, rejects a durable pending approval or question before submission;
5. consumes and renders the typed fused event stream while the turn is submitted;
6. drains events already published before rendering the terminal response;
7. repeats atomic watch after lag;
8. reports a new approval or question suspension with exit status 4;
9. calls consuming SDK shutdown before exit.

The CLI never combines standalone snapshot and subscription calls as if they were atomic.

## Approvals and questions

The implemented CLI never reads an approval or question answer from stdin after the turn prompt has been consumed. `run` and `session resume` return exit status 4 for either suspension. Without a matching pre-authorized policy profile, an operation requiring approval fails closed and a structured question produces the documented non-interactive outcome.

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

During an active turn, the first interrupt requests `cancel_turn` once the active turn ID is known and continues draining events until the turn reaches a terminal state. A second interrupt stops waiting and exits with status 130 after SDK shutdown, which is bounded by the SDK shutdown grace period. When no turn is active, interrupt stops the command and exits with status 130 after SDK shutdown.

Normal return, argument failure after SDK open, event-stream failure, and a panic during command execution all attempt consuming SDK shutdown. A contained panic is reported as `internal_error` with exit status 1. Secrets and user/provider content are not written to diagnostics during this path.

## Configuration and environment

All SunCode-owned environment variables begin with `SUNCODE_`. The approved CLI registry is:

| Variable | Purpose |
| --- | --- |
| `SUNCODE_DATA_DIRECTORY` | Override the application data root |
| `SUNCODE_DATABASE_PATH` | Override the SQLite database path |
| `SUNCODE_NON_INTERACTIVE` | Force fail-closed non-interactive policy behavior |
| `SUNCODE_USER_ID` | Override the logical local user ID |
| `SUNCODE_MODEL` | Select or override the model for a CLI turn |
| `SUNCODE_REASONING_EFFORT` | Select the default reasoning effort when supported |
| `SUNCODE_OUTPUT` | Select `text` or `jsonl` output |
| `SUNCODE_LOG_OUTPUT` | Select diagnostic console mirror: `none`, `stderr`, `stdout`, or `both` |
| `SUNCODE_POLICY_PROFILE` | Select a future Rust-owned non-interactive policy profile |
| `SUNCODE_COLOR` | Select `auto`, `always`, or `never` |

Precedence is explicit command-line option, then the corresponding `SUNCODE_` environment variable, then persisted project/session configuration where applicable, then the documented default. An invalid value fails with exit status 2 rather than silently falling back.

The `--log-output` global option controls only the Rust diagnostic logger's console mirror for this CLI process. The configured `agent.log` file remains active. It defaults to `stderr`; use `--log-output none` when stdout and stderr must not contain diagnostic log lines.

CLI startup selects the process environment source for `suncode-config`. Avalonia and the blocking/native default host select the disabled source, so inherited `SUNCODE_*` variables do not change desktop behavior. Provider credentials may be supplied to a CLI process through `SUNCODE_OPENAI_API_KEY`, `SUNCODE_ANTHROPIC_API_KEY`, `SUNCODE_DEEPSEEK_API_KEY`, `SUNCODE_QWEN_API_KEY`, `SUNCODE_ZHIPU_API_KEY`, `SUNCODE_KIMI_API_KEY`, or `SUNCODE_GEMINI_API_KEY`; these values are an in-memory overlay, never persisted to SQLite, child processes, logs, traces, or JSON output. When absent, provider credentials continue to resolve from SQLite.

Runtime CLI overrides include `SUNCODE_FULL_CONTROL`, `SUNCODE_TOOL_ALLOWLIST` (comma-separated tool names), `SUNCODE_TURN_TIMEOUT_MS`, `SUNCODE_BASH_TIMEOUT_MS`, and `SUNCODE_TOOL_CALL_LIMIT`. Runtime values resolve environment over session, project, and global SQLite settings. The effective tool ceiling is intersected with specialist and host capability ceilings, the bash timeout is capped rather than expanded by a turn request, and permanent hard-deny command rules remain unconditional. The effective runtime configuration is snapshotted when a turn is admitted.

## Packaging

The CLI is built as a Rust binary depending on `sdks/rust`; it is not part of the agent crate workspace and does not make the SDK depend on CLI libraries. Initial release artifacts do not bundle the Browser Use runtime and do not claim Browser or Computer Use support. Platform packaging, signing, completions, manpages, and update distribution require focused release work before the CLI is described as shipped.

## Deferred CLI scope

- Interactive multi-turn `chat` and terminal approval/question resolution.
- Full-screen TUI and alternate-screen rendering.
- PTY sessions and interactive child processes.
- Background daemon or cross-process attach.
- Browser Use and Computer Use terminal UX and packaging.
- Shell integration that mutates the parent shell directory or environment.
- Automatic updates, plugin installation, and hosted execution.
