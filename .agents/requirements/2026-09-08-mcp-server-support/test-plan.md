# Test Plan

## Scope

Verify MCP desired-state persistence, transport lifecycle, hot catalog replacement, authority, audit, SDK parity, and the Settings experience without adding a production TypeScript dependency.

## Unit tests

- Validate local/remote configuration, names, URLs, commands, timeouts, secret mutations, and tool schema compatibility.
- Verify deterministic namespacing, collision handling, generation checks, state transitions, and redaction.
- Verify MCP content normalization, size bounds, server-declared errors, unsupported content, and cancellation.
- Verify `Risk::ExternalTool` policy and sequential execution.

## Integration and conformance tests

- Exercise initialize/list-tools/call-tool/list-changed/close over deterministic stdio and Streamable HTTP test servers.
- Verify enabled startup reconciliation, failed startup isolation, retry, edit replacement, disable, delete, and rapid stale-generation races.
- Verify SQLite CRUD, optimistic revisions, idempotency, secret redaction, and the exact 15-to-16-table additive initialization path.
- Verify an existing session sees create/edit/enable/disable/delete in the next provider request without reopening the session.
- Verify approval suspension/revalidation and `session_tool_use` projections for success, MCP error, timeout, disconnect, and cancellation.
- Verify Rust/C/C# DTO and error-code parity.

## Regression checks

- Built-in tool definitions and operations are unchanged when MCP has no connected servers.
- Provider request schema remains valid for all six built-in providers.
- Startup remains available when every enabled MCP server fails.
- Existing 15-table databases remain accepted through the one additive compatibility path.
- Session recovery never blindly replays an uncertain MCP call.

## Manual checks

- Settings list, empty state, add, edit, delete confirmation, enable, disable, connecting, connected, failed, retry, and pending controls.
- Light and dark themes at 900 x 672 and the 720 x 552 minimum window.
- Keyboard focus order, Escape/cancel behavior, initial focus, tooltips, text wrapping, long names/URLs/commands, and status announcements.
- Confirm secrets never reappear in edit fields, errors, status text, logs, or Tool activity.
- Confirm local-process and remote-server warnings accurately describe authority and undo limitations.

## Commands and results

- `cargo fmt --manifest-path agent/Cargo.toml --all`: passed.
- `cargo test --workspace --manifest-path agent/Cargo.toml`: passed, including 36 core, 8 data, 1 database, 5 MCP adapter, and existing crate tests (95 unit tests total; doc tests also passed).
- `cargo fmt --manifest-path sdks/rust/Cargo.toml --all`: passed.
- `cargo test --manifest-path sdks/rust/Cargo.toml --lib`: passed (14 tests).
- `cargo fmt --manifest-path sdks/c/Cargo.toml --all`: passed.
- `cargo test --manifest-path sdks/c/Cargo.toml`: passed (2 tests).
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj`: passed (62 tests).
- `npm run build` in `design-system/`: passed.
- Browser review at 1280 px: light/dark render passed, `.dialog-backdrop` count was zero, Escape removed the editor, and document width did not overflow.
- MCP environment tests verify the platform baseline contains launch/runtime values, working-directory metadata, and that configured variables override defaults without removing unrelated baseline values.

## Residual risks

- Cross-platform process-tree behavior and third-party server protocol quirks require real-platform testing beyond mocks.
- Deterministic live stdio/Streamable HTTP conformance fixtures were not added in this delivery; the adapter currently has focused normalization and redaction tests while runtime behavior is covered below the transport boundary.
- The native Avalonia editor was compiled and its typed UI boundary was tested, but a manual 520 x 520 native-window pass was not automated.
- MCP server side effects are not generally reversible by SunCode.
