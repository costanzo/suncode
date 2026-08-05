# Suncode Product Overview

Suncode is a local coding-agent platform comparable in purpose to OpenCode, Claude Code, and Codex. It provides Qt desktop, web, and mobile interfaces backed by one on-demand local runtime.

## Product goals

- Give users a consistent coding-agent experience across supported interfaces.
- Keep machine-affecting operations and permission enforcement in a trusted Rust core.
- Keep model integration, context construction, and agent orchestration in TypeScript.
- Preserve durable, recoverable session history locally.
- Make contracts language-neutral and generated for Rust and TypeScript.

## Current status

The foundational architecture is approved. Product behavior and the repository harness described in `ARCHITECTURE.md` are not yet implemented unless corresponding source files and completed requirement records exist.

## Initial exclusions

The foundational milestone excludes functional agent loops, model providers, machine operations, persistence implementations, product UIs, distribution, remote execution, and parallel subagents within a session.
