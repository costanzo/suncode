# Repository Guidance

This repository is Suncode, a coding-agent platform supporting local and cloud-hosted execution. Unless the user requests otherwise, communicate and document work in English.

## Current state

The project is in its foundational stage. The approved target architecture is documented, but most product behavior and source workspaces do not exist yet. Do not describe planned components as implemented.

## Start here

Before non-trivial work:

1. Read `.agents/README.md` and `.agents/AGENTS.md`.
2. Read `.agents/PRODUCT.md` and `.agents/ARCHITECTURE.md` for project-wide changes.
3. Read the relevant dated requirement, feature, specification, and decision records.
4. For a new substantial delivery, create `.agents/requirements/YYYY-MM-DD-short-topic/` from `.agents/requirements/_template/`.

## Architecture boundaries

- Rust is the trusted local core for machine operations, persistence, permissions, sessions, and secrets.
- TypeScript owns model integrations, context construction, agent loops, orchestration, and the local runtime API.
- Qt desktop, web, and mobile clients consume the shared client API; they do not access Rust, SQLite, or model providers directly.
- The desktop application must use Qt. Electron is prohibited.
- JSON Schema and OpenRPC contracts are canonical. Generated Rust and TypeScript protocol types are never edited manually.
- Security decisions are enforced at the lowest trusted Rust layer.

## Working principles

- Understand the affected boundary and contracts before editing.
- Make the smallest coherent change and avoid speculative scaffolding.
- Preserve user changes and avoid unrelated formatting or cleanup.
- Verify focused behavior first, then broaden checks according to risk.
- Keep `.agents/` current when requirements, architecture, specifications, or durable decisions change.
- Never add credentials, tokens, personal paths, runtime logs, process IDs, or caches to tracked project knowledge.

## Repository layout

The intended product layout is defined in `.agents/ARCHITECTURE.md`. Create its directories only when a milestone needs real buildable files. Durable contributor and agent context belongs in `.agents/`; transient local tool state belongs in ignored `.codex/` or `.claude/` directories.

## Completion checks

Before reporting completion, inspect the diff, run `git diff --check`, run applicable tests or validation commands, confirm generated artifacts are current when contracts change, and state any checks that could not run.
