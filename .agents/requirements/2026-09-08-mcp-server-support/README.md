# MCP Server Support

- Date: 2026-09-08
- Status: Implemented
- Related features: `agent-phase-1`
- Related specifications: `agent-phase-1`, `agent-sdk`, `sqlite-schema`
- Related decisions: architecture approved for implementation on 2026-09-08

## Documents

- `requirement.md`
- `architecture.md`
- `changes.md`
- `plan.md`
- `progress.md`
- `todo.md`
- `test-plan.md`

The approved architecture is implemented. MCP create and edit use a separate native window without a modal backdrop. Focused Rust, SDK, C ABI, Avalonia, and design-system checks pass; live third-party MCP interoperability remains a residual verification item.
