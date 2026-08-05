# Suncode Product Overview

Suncode is a coding-agent platform comparable in purpose to OpenCode, Claude Code, and Codex. It provides Qt desktop, web, and mobile interfaces backed by either a local runtime or an isolated cloud-hosted runtime.

## Product goals

- Give users a consistent coding-agent experience across local and remote workspaces.
- Support local execution and cloud-hosted execution without changing the three logical layers.
- Keep machine-affecting operations and permission enforcement in a trusted Rust core colocated with its workspace.
- Keep model integration, context construction, and agent orchestration in TypeScript.
- Preserve durable, recoverable session history within the selected local or cloud deployment.
- Make contracts language-neutral and generated for Rust and TypeScript.

## Current status

The foundational architecture is approved. Product behavior and the repository harness described in `ARCHITECTURE.md` are not yet implemented unless corresponding source files and completed requirement records exist.

## Initial exclusions

The foundational milestone excludes functional agent loops, model providers, machine operations, persistence implementations, product UIs, distribution, cloud-hosting infrastructure, and parallel subagents within a session.
