# Architecture

## Current state

The Rust core builds the nine built-in tool definitions directly for every provider request and resolves known calls through core policy into `suncode-tool::Operations`. Unknown tool names fail closed. SQLite has 15 normalized tables, the Settings client uses named SDK methods, and provider endpoint changes already demonstrate persist-then-replace runtime behavior.

There is no MCP dependency, connection manager, external tool registry, or MCP persistence contract today.

Reference findings:

- OpenCode keeps desired configuration separate from in-memory clients, statuses, cached definitions, and instructions. It closes stale clients, watches `notifications/tools/list_changed`, and only materializes definitions from connected servers.
- The inspected pi coding-agent tree does not contain an equivalent built-in MCP manager. Its applicable pattern is dynamic tool registration: the effective tool set is rebuilt from currently registered tools instead of being fixed for the whole session.
- Neither reference's TypeScript runtime, trust assumptions, or configuration storage is suitable as a SunCode production dependency.

## Proposed design

### Package boundary

Add `agent/crates/mcp` (`suncode-mcp`) as a protocol adapter. It owns:

- MCP client negotiation and protocol DTO conversion;
- stdio and Streamable HTTP transports;
- connection cancellation and timeouts;
- cached server capabilities and tool definitions;
- tool-list-change and connection-close notifications;
- request execution and MCP result normalization.

It does not own SQLite, session state, SDK DTOs, user approval, or the built-in operation dispatcher. Prefer the official Rust MCP SDK (`rmcp`) after a focused compatibility spike; pin the selected version and keep all SDK-specific types inside this crate.

Core adds an `McpManager` composition service containing:

- desired configuration loaded through `suncode-data`;
- one lifecycle slot per `(server_id, project_id)` with a monotonically increasing generation;
- current status and redacted error;
- an atomically replaceable effective catalog;
- active request cancellation/retirement handles.

Connections are project-scoped even though server definitions are global. A local stdio instance starts with that project's canonical root as its selected working directory, and remote clients advertise only that project's root when roots support is negotiated. Sessions in the same project share the connection; different projects never share a client or cached capabilities.

`suncode-tool` remains the owner of built-in definitions and machine operations. Core merges built-in definitions with the current project's MCP catalog snapshot before each provider call. MCP calls still pass through the same core validation, policy, approval, `session_tool_use`, cancellation, and result-recording pipeline before the manager sends a protocol request.

### Persistence model

Add a 16th current table:

```text
mcp_server
  mcp_server_id         TEXT PRIMARY KEY
  display_name          TEXT NOT NULL
  tool_prefix           TEXT NOT NULL
  transport_type        TEXT NOT NULL  -- stdio | streamable_http
  transport_config_json TEXT NOT NULL  -- validated tagged contract
  enabled               INTEGER NOT NULL
  sort_order            INTEGER NOT NULL
  revision              INTEGER NOT NULL
  created_at            TEXT NOT NULL
  updated_at            TEXT NOT NULL
```

`display_name` and `tool_prefix` have case-insensitive unique indexes. `transport_config_json` stores one validated versioned shape, not runtime state. Environment/header secrets are currently plaintext at rest, consistent with provider credentials, but read projections return only key names plus redacted presence/preview. Logs, errors, events, and tool traces never include secret values.

Database initialization recognizes the prior exact 15-table schema and may add the empty `mcp_server` table and indexes transactionally. Other structural mismatches remain rejected. This is an explicit additive compatibility case, not a general migration runner.

### SDK surface

Proposed Rust facade methods, mirrored through C and C#:

```text
list_mcp_servers(project_id?) -> [McpServerSummary]
create_mcp_server(project_id?, idempotency_key, input) -> McpServerDetail
update_mcp_server(project_id?, id, expected_revision, idempotency_key, input) -> McpServerDetail
set_mcp_server_enabled(project_id?, id, expected_revision, idempotency_key, enabled) -> McpServerDetail
delete_mcp_server(id, expected_revision, idempotency_key) -> DeleteResult
retry_mcp_server(project_id, id) -> McpServerDetail
```

Read DTOs expose desired configuration, revision, transport summary, the selected project's runtime status, tool count, and redacted failure. Secret fields use write-only set/remove operations. `retry` changes one project's runtime state only and does not mutate SQLite. Without a project ID, runtime status is `not_started` and no connection is created just to render Settings.

The first delivery refreshes Settings by reading this snapshot when the page opens, after every mutation, after retry, and on a low-frequency timer while the MCP page is visible. It does not overload the session event stream with global lifecycle events.

## Boundaries and dependencies

```text
Avalonia MCP Settings
  -> C# SDK -> C ABI -> Rust SDK facade
    -> suncode-data -> SQLite mcp_server (desired state)
    -> core McpManager.reconcile(server_id, project_id, revision)
      -> project-scoped suncode-mcp transport/client (effective state)

provider request
  -> built-in definitions + McpManager.catalog_snapshot()
  -> model emits namespaced MCP call
  -> validation -> policy -> approval -> session_tool_use audit
  -> McpManager.call(server/tool generation) -> MCP server
  -> bounded normalized result -> model context
```

The Avalonia client never opens SQLite or creates MCP transports. `suncode-llm` remains provider-neutral and receives only ordinary `ToolDefinition` values. No Node.js or Bun runtime enters production.

## Data and control flow

### Startup

1. Rust validates/opens SQLite and loads all MCP rows.
2. Disabled rows publish `disabled` runtime summaries without creating transports.
3. When a project opens, its enabled rows enter `connecting`; connection tasks run with bounded concurrency and that project alone is advertised as the MCP root.
4. A successful handshake and tool list atomically install one catalog generation.
5. Failures publish `failed` and install no tools; agent startup itself remains available.

### Create, update, enable, and disable

1. SDK validates the complete proposed record, URL/command constraints, uniqueness, secrets, and optimistic revision.
2. A transaction writes desired state and advances the revision/idempotency outcome.
3. The manager advances the server generation for every active project and immediately retires old catalogs from new snapshots.
4. Disable closes/cancels the transport and returns `disabled`.
5. Enabled configuration connects within the startup timeout. Success installs definitions; failure returns persisted desired state plus `failed` runtime state.
6. A late result whose generation/revision no longer matches is closed and discarded.

The database is authoritative for desired state. Runtime status is authoritative only in memory and is keyed by project. A persistence failure leaves runtime untouched. A connection failure does not roll back valid desired configuration and never restores a stale client.

### Delete

The transaction removes the row and records the idempotent mutation outcome. The manager retires every project catalog, cancels connection work, and closes each client. Existing executing requests retain only the handle needed to finish; no new request can resolve that server.

### Provider catalog and invocation

Before each provider request, core snapshots connected definitions and converts them to provider-neutral schemas. Names use `mcp__<immutable-prefix>__<remote-tool>` when provider limits allow; deterministic truncation plus a short hash preserves uniqueness otherwise. The mapping stores opaque server ID, generation, and original tool name.

At execution, core resolves the mapping, validates JSON arguments against the advertised object schema, assigns `Risk::ExternalTool`, and evaluates policy. After approval it revalidates that the server remains enabled and the generation is callable. Unknown, retired, or disconnected definitions return a recoverable tool result so the model can continue without that tool.

MCP calls are sequential by default because their side effects are opaque. A future explicit read-only capability and policy contract would be required before parallel execution.

## Security and failure handling

- `Risk::ExternalTool` always requires interactive approval unless Full Control is active; non-interactive mode denies it by default.
- Approval and Tool activity use the display name and original remote tool name, while audit records also retain stable IDs and the exposed namespaced name.
- Local stdio is launched without a shell from a structured executable plus argument list. The default working directory is the current project's canonical root; an application-data option never grants project authority to another project.
- The child receives a platform-specific allowlisted baseline plus configured entries. Unix-like systems include `PATH` (with standard Homebrew, local, Cargo, system, and host entries), `HOME`, user/login and shell identity, locale, terminal/display session variables, temporary directory, XDG directories, `PWD`, and common non-secret toolchain directory markers. Windows includes `PATH`, `SystemRoot`/`WINDIR`, user/profile and application-data directories, `ComSpec`, `PATHEXT`, username, OS, and temporary directories. Host values are used when available, deterministic safe defaults fill required runtime values, and configured entries override the baseline. Secret values are redacted before logging; dynamic loader and code-injection variables such as `LD_PRELOAD`, `DYLD_*`, and `NODE_OPTIONS` are never inherited implicitly.
- Remote URLs follow the global HTTPS certificate settings. HTTPS is required except loopback HTTP.
- Connection and request timeouts are bounded. Dropped connections atomically remove tools and become `failed`.
- Text and structured content are capped using the common tool-output bounding path. Unsupported MCP content types fail explicitly.
- Local process-tree termination uses the existing platform process primitives where possible. This is lifecycle cleanup, not sandboxing.
- MCP servers can perform arbitrary work outside SunCode's checkpoints. The approval UI must state that MCP changes may not be undoable by SunCode.

## Compatibility and migration

- Existing sessions need no stored change; MCP definitions are supplied only to future provider requests.
- Existing tool rows already store arbitrary names and can represent namespaced MCP calls, but projections/tests must verify length and display behavior.
- Existing databases receive one narrowly validated additive table upgrade.
- Built-in tool names and behavior remain unchanged.
- Removing the feature code after rollback leaves one unused recognized table; enabled servers must be disabled before downgrading to binaries that reject the 16-table schema.

## Risks and rollback

- Rust MCP SDK maturity and transport behavior need a spike before implementation commitment.
- Tool schemas may violate model-provider restrictions; reject only the affected server catalog and expose a redacted actionable error.
- A malicious local server inherits the launched process authority. Settings copy and approval language must not imply isolation.
- Large server catalogs increase provider input cost. Add a documented per-server and total tool-count bound during implementation.
- Long-lived local processes require reliable shutdown and stale-generation tests on macOS, Windows, and Linux.

Rollback disables all MCP rows, retires the runtime catalog, and leaves built-in execution unchanged. Schema rollback is not automatic.

## Open questions

- Global definitions with project-scoped connections versus fully separate per-project definitions.
- Tools-only initial scope versus prompts/resources/OAuth.
- Exact local working-directory choices and environment baseline.
- Per-server and aggregate tool-count limits.
- Whether Full Control should cover opaque MCP calls or require a distinct grant.
