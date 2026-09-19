# Requirement

## Background

SunCode plans to use Language Server Protocol processes to improve agent coding tasks with diagnostics, definitions, references, hover information, and symbols. The desktop design system needs to specify configuration, project runtime status, and failure recovery before production implementation begins.

## Goals

- Add a first-level Language servers destination to the Desktop Settings specimen.
- Specify configured, indexing, failed, disabled, empty, editing, deletion, and responsive states.
- Make the project-scoped runtime states legible without adding redundant explanatory banners.
- Keep the surface consistent with the existing Settings and MCP interaction system.

## Non-goals

- Implement LSP behavior in Rust.
- Modify Avalonia production UI.
- Turn the read-only editor into an IDE or editable code surface.
- Specify automatic rename, code actions, server-requested edits, or command execution.

## Requirements

- Language server definitions remain in a stable content list rather than becoming navigation children.
- Each row shows identity, executable, language scope, runtime state, enablement, and compact actions.
- The editor is a separate native window with structured executable and argument fields, language IDs, root markers, initialization options, environment, timeouts, and enabled state. Server name, executable, and at least one language ID are required before submission. Environment values are write-only after saving; edits use `NAME=value` to replace an entry and `-NAME` to remove one without revealing stored values.
- Copy distinguishes persistent desired state from the selected project's runtime state.

## Edge cases

- Missing executable or startup failure.
- Indexing that has not completed.
- Disabled server.
- No configured servers.
- Long commands, language lists, root markers, and error messages.
- Narrow Settings content width.

## Acceptance criteria

- The design-system Settings specimen exposes and exercises the Language servers page.
- Both list and editor states are keyboard-accessible and responsive.
- `DESIGN.md` records the durable Language Server Settings contract.
- Design-system build, formatting check, and repository diff checks pass.

## Open questions

- None for the design-system delivery. Production protocol and persistence details remain future implementation work.
