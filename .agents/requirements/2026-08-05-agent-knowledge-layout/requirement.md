# Requirement

## Background

The repository stored approved design documents below a tool-specific documentation path and retained local brainstorming process state. That tool integration has been removed.

## Goal

Adopt the reference project’s `.agents` knowledge-base pattern and keep all durable agent-facing project context in a clear, tool-independent structure.

## Requirements

- Add project-level contributor guidance and a `.agents` index.
- Separate stable architecture, decisions, features, technical specs, and dated requirements.
- Preserve the approved foundational architecture and the completed Git-ignore delivery record.
- Remove obsolete tool-specific directives, directories, and runtime artifacts.
- Avoid creating speculative product source directories before implementation begins.

## Acceptance criteria

No legacy tool path or directive remains, and all durable project context is reachable from `.agents/README.md` without content loss.