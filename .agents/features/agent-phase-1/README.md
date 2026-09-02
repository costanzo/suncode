# Embedded Rust Agent Phase 1

**Status:** Implemented and focused-tested

The Phase 1 agent is an embedded Rust SDK. It owns provider access, turn orchestration, policy, approvals, durable state, credentials, recovery, undo, and the native SDK surface. It runs in-process with the .NET 10 Avalonia client; there is no client-facing server, loopback transport, or production TypeScript runtime.

Turn submission supports a compatibility text-only method and an attachment-aware method accepting up to three persisted same-session image IDs. Core validates ownership, uniqueness, file bounds, format, and the selected model's vision capability; user messages persist `image_ref` parts, provider calls receive transient data URLs, and provider traces retain redacted attachment markers. Image-bearing submissions are not queued behind an active turn.

## Agent workflow

- Sessions are scoped to a project. Turns support idempotent admission, queued follow-up input, cancellation, model selection, optional `low`/`medium`/`high` reasoning effort, context-window-aware compaction, and cumulative provider-reported usage.
- The provider receives canonical messages, host platform/shell context, and a bounded project-root `AGENTS.md` system message on every request. Successful project reads can attach unseen nested instructions, nearest-first, within bounded size and depth limits.
- The model-facing catalog contains exactly `read`, `glob`, `grep`, `question`, `todowrite`, `write`, `edit`, `bash`, and `webfetch`. Invalid or malformed arguments are recoverable tool failures; authorization, approval, and agent failures remain terminal.
- `question` durably suspends a turn for one to eight structured prompts and resumes or rejects it through the SDK. `todowrite` replaces up to 100 prioritized items, allowing at most one `in_progress` item, and projects the current list in normalized turn snapshots.

## Providers and settings

The `suncode-llm` package owns provider-neutral messages, tool schemas, completion results, usage normalization, model routing, and OpenAI-compatible HTTP/SSE. It has no database dependency. Core loads the enabled catalog from `llm_model_provider` and `llm_model`, resolves credentials, and injects them through an internal trait. The seeded catalog has six providers with two models each: DeepSeek, Zhipu GLM, OpenAI, Kimi, Claude, and Gemini. Custom providers are trusted in-process Rust registrations or persisted OpenAI-compatible rows; dynamic plugins are not supported. The named SDK endpoint method validates and persists an existing provider's URL and atomically replaces its live route for subsequent calls while preserving credentials, models, and catalog ordering.

Configuration uses global, project, and session overlays. `tool_call_limit` is project-only, defaults to 64, and accepts 1-256. `full_control` is session-scoped and only skips repeat interactive approval for known risks; validation, scope checks, audited dispatch, checkpoints, cancellation, and unknown-tool denial still apply. Global `verify_https_certificates` defaults to `true`; disabling it makes subsequent built-in provider and WebFetch requests accept invalid certificates and hostnames, equivalent to `curl -k`, without bypassing other network controls. Provider API keys are loaded exclusively from plaintext SQLite values; environment-variable credentials are ignored. Keys never appear in DTOs, events, diagnostics, or logs.

The Rust agent writes rotating `agent.log` diagnostics. SDK FFI errors and panics, startup/close, event serialization failures, subscription lag/channel termination, and background turn failures are logged at the owning boundary with operation, session/turn, error code, retryability, and provider request ID where available. Sensitive request and response content is excluded from diagnostics.

## Authority and recovery

Every machine-affecting call passes argument validation, declared-risk policy evaluation, approval when required, audit, and the narrow in-process operations dispatcher. Filesystem mutations capture pre-images and expose turn-level checkpoint manifests for conflict-aware undo. Process execution uses explicit structured or platform-native shell semantics, filtered environment, project-scoped working directories, bounded artifacts, process-tree cancellation, and honest failure states. WebFetch is approval-gated, same-origin redirect limited, text-only, bounded to 5 MiB, and previewed to 64 KiB.

Startup acquires a single-instance data-directory lock, opens the current SQLite schema, reconciles interrupted work, and reports unknown completion instead of replaying blindly. Live events are in-memory notifications; lagged subscribers receive `resync.required` and reload a normalized snapshot.

## Contracts and verification

The public native contract is [`contracts/agent-sdk/README.md`](../../../contracts/agent-sdk/README.md). Persistence rules are in [`contracts/persistence.md`](../../../contracts/persistence.md) and [`contracts/sqlite-schema.md`](../../../contracts/sqlite-schema.md). Focused Rust tests cover the agent, LLM, data, database, and operations crates.
