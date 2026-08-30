# Repository Guidance

This repository is SunCode, a general-purpose coding agent. Unless the user requests otherwise, communicate and document work in English.

## Current state

The project has started implementation. A .NET 10 Avalonia desktop client and the Rust Phase 1 agent exist and were focused-tested. Migration to a reusable Rust SDK facade is in progress; do not describe it as complete until provider, agent loop, policy, SQLite, credentials, API, operations, approvals, recovery, and undo are all Rust-owned and the obsolete TypeScript production path is removed. CLI/TUI/Web and executable extensions remain deferred.

## Start here

Before non-trivial work:

1. Read `.agents/README.md` and `.agents/AGENTS.md`.
2. Read `.agents/PRODUCT.md` and `.agents/ARCHITECTURE.md` for project-wide changes.
3. Read the relevant feature, specification, and decision records. Dated delivery packages are intentionally consolidated; create a new requirement package only for new substantial work.
4. For a new substantial delivery, create `.agents/requirements/YYYY-MM-DD-short-topic/` from `.agents/requirements/_template/`.

## Architecture boundaries

- The agent workspace owns the complete Rust implementation. Keep machine operations behind a narrow internal audited dispatcher, separate from provider and agent semantics.
- TypeScript is migration-only and must not remain a Phase 1 production dependency.
- Clients consume the client API or Rust SDK facade; they do not access SQLite or model providers directly. Phase 1 ships the Avalonia desktop client, and CLI/TUI/Web are deferred.
- The production desktop application uses .NET 10 and Avalonia. Other desktop UI toolkits and Electron are not supported production dependencies.
- Protocol contracts are written documents, hand-implemented per language, and verified by focused implementation tests. Nothing is generated.
- The agent core is Rust. Node.js and Bun are prohibited as Phase 1 production runtime dependencies.
- The Rust boundary is not an OS-enforced sandbox around the agent. Its value is containing third-party code and providing one auditable path. Do not write designs that assume it isolates a compromised agent.
- Phase 1 uses an embedded desktop agent. Do not add tenancy, remote identity, or hosted-infrastructure assumptions without an approved requirement.
- Vocabulary: **project** (a directory tree the user opened), **session** (one conversation), **turn** (one user submission and its execution). "Workspace" and "task" are retired as domain nouns.

## UI and interaction design system

The `design-system/` project is the repository-wide source of truth for UI appearance, component behavior, interaction patterns, states, and layout specifications.

- Before changing any user interface or interaction, inspect the relevant page in `design-system/`, its shared component implementation, and the applicable tokens under `design-system/src/styles/tokens/`.
- Implement production UI, including Avalonia views, consistently with the corresponding design-system specification. Reuse established tokens, dimensions, typography, spacing, colors, states, and interaction behavior instead of introducing local alternatives.
- If the required UI or state is not specified, add or update the design-system specimen as part of the same change before treating the production implementation as complete.
- After every UI or interaction change, compare the result with the relevant design-system page and verify all affected states, themes, sizing, alignment, overflow, and interaction behavior.
- Any intentional deviation from the design system must be explicitly justified and must update the design-system specification so the prototype and production client do not diverge.
- The React/Vite design-system project is specification and review tooling only. It must not become a production runtime dependency of the Avalonia desktop client.

## Working principles

- Understand the affected boundary and contracts before editing.
- Make the smallest coherent change and avoid speculative scaffolding.
- Preserve user changes and avoid unrelated formatting or cleanup.
- Verify focused behavior first, then broaden checks according to risk.
- Keep `.agents/` current when requirements, architecture, specifications, or durable decisions change.
- Never add credentials, tokens, personal paths, agent logs, process IDs, or caches to tracked project knowledge.

## Repository layout

The intended product layout is defined in `.agents/ARCHITECTURE.md`. Create its directories only when a milestone needs real buildable files. Durable contributor and agent context belongs in `.agents/`; transient local tool state belongs in ignored `.codex/` or `.claude/` directories.

Note: an empty untracked `docs/` tree may exist from earlier tooling. It is not the `docs/` described in the architecture and can be deleted. Empty legacy `tooling/` directories can also be removed.

## Completion checks

Before reporting completion, inspect the diff, run `git diff --check`, run applicable tests or validation commands, update focused contract tests when a contract changes, and state any checks that could not run.
