# Repository Guidance

This repository is Suncode, a local-first coding agent. Unless the user requests otherwise, communicate and document work in English.

## Current state

The project is in its foundational stage. The approved target architecture is documented, but no product behavior and no source workspaces exist yet. Do not describe planned components as implemented.

## Start here

Before non-trivial work:

1. Read `.agents/README.md` and `.agents/AGENTS.md`.
2. Read `.agents/PRODUCT.md` and `.agents/ARCHITECTURE.md` for project-wide changes.
3. Read the relevant dated requirement, feature, specification, and decision records.
4. For a new substantial delivery, create `.agents/requirements/YYYY-MM-DD-short-topic/` from `.agents/requirements/_template/`.

## Architecture boundaries

- Rust is the audited execution point for filesystem, process, project-boundary, and OS-capability operations. It owns primitives, not semantics; keep its surface narrow.
- TypeScript owns model integrations, context construction, agent loops, orchestration, durable state, and the client API.
- Clients consume the client API; they do not access Rust, SQLite, or model providers directly. Committed surfaces are CLI/TUI first, then Qt desktop.
- The desktop application must use Qt. Electron is prohibited.
- Protocol contracts are written documents, hand-implemented per language and verified by shared test vectors. Nothing is generated.
- The agent runtime uses Node.js. Bun is prohibited.
- The Rust boundary is not an OS-enforced sandbox around the runtime. Its value is containing third-party code and providing one auditable path. Do not write designs that assume it isolates a compromised runtime.
- Suncode is local-first. Do not add tenancy, remote identity, or hosted-infrastructure assumptions.
- Vocabulary: **project** (a directory tree the user opened), **session** (one conversation), **turn** (one user submission and its execution). "Workspace" and "task" are retired as domain nouns.

## Working principles

- Understand the affected boundary and contracts before editing.
- Make the smallest coherent change and avoid speculative scaffolding.
- Preserve user changes and avoid unrelated formatting or cleanup.
- Verify focused behavior first, then broaden checks according to risk.
- Keep `.agents/` current when requirements, architecture, specifications, or durable decisions change.
- Never add credentials, tokens, personal paths, runtime logs, process IDs, or caches to tracked project knowledge.

## Repository layout

The intended product layout is defined in `.agents/ARCHITECTURE.md`. Create its directories only when a milestone needs real buildable files. Durable contributor and agent context belongs in `.agents/`; transient local tool state belongs in ignored `.codex/` or `.claude/` directories.

Note: an empty untracked `docs/` tree may exist from earlier tooling. It is not the `docs/` described in the architecture and can be deleted.

## Completion checks

Before reporting completion, inspect the diff, run `git diff --check`, run applicable tests or validation commands, add or update shared test vectors when a contract changes, and state any checks that could not run.
