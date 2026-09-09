# Requirement

## Background

SunCode currently exposes a fixed Rust-owned tool catalog. Users need to configure Model Context Protocol (MCP) servers from the desktop Settings window and make their tools available to existing sessions without restarting the application or creating another session.

The implementation must preserve SunCode's existing ownership and authority boundaries: Rust owns persistence and execution, Avalonia consumes typed SDK operations, and an MCP server is not a trusted sandbox merely because it is reached through a protocol client.

## Goals

- Add a first-level `MCP servers` destination to Settings.
- List, create, edit, and delete global MCP server configurations.
- Enable or disable each server independently.
- Persist desired MCP configuration in Rust-owned SQLite.
- Apply every successful mutation to the embedded runtime immediately so the next turn in an existing session uses the new effective MCP tool catalog.
- Show useful connection state and errors in Settings.
- Support local stdio servers and remote Streamable HTTP servers in the initial delivery.
- Keep every model-initiated MCP tool invocation visible, policy-controlled, and audited as session tool activity.

## Non-goals

- MCP prompts, resources, resource templates, sampling, elicitation, roots, or OAuth in the initial delivery.
- Importing OpenCode or pi configuration files.
- Installing MCP server packages or managing their software dependencies.
- Treating a local MCP child process or remote MCP server as isolated or trusted.
- Allowing MCP tools to bypass approval, operation journaling, cancellation, output bounds, or diagnostics redaction.
- Changing MCP configuration from CLI, TUI, Web, or an executable extension.

## Requirements

### Settings experience

1. Settings has one first-level `MCP servers` navigation item.
2. The page presents a compact list with server name, transport summary, effective runtime status, discovered tool count, enabled state, edit, delete, and retry actions where applicable.
3. Create and edit use one shared separate native window without a backdrop. Closing that window discards unsaved edits. Delete uses the shared confirmation dialog and names the affected server.
4. The create/edit window supports:
   - display name;
   - enabled state;
   - local stdio or remote Streamable HTTP transport;
   - local command, arguments, working directory, and environment entries;
   - remote URL and HTTP header entries;
   - startup and per-request timeouts.
5. Secret-looking environment and header values are masked after persistence and never returned in plaintext by read APIs.
6. A mutation stays visibly pending until SQLite persistence and runtime reconciliation have completed or failed.

### Persistence and contracts

1. Rust is the only owner of MCP configuration persistence.
2. Server definitions are global to the local embedded agent in the initial delivery and therefore apply to every project using that data directory. Runtime connections are project-scoped so local working directories and advertised MCP roots never cross project boundaries.
3. Desired configuration is stored in a dedicated SQLite table. Transient runtime status, discovered tool definitions, process identifiers, and errors are not persisted as configuration.
4. CRUD and enable operations are exposed as named typed Rust SDK methods, then hand-implemented across the C and C# bindings.
5. Mutations use idempotency keys and optimistic revisions where duplicate creation or lost updates are possible.

### Runtime behavior

1. Enabled server definitions are loaded on agent startup. One connection instance is created per active project and reconciled when that project opens and after every mutation.
2. A successful create, update, enable, disable, or delete does not require application restart or a new session.
3. Disabling or deleting a server removes its definitions from new model requests immediately and prevents new invocations. A request already executing may finish; its result remains audited.
4. Editing a server invalidates the old connection before the updated configuration can serve new calls. A failed replacement remains failed and does not silently keep using stale configuration.
5. A turn obtains an effective MCP catalog at each provider-call boundary. Therefore a settings change is guaranteed to affect the next turn and may affect a later provider call in the current turn; a call emitted against a retired definition fails as a recoverable unavailable-tool result.
6. Tool-list-change notifications refresh that server's cached definitions without restarting the session.
7. Runtime state includes at least `disabled`, `connecting`, `connected`, and `failed`. Failed state includes a redacted user-safe error.
8. Tool names are deterministically namespaced per server and mapped back to an opaque server/tool handle. Collisions or provider-incompatible schemas fail closed and keep the affected server out of the effective catalog.
9. Settings shows runtime status for the current project connection. If no project is active, it shows the desired configuration and a not-started state rather than implying a global connection exists.

### Authority and safety

1. MCP server enablement is explicit user intent to start a local child process or contact a remote endpoint; Settings must identify this consequence.
2. Every model-initiated MCP tool call is classified as an opaque external operation. Interactive use requires approval unless the session already has Full Control; non-interactive use denies the call without an explicit future profile grant.
3. Approval text identifies the server, remote tool name, and arguments. Approval is revalidated against the current enabled connection before execution.
4. MCP tool calls use the existing `session_tool_use` lifecycle and appear in Tool activity.
5. MCP results are bounded before entering model context. Text and structured JSON are supported initially. Unsupported binary/audio content produces a recoverable tool error rather than being silently discarded.
6. Cancellation propagates to the MCP request. Transport loss, protocol errors, server-declared errors, malformed schemas, and timeouts become redacted typed failures.
7. Local processes receive a deliberate, documented, OS-specific allowlist rather than an automatic copy of every host environment variable. The baseline supplies process launch, user/profile, locale, terminal/session, temporary-directory, working-directory, and standard toolchain lookup values; configured entries override it. Dynamic loader and runtime code-injection variables are not inherited implicitly.

## Edge cases

- Duplicate names and generated tool prefixes are rejected case-insensitively.
- An enabled server may have zero tools and still be connected.
- A server can disconnect after Settings is closed; reopening or refreshing Settings reads current runtime state.
- Delete during connect cancels the pending connection and prevents a late completion from reinstalling the server.
- Rapid edits and toggles use a per-server generation/revision so stale asynchronous work cannot win.
- A remote URL must use HTTPS by default. Loopback HTTP is allowed for local development; other plaintext HTTP requires an explicit future decision.
- Renaming a display name does not change the immutable tool prefix created with the server.
- Secrets are write-only in SDK DTOs: an omitted value preserves the stored secret and an explicit remove action deletes it.

## Acceptance criteria

- The reviewed design-system Settings specimen demonstrates the separate create/edit window plus all CRUD and enable/disable paths, including failure and retry states, in both themes and at the documented minimum width.
- Focused persistence tests cover CRUD, uniqueness, revisions, secret redaction, and additive initialization of a prior current database.
- Runtime tests prove next-provider-call catalog replacement, stale-reconcile suppression, notification refresh, cancellation, and graceful disconnect.
- Agent tests prove policy/approval behavior, audit rows, recoverable failures, output bounds, and no new-session requirement.
- SDK contract tests cover Rust, C, and C# DTO parity and error mapping.
- Avalonia tests cover navigation, dialog validation, mutation pending states, status rendering, keyboard behavior, and deletion confirmation.

## Confirmed scope

- The first delivery exposes MCP tools only; prompts, resources, and OAuth remain deferred.
- Server definitions are global and connection instances are project-scoped.
- Opaque MCP tools require approval per call unless Full Control is active.
- Initial transports are local stdio and remote Streamable HTTP; legacy SSE is deferred.
- MCP create and edit use a separate native window without a backdrop.
