# Progress

- Status: Implemented
- Last updated: 2026-09-08

## Completed

- Inspected the current Rust core, fixed tool catalog, policy path, SDK settings facade, SQLite initialization, Settings design-system page, and Avalonia Settings structure.
- Compared applicable MCP lifecycle and dynamic tool-catalog patterns in the local OpenCode and pi source trees.
- Drafted the requirement and proposed architecture.
- Added the MCP Settings design-system specimen.
- Added the additive `mcp_server` SQLite schema and typed CRUD operations with optimistic revisions, deterministic prefixes, secret patching, validation, and the explicit 15-to-16-table initialization path.
- Added the Rust `suncode-mcp` adapter and project-scoped runtime manager for stdio and Streamable HTTP, bounded discovery/results, generation-safe reconciliation, notification refresh, cancellation, and redacted failures.
- Rebuilt the effective MCP catalog before provider calls and routed namespaced MCP tools through schema validation, opaque external-tool approval, session tool activity, and stale-generation rejection.
- Added typed Rust, C ABI, and C# SDK operations and bumped the native ABI to version 5.
- Added the first-level Avalonia MCP Settings page, status polling while visible, enable/disable/retry/edit/delete interactions, and the separate native create/edit window.
- Updated product, architecture, design, SDK, persistence, schema, and decision records.

## Verification

- `cargo test --workspace --manifest-path agent/Cargo.toml`: passed (93 unit tests plus doc tests).
- `cargo test --manifest-path sdks/rust/Cargo.toml --lib`: passed (14 tests).
- `cargo test --manifest-path sdks/c/Cargo.toml`: passed (2 tests).
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj`: passed (62 tests).
- `npm run build` in `design-system/`: passed.
- Browser review passed for light/dark themes, no modal backdrop, Escape dismissal, and no horizontal page overflow at 1280 px.
- `git diff --check`: passed after the final documentation update.

## Blocked

- None.

## Log

### 2026-09-08

- Requirement initialized as a proposal; no production implementation started.
- Proposed global server definitions with project-scoped connections, tools-only scope, stdio plus Streamable HTTP, and approval per opaque MCP call unless Full Control is active.
- User approved implementation and required MCP create/edit to open a separate native window without a backdrop.
- Replaced the MCP modal specimen with a separate 620 x 610 native-window specimen. The Settings window remains visible without a backdrop; the form scrolls independently, its actions stay pinned, and Escape closes the editor.
- Confirmed `rmcp` 3.2.0 supports the required client, child-process, and Streamable HTTP client features on the repository's Rust toolchain.
- Completed the Rust persistence/runtime, SDK binding, and Avalonia implementation.
- Aligned the production editor's default startup timeout with the 30-second design-system value.
