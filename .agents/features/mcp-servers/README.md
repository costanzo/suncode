# MCP Servers

**Status:** Implemented and focused-tested

The Rust agent supports Model Context Protocol tools from local stdio servers and remote Streamable HTTP servers. `suncode-mcp` is the protocol adapter, built on `rmcp`. Core owns the project-scoped runtime manager, the tool catalog, and policy. Desktop Settings has a first-level MCP servers page and a separate native create/edit window.

## What users can rely on

- Server definitions are global desired state in the `mcp_server` table. Live connections and their states are per project, so working directories never cross project boundaries. The states are `not_started`, `disabled`, `connecting`, `connected`, and `failed`. A failed state carries a redacted error.
- Create, update, enable, disable, delete, and retry are named SDK methods. They persist first, then reconcile at runtime, and need no restart and no new session. Writes use idempotency keys and optimistic revisions. Environment and header values are write-only (`{ set, remove }` patches), and reads return only key names.
- A stale connection generation cannot win. An edit retires the old connection before the new configuration serves calls. Deleting a server while it is connecting cancels the connection and prevents a late reinstall. A failed replacement stays failed and does not fall back to the old configuration.
- Existing sessions take a fresh MCP catalog before every provider call. Tool-list-change notifications refresh that server's tools. A call made against a retired definition returns a recoverable unavailable-tool result.
- Tools are exposed as `mcp__<prefix>__<tool>`, at most 64 bytes. The prefix is fixed when the server is created and does not change when it is renamed. Duplicate names and prefixes are rejected case-insensitively. Collisions and incompatible schemas keep the affected server out of the catalog. Limits: 128 tools per server, 256 MCP tools per project, and 1 MiB per result.
- Results support text and structured JSON. Binary and audio content returns a recoverable tool error. Cancellation reaches the MCP request. Transport, protocol, server, schema, and timeout failures become redacted typed errors.
- Every MCP call has `ExternalTool` risk and goes through `session_tool_use` audit. Interactive use requires approval unless the session has Full Control. Non-interactive use is denied. Approval is revalidated against the current connection. MCP side effects are outside SunCode undo, and a local process runs with the user's OS authority.
- Local servers start without a shell and receive the OS-specific environment allowlist, with configured entries applied last. Remote URLs must be HTTPS, except loopback HTTP, and use the global certificate and proxy settings.

## Background project loading

Opening or selecting a project does not wait for MCP. The Workspace calls `start_mcp_project`, which starts enabled servers in the background with at most four connecting at once and returns initial progress. `mcp_load_progress` reports `total`, `settled`, `connected`, `failed`, and `loading` from memory. The desktop shows a footer progress bar until every server has connected or failed. Chat and built-in tools work while servers are still connecting.

Not implemented: MCP prompts, resources, sampling, elicitation, roots, OAuth, legacy HTTP+SSE transport, and config import from other tools. Built-in child agents never receive MCP tools.

Contract: [`contracts/agent-sdk/README.md`](../../../contracts/agent-sdk/README.md). Decision: `ADR-20260908-mcp-server-runtime`.
