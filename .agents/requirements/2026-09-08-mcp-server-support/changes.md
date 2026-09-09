# Changes

## Source

- Add a Rust MCP protocol/runtime crate and core lifecycle manager.
- Merge the effective MCP catalog into provider calls and route namespaced calls through existing policy/audit flow.
- Add data operations and database resources for `mcp_server`.
- Add Rust, C, and C# SDK methods and DTOs.
- Add Avalonia Settings navigation, list, separate create/edit window, deletion confirmation, status refresh, and validation.
- Add the approved design-system MCP Settings specimen before production UI work.

## Contracts and generated artifacts

- Update `contracts/agent-sdk/README.md`, `contracts/sqlite-schema.md`, and `contracts/persistence.md` by hand.
- No generated protocol or binding artifacts.

## Configuration and persistence

- Add the dedicated global `mcp_server` table and narrowly additive initialization support.
- Store desired transport configuration and enabled state; keep runtime state in memory.
- Launch local stdio servers with an OS-specific allowlisted default environment, deterministic runtime fallbacks, standard tool lookup paths, and explicit configured-entry overrides.

## Tests

- Add database, data, MCP transport, core agent, SDK/binding, and Avalonia focused tests.
- Use deterministic mock MCP stdio and HTTP servers in Rust tests.

## Documentation

- Update `DESIGN.md` with the MCP Settings interaction contract.
- On completion, promote stable behavior to feature/spec records and record accepted security/scope tradeoffs in `DECISIONS.md`.
